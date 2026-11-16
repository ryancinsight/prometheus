# Prometheus Backlog

Open items in priority order, under the milestone ladder below
([ADR 0003](docs/adr/0003-release-scope-1-0.md)); every item carries
`milestone:`. The meta item `ATLAS-PROMETHEUS-PROMOTION-2026-09-03` (atlas
`backlog.md#prometheus-promotion`) owns registration and first publication and
is not copied here. Evidence below was read at `basis:` unless stated
otherwise; no test was executed.

<a id="M-rate-typing"></a>
## M-rate-typing (0.2.0): the rate-constant dimension is enforced — needs: none
- exit: a dimensionally wrong coefficient is rejected at compile time or as a typed error, for `mass_action_rate` and `ReactionNetwork::evaluate`; runnable check `cargo nextest run --locked --all-features -- rate` plus the wrong-order rejection test — fails on a build where a first-order constant with order-2 reactants returns a rate.
- co-evolution at release: a consumer of `mass_action_rate` or `ReactionNetwork::new` (M-ecosystem names kwavers and CFDrs) migrates to `Result<ReactionRate<T>, RateOrderMismatch>` and `RateConstant<T>` when 0.2.0 releases, per [docs/migration.md](docs/migration.md); receipts live on their boards, and the release item filed when this milestone closes carries this note.
<a id="M-oracle-hardening"></a>
## M-oracle-hardening (0.2.0): the network oracle and power edges are pinned — needs: none
- exit: narrowing delivered (PR #80: the dense reference raises each base by repetition); the `-0.0`/`-infinity`/subnormal edge tests pass and kill the `base.abs()` mutant — the remaining check fails without its fix.
<a id="M-board-hygiene"></a>
## M-board-hygiene (0.2.0): the changelog states the shipped cohort — needs: none
- exit: every provider version named in `CHANGELOG.md` equals the `Cargo.toml` requirement at the head — the grep comparison fails on drift.
<a id="M-1.0"></a>
## Milestone M-1.0 (1.0.0): stable kinetics release (authority-gated) — needs: M-rate-typing, M-oracle-hardening, M-board-hygiene
- exit: `cargo semver-checks`, publish dry-run, `cargo doc`, nextest/doctest green at all features, zero open correctness items. Veto: release authority sets the API-freeze contents; never forces `publish = true` without it.
<a id="M-ecosystem"></a>
## M-ecosystem: consumer-owned migrations (kwavers, CFDrs) — needs: M-1.0
- exit: consumer receipts on their own boards; does not gate 1.0.

<a id="PROMETHEUS-RATE-DIMENSION-SEAL"></a>
## PROMETHEUS-RATE-DIMENSION-SEAL — Seal RateConstantDimension against downstream impls [patch] — todo
- priority: correctness; needs: none
- milestone: M-rate-typing.
- basis: dc23785
- scope: `src/rate/dimension.rs`, `docs/adr/0002-rate-constant-typing.md`
- Defect (read at basis): `pub trait RateConstantDimension { const ORDER: i32; }` has no supertrait, so a downstream `impl RateConstantDimension for Local { const ORDER: i32 = 2; }` compiles and `Local` then meets the `D: RateConstantDimension` bound of `mass_action_rate` and `RateConstant::from`: a caller-declared order tag, the design [ADR 0002](docs/adr/0002-rate-constant-typing.md) rejects. The trait docs say it is "implemented for exactly" the `mol^(1-n)·m^(3n-3)·s⁻¹` dimensions, false for such a type. The sentence "Each dimension below differs from the second-order constant in one exponent" is also false for the range-edge examples (amount and length differ) and the semantics-marker example (the marker differs).
- Outcome: the trait is sealed (private supertrait, implemented only for the one `Dimension<…>` shape under its existing bounds); the docs state what each example varies; ADR 0002 carries a dated revision line.
- Acceptance: a `compile_fail` doctest on the trait pinned to the code rustc reports (expected `E0277`) for `struct Local; impl RateConstantDimension for Local { const ORDER: i32 = 2; }`. That impl alone compiles at basis (no supertrait), so the doctest fails there; the existing `compile_fail` doctests and `order_*` tests stay green.
- Next step: add the sealed supertrait in a private module, reuse the existing bounds on its one impl, add the doctest, reword the docs, add the ADR 0002 revision line.

<a id="PROMETHEUS-RELEASE-LADDER"></a>
## PROMETHEUS-RELEASE-LADDER — Obtain the independent judge verdict on the release ladder [patch] — todo
- priority: verification; needs: none
- milestone: M-1.0 (exit evidence).
- basis: dc23785
- scope: `docs/adr/0003-release-scope-1-0.md`, `backlog.md`
- Gap (read at basis): the ladder and [ADR 0003](docs/adr/0003-release-scope-1-0.md) landed through PR #79 under `judge-required`, but no judge verdict is recorded: no commit status on PR #79's head or on its merge commit, and no comments or reviews on it. The obligation of the original item is open.
- Outcome: an independent judge has falsified the ladder at its default-branch head and its findings are fixed or filed.
- Acceptance: a judge verdict that did not generate the ladder maps every capability area of the ADR table and every open feature item (none open at basis) to a check or a non-goal, and finds each exit check runnable and failing without its feature; the verdict is posted on the PR carrying any fix, or on the PR that closes this item.
- Next step: dispatch a judge on `backlog.md` and ADR 0003 at the fetched default head, with this acceptance and the exit lines as its oracle.

<a id="PROMETHEUS-NO-STD-TEST-TARGETS"></a>
## PROMETHEUS-NO-STD-TEST-TARGETS — Build the test targets without default features [patch] — todo
- priority: verification; needs: none
- milestone: M-1.0 (exit evidence: tests green at all features).
- basis: dc23785
- scope: `Cargo.toml`, `tests/common/mod.rs`, `tests/rate.rs`, `tests/integration.rs`, `tests/production.rs`, `tests/stoichiometry.rs`, `.github/workflows/ci.yml`
- Gap: CI checks `--no-default-features` for the library only (`cargo check --locked --no-default-features`) and runs clippy with `--all-features`. `cargo clippy --all-targets --no-default-features` was reported failing to compile test targets before filing and was not rerun here. Read at basis: `tests/common/mod.rs:9` imports `StoichiometricMatrix`, which `src/lib.rs` gates on `std`, and every test file but `rate.rs` includes `common`; `tests/integration.rs:11-16` and `tests/rate.rs:23-24` import the optional `horae`.
- Outcome: the no-default-features test targets compile, and each target that needs `std` declares `required-features = ["std"]`.
- Acceptance: `cargo clippy --locked --all-targets --no-default-features -- -D warnings` exits 0 and runs as a CI step; it fails at basis. `cargo nextest run --locked --no-default-features` still runs `tests/species.rs` and `tests/enthalpy.rs`, which use only `common::assert_close`.
- Next step: gate `water_formation` and its imports in `tests/common/mod.rs` on `cfg(feature = "std")`, add `[[test]] required-features` for `integration`, `rate`, `production` and `stoichiometry`, then add the CI step.

<a id="PROMETHEUS-CHANGELOG-DRIFT"></a>
## PROMETHEUS-CHANGELOG-DRIFT — Bring the Unreleased changelog in line with the manifest and lock [patch] — todo
- priority: verification; needs: none
- milestone: M-board-hygiene.
- basis: dc23785
- scope: `CHANGELOG.md`
- Defect (read at basis; the original version drift is gone, each provider version the entry names equals `Cargo.toml`): (1) the cohort entry says the lock "resolves each at its repository's current default head"; `Cargo.lock` pins `aequitas`, `eunomia`, `horae-time` and `leto` at revisions that differ from `git ls-remote <repository> HEAD` (read 2026-11-16), so the sentence is false for all four and false again after any provider commit. (2) `[Unreleased]` carries two `### Added` headings, with `### Changed` between them.
- Outcome: the Unreleased section states only what the manifest, the lock and ADR 0001 state, under one `Added` and one `Changed` heading, in the repository's changelog convention.
- Acceptance: `git grep -n 'default head' CHANGELOG.md` prints nothing (the sentence wraps across two lines, so a search for `current default head` finds nothing at basis); `git grep -c '^### Added' CHANGELOG.md` prints `CHANGELOG.md:1` (`CHANGELOG.md:2` at basis); each provider version named equals the `Cargo.toml` requirement (`git grep -n 'eunomia\|leto\|aequitas\|horae' CHANGELOG.md Cargo.toml`). The first two fail at basis; the third holds at basis and guards the edit.
- Next step: re-read each cohort statement against `Cargo.toml` and `Cargo.lock` (the ADR 0001 entry names Leto 0.47, the lock resolves leto 0.50.0), edit the entry, then run the docs and link checks of the committed gate.

<a id="PROMETHEUS-ORDER-SIGNED-EDGE-TESTS"></a>
## PROMETHEUS-ORDER-SIGNED-EDGE-TESTS — Pin signed zero, infinity and subnormal bases above order 2^31 [patch] — todo
- priority: verification; needs: none
- milestone: M-oracle-hardening.
- basis: dc23785
- scope: `src/rate.rs` (module `tests`)
- Gap (read at basis): the `reaction_power` tests cover bases -2, -0.5, -1, +0, +infinity and NaN at orders above `i32::MAX`. No test pins a `-0.0` base, a `-infinity` base at an odd order, or a subnormal base (positive and negative, `f64` and `f32` minimum subnormal) above `2^31`.
- Outcome: each case asserts its IEEE result bit for bit through `assert_exact_at_both`: `-0.0` gives `-0.0` at an odd order and `+0.0` at an even one, `-infinity` gives `-infinity` at an odd order and `+infinity` at an even one, and a subnormal base underflows to a zero carrying the parity sign.
- Acceptance: the new tests pass at head, and the `-0.0` and `-infinity` tests fail under a mutant that applies `base.abs()` only when the base is zero or infinite, which the existing tests (`+0.0` and `+infinity` only) do not kill. The subnormal tests pin IEEE underflow behavior and kill no distinct mutant, because the mutant leaves a nonzero finite base unchanged.
- Next step: append the cases to `src/rate.rs` `tests`, then run the mutants and record which test kills each.

<a id="PROMETHEUS-ORDER-SUM-SATURATION-TEST"></a>
## PROMETHEUS-ORDER-SUM-SATURATION-TEST — Kill the saturating_add to wrapping_add mutant in the order sum [patch] — todo
- priority: verification; needs: none
- milestone: M-oracle-hardening.
- basis: dc23785
- scope: `src/rate.rs` (`check_order` and module `tests`)
- Gap: `check_order` forms the reactant order sum as `fold(0_u64, |total, order| total.saturating_add(u64::from(order)))`. A mutant replacing `saturating_add` with `wrapping_add` was reported surviving the tests before filing (not rerun here). It is not equivalent: `2^32 + 1` orders of `u32::MAX` sum to `2^64 - 1`, and one further order 3 saturates to `u64::MAX` (an error against declared order 2) but wraps to `2` (accepted).
- Outcome: a value-semantic test pins the saturation, with a workload that fits the committed test budget.
- Acceptance: the test asserts `Err(RateOrderMismatch { reactant_order: u64::MAX, .. })` for declared order 2; it passes at head and `cargo mutants --file src/rate.rs` reports the `wrapping_add` mutant caught.
- Next step: run `check_order(2, core::iter::repeat_n(u32::MAX, (1 << 32) + 1).chain([3]))` under nextest; if it exceeds the 30 s slow period, extract the per-order step as a function over `(u64, Order)` and assert `u64::MAX - 1` plus order 3 gives `u64::MAX`, an extraction and never a smaller workload.
