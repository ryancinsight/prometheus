#!/usr/bin/env python3
"""Check or regenerate Cargo.lock outside the Atlas dependency overlay.

The stack overlay redirects first-party git dependencies to local checkouts.
Resolving there can remove the git sources needed by standalone consumers and
CI. Cargo runs from a neutral directory here; Windows also maps the repository
to a temporary drive root, which is removed before each invocation returns.

Checks first fetch locked dependencies, then resolve default and all-feature
activation offline under --locked. Fetching permits a cold Cargo cache without
changing the committed dependency selection.
Regeneration resolves both activation sets until a bounded full pass leaves the
lock unchanged, then checks both under --locked. Missing cached dependencies,
network failures, and incompatible source requirements retain Cargo's original
diagnostic; they are not classified as lock staleness without evidence.

Usage:
    scripts/lockfile.py --check
    scripts/lockfile.py --check-staged
    scripts/lockfile.py --regenerate
"""

from __future__ import annotations

import argparse
import ctypes
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPOSITORY = Path(__file__).resolve().parent.parent
LOCKFILE = REPOSITORY / "Cargo.lock"
MANIFEST = REPOSITORY / "Cargo.toml"

# Any first-party dependency resolves through one of these. A lock with none of
# them has been flattened by the overlay.
FIRST_PARTY_SOURCE = re.compile(r'^source = "git\+https://github\.com/ryancinsight/', re.M)

# Metadata performs resolution rather than compilation. The deadline bounds a
# blocked registry/file-cache acquisition as well as network access.
RESOLUTION_TIMEOUT_SECONDS = 60
LOCAL_COMMAND_TIMEOUT_SECONDS = 10
ACTIVATIONS = ((), ("--all-features",))
# Allow one pass per activation and a final stability check. Non-convergence
# remains an error rather than silently accepting the final written lock.
RESOLUTION_PASSES = len(ACTIVATIONS) + 1


def run_command(
    arguments: list[str], *, cwd: str | None = None, timeout: int
) -> subprocess.CompletedProcess[str]:
    """Capture a bounded command, preserving failures for the caller."""
    try:
        return subprocess.run(
            arguments,
            cwd=cwd,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            check=False,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        diagnostic = error.stderr or ""
        if isinstance(diagnostic, bytes):
            diagnostic = diagnostic.decode("utf-8", errors="replace")
        return subprocess.CompletedProcess(
            arguments, 124, "", f"command exceeded {timeout}s: {arguments!r}\n{diagnostic}"
        )
    except OSError as error:
        return subprocess.CompletedProcess(arguments, 1, "", str(error))


def run_outside_the_overlay(
    arguments: list[str], manifest: Path | None = None
) -> subprocess.CompletedProcess[str]:
    """Run cargo with a working directory outside the stack root.

    Cargo resolves `.cargo/config.toml` by walking up from the workspace root
    as well as the working directory. On Windows, an absolute manifest path
    therefore still finds the Atlas overlay even when Cargo is launched from a
    neutral temporary directory. A temporary drive mapping makes the
    repository itself the filesystem root for this invocation, so only the
    repository's own configuration remains visible.
    """
    manifest = MANIFEST if manifest is None else manifest.resolve()
    with tempfile.TemporaryDirectory() as neutral_directory:
        drive = unused_windows_drive()
        if drive is not None:
            mapped = run_command(
                ["subst", drive, str(manifest.parent)],
                timeout=LOCAL_COMMAND_TIMEOUT_SECONDS,
            )
            if mapped.returncode != 0:
                return subprocess.CompletedProcess(
                    ["cargo", *arguments],
                    mapped.returncode,
                    mapped.stdout,
                    f"failed to map repository drive {drive}: {mapped.stderr}",
                )
            manifest = Path(f"{drive}\\{manifest.name}")
        completed = None
        try:
            completed = run_command(
                ["cargo", *arguments, "--manifest-path", str(manifest)],
                cwd=neutral_directory,
                timeout=RESOLUTION_TIMEOUT_SECONDS,
            )
        finally:
            if drive is not None:
                remove_drive(drive, completed)
        return completed


def remove_drive(
    drive: str, completed: subprocess.CompletedProcess[str] | None
) -> None:
    """Remove a mapping without losing a preceding command failure."""
    cleanup = run_command(["subst", drive, "/d"], timeout=LOCAL_COMMAND_TIMEOUT_SECONDS)
    if cleanup.returncode != 0:
        diagnostic = f"failed to remove repository drive {drive}: {cleanup.stderr}"
        if completed is not None and completed.returncode != 0:
            diagnostic = (
                f"cargo failed with status {completed.returncode}:\n"
                f"{completed.stderr}\n{diagnostic}"
            )
        raise RuntimeError(diagnostic)


def unused_windows_drive() -> str | None:
    """Return an unused Windows drive root, or ``None`` on other hosts."""
    if os.name != "nt":
        return None

    drive_mask = ctypes.windll.kernel32.GetLogicalDrives()
    for index in range(25, 2, -1):
        if drive_mask & (1 << index) == 0:
            return f"{chr(ord('A') + index)}:"
    raise RuntimeError("no unused Windows drive is available for overlay isolation")


def check_resolution(manifest: Path | None = None) -> int:
    """Hydrate the committed dependency graph before checking both activations."""
    completed = run_outside_the_overlay(["fetch", "--locked"], manifest)
    if completed.returncode != 0:
        print(
            "error: locked dependency hydration failed.\n"
            f"cargo said:\n{completed.stderr.strip()}",
            file=sys.stderr,
        )
        return 1
    return check_cached_resolution(manifest)


def check_cached_resolution(manifest: Path | None = None) -> int:
    """Require both gate activation sets to accept the existing lock offline."""
    for activation in ACTIVATIONS:
        completed = run_outside_the_overlay(
            ["metadata", "--locked", "--offline", "--format-version", "1", *activation],
            manifest,
        )
        if completed.returncode != 0:
            mode = "all features" if activation else "default features"
            print(
                f"error: locked offline resolution failed for {mode}.\n"
                f"cargo said:\n{completed.stderr.strip()}",
                file=sys.stderr,
            )
            return 1
    return 0


def stabilize_resolution(manifest: Path) -> int:
    """Resolve every gate activation until a complete pass leaves the lock unchanged."""
    lockfile = manifest.with_name("Cargo.lock")
    for _ in range(RESOLUTION_PASSES):
        before = lockfile.read_bytes()
        for activation in ACTIVATIONS:
            completed = run_outside_the_overlay(
                ["metadata", "--format-version", "1", *activation], manifest
            )
            if completed.returncode != 0:
                mode = "all features" if activation else "default features"
                print(
                    f"error: resolution failed for {mode}:\n{completed.stderr.strip()}",
                    file=sys.stderr,
                )
                return 1
        if lockfile.read_bytes() == before:
            return check_cached_resolution(manifest)
    print(
        f"error: Cargo.lock changed after each of {RESOLUTION_PASSES} activation passes",
        file=sys.stderr,
    )
    return 1


def check() -> int:
    if not LOCKFILE.is_file():
        print(f"error: {LOCKFILE} does not exist", file=sys.stderr)
        return 1

    sources = len(FIRST_PARTY_SOURCE.findall(LOCKFILE.read_text(encoding="utf-8")))
    if sources == 0:
        print(
            "error: Cargo.lock contains no first-party git sources.\n"
            "\n"
            "It was regenerated with the Atlas stack overlay active, which\n"
            "resolves those dependencies to local paths and drops their git\n"
            "sources. CI has no overlay and will fail every --locked job.\n"
            "\n"
            "Fix: scripts/lockfile.py --regenerate",
            file=sys.stderr,
        )
        return 1

    if check_resolution() != 0:
        return 1

    print(f"Cargo.lock resolves under --locked; {sources} first-party git sources.")
    return 0


def check_staged() -> int:
    """Structural check of the *staged* `Cargo.lock`, for use from `pre-commit`.

    Deliberately does not run cargo. A pre-commit hook has to be fast enough that
    nobody reaches for `--no-verify`, and the flattened lock has an unmistakable
    signature -- zero first-party git sources -- that a text scan settles
    instantly. Staleness, the other failure `--check` detects, needs real
    resolution and stays a pre-push concern.

    Checking the *staged blob* rather than the working file is the point: the
    working copy may already have been repaired while the poisoned version sits
    in the index, and it is the index that becomes the commit.
    """
    staged = run_command(
        ["git", "diff", "--cached", "--name-only", "--", "Cargo.lock"],
        timeout=LOCAL_COMMAND_TIMEOUT_SECONDS,
    )
    if staged.returncode != 0:
        print(f"error: staged paths could not be read:\n{staged.stderr}", file=sys.stderr)
        return 1
    if not staged.stdout.strip():
        return 0

    blob = run_command(
        ["git", "show", ":Cargo.lock"],
        timeout=LOCAL_COMMAND_TIMEOUT_SECONDS,
    )
    if blob.returncode != 0:
        print(f"error: staged Cargo.lock could not be read:\n{blob.stderr}", file=sys.stderr)
        return 1

    if len(FIRST_PARTY_SOURCE.findall(blob.stdout)) > 0:
        return 0

    print(
        "error: the staged Cargo.lock contains no first-party git sources.\n"
        "\n"
        "A cargo command run against a tree under the Atlas stack root rewrote\n"
        "it with the overlay active, which resolves those dependencies to local\n"
        "paths and drops their git sources. Committing it now is what turns a\n"
        "working branch into one that can never be pushed.\n"
        "\n"
        "Fix: scripts/lockfile.py --regenerate, then stage the result.",
        file=sys.stderr,
    )
    return 1


def regenerate() -> int:
    completed = run_outside_the_overlay(["generate-lockfile"])
    if completed.returncode != 0:
        print(f"error: regeneration failed:\n{completed.stderr.strip()}", file=sys.stderr)
        return 1
    if stabilize_resolution(MANIFEST) != 0:
        return 1
    print("Cargo.lock regenerated outside the overlay for default and all features.")
    return check()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="verify the committed lock")
    mode.add_argument("--regenerate", action="store_true", help="rewrite the lock correctly")
    mode.add_argument(
        "--check-staged",
        action="store_true",
        help="fast structural check of the staged lock, for pre-commit",
    )
    arguments = parser.parse_args()
    if arguments.regenerate:
        return regenerate()
    if arguments.check_staged:
        return check_staged()
    return check()


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
