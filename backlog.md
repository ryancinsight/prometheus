# Prometheus Backlog

<a id="prometheus-stable-qualification"></a>
## PROMETHEUS-STABLE-QUALIFICATION — prometheus stable-release qualification [patch] — todo
- priority: verification; milestone: M-rate-typing; basis: `b98d17547cd3`.
- Outcome: Enforce rate-constant dimensions at the reaction boundary. Follow the [release scope](docs/adr/0003-release-scope-1-0.md) and `M-rate-typing`. Planning targets for `prometheus-kinetics` are 0.1.1 if compatible oracle fixes land first, then 0.2.0, then 1.0.0. Verification alone can advance the current version. Use Horae Time 0.4.0 for the [verified downstream contract](gap_audit.md#verified-downstream-provider-migration).
- Representative manifest `prometheus-kinetics 0.1.0`. Target 1.0.0. Index coverage at review: 0/1 publish-enabled workspace versions.
- [ ] [arch] Confirm the independent release-ladder review and the error contract for rate order before changing the public API. Document the pre-1.0 caller migration.
- [ ] [minor] Exercise mass_action_rate and ReactionNetwork::evaluate with matching and mismatched rate dimensions. Verify the expected rate, exact conservation for an analytically balanced system and the specified rejection.
- [ ] [major] Finish oracle hardening and the no-default-feature test targets. Verify first-publication package identity and an independent consumer before stable release qualification.
- [ ] [patch] **0.1.1 if compatible oracle fixes land first.** Run the existing kinetics suite against a balanced reaction. Compare the rate to the analytical expression and verify stoichiometric conservation and boundary behavior at zero concentration. [Source and baseline](gap_audit.md#incremental-release-baseline).
- [ ] [major] **0.2.0.** After rate-typing review, migrate the caller to the approved dimensional contract. Exercise matching and mismatched reaction order, the documented failure and the full network result with default features off.
- [x] [patch] Verify Horae Time 0.4.0 through the real consumer and independent dependency resolution. [Executed coverage](gap_audit.md#verified-downstream-provider-migration).
- [ ] [major] **1.0.0-beta.1 → 1.0.0-rc.1 → 1.0.0.** Freeze the supported surface at beta. A fresh consumer of the first admitted archive reproduces the reaction law and the board’s rate, oracle and cohort exits. Repeat the applicable exits on the exact RC before stable promotion. Attach exact-revision evidence for the local exits and this member’s [Atlas qualification criteria](https://github.com/ryancinsight/atlas/blob/17b0124a7548ad81e817284afc5cb115c1d1bc78/docs/parity/repositories.json). Require complete implementations and passing product-path acceptance for all critical features.

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

<a id="PROMETHEUS-RELEASE-LADDER"></a>
## PROMETHEUS-RELEASE-LADDER — Obtain the independent judge verdict on the release ladder [patch] — todo
- priority: verification; needs: none
- milestone: M-1.0 (exit evidence).
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `docs/adr/0003-release-scope-1-0.md`, `backlog.md`
- Gap (read at basis): the ladder and [ADR 0003](docs/adr/0003-release-scope-1-0.md) landed through PR #79 under `judge-required`, but no judge verdict is recorded: no commit status on PR #79's head `d78a862` or on its merge commit `6bdf5c9`, and no comments or reviews on it (hosting API read 2026-11-16). The obligation of the original item is open.
- Outcome: an independent judge has falsified the ladder at its default-branch head and its findings are fixed or filed.
- Acceptance: a judge verdict that did not generate the ladder maps every capability area of the ADR table and every open feature item (none open at basis) to a check or a non-goal, and finds each exit check runnable and failing without its feature; the verdict is posted on the PR carrying any fix, or on the PR that closes this item.
- Cases the judge also tries (the exits' runnable checks were not run at `basis:`): (a) the seal landed in `ef30785` and ADR 0003 (:108-111) states it delivered with no exit change: does the seal's `compile_fail,E0277` doctest (`src/rate/dimension.rs:22-28`) join the `M-rate-typing` exit, a judged ADR 0003 revision; (b) the `M-board-hygiene` exit fails at basis: `CHANGELOG.md:38-47` name Eunomia 0.12, Leto 0.47, `aequitas` 0.3.1, `eunomia` 0.12.0, `leto` `>=0.47.1, <0.51.0` and `horae-time` 0.2.2, against `Cargo.toml:24-32` (aequitas 0.5.0, eunomia 0.12.1, leto 0.52.0, horae-time 0.4.0); the judge decides whether the exit closes with `PROMETHEUS-CHANGELOG-DRIFT` and whether its grep comparison is the check the exit means.
- Next step: dispatch a judge on `backlog.md` and ADR 0003 at the fetched default head, with this acceptance, the cases above and the exit lines as its oracle.

<a id="PROMETHEUS-NO-STD-TEST-TARGETS"></a>
## PROMETHEUS-NO-STD-TEST-TARGETS — Build the test targets without default features [patch] — todo
- priority: verification; needs: none
- milestone: M-1.0 (exit evidence: tests green at all features).
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `Cargo.toml`, `tests/common/mod.rs`, `tests/rate.rs`, `tests/integration.rs`, `tests/production.rs`, `tests/stoichiometry.rs`, `.github/workflows/ci.yml`
- Gap: CI checks `--no-default-features` for the library only (`cargo check --locked --no-default-features`) and runs clippy with `--all-features`. `cargo clippy --all-targets --no-default-features` was reported failing to compile test targets before filing and was not rerun here. Read at basis: `tests/common/mod.rs:9` imports `StoichiometricMatrix`, which `src/lib.rs` gates on `std`, and every test file but `rate.rs` includes `common`; `tests/integration.rs:11-16` and `tests/rate.rs:23-24` import the optional `horae`.
- Outcome: the no-default-features test targets compile, and each target that needs `std` declares `required-features = ["std"]`.
- Acceptance: `cargo clippy --locked --all-targets --no-default-features -- -D warnings` exits 0 and runs as a CI step; it fails at basis. `cargo nextest run --locked --no-default-features` still runs `tests/species.rs` and `tests/enthalpy.rs`, which use only `common::assert_close`.
- Next step: gate `water_formation` and its imports in `tests/common/mod.rs` on `cfg(feature = "std")`, add `[[test]] required-features` for `integration`, `rate`, `production` and `stoichiometry`, then add the CI step.

<a id="PROMETHEUS-ERROR-TRAITS"></a>
## PROMETHEUS-ERROR-TRAITS — Implement Display and Error for the public error types [minor] — todo
- priority: verification; needs: none
- milestone: M-rate-typing.
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `src/network.rs`, `src/stoichiometry.rs`, `src/concentration.rs`, `src/species.rs`, `src/enthalpy.rs`, `tests/`
- Gap (read at basis): only `RateOrderMismatch` implements `Display` and `core::error::Error` (`src/rate.rs:62-72`); `InvalidNetwork` (`src/network.rs:22`), `StateDimensionMismatch`, `InvalidConcentration`, `NonPositiveMolarMass`, `InvalidStoichiometry`, `MisshapedMasses`, `MisshapedRates` and `MisshapedEnthalpies` implement neither. Six of the eight are structs; `InvalidConcentration` (`src/concentration.rs:20`) and `NonPositiveMolarMass` (`src/species.rs:39`) are unit structs with no value to name.
- Outcome: each type implements both traits; `Display` names the broken invariant, and the offending values for each type that carries them.
- Acceptance: one test per enum variant and one per struct asserts the `Display` text through a `&dyn core::error::Error` coercion, following `tests/rate.rs:317-328`; a missing impl is a compile error in that test. For each type that carries values, the test builds the error from values it chooses and asserts the text contains each of them formatted (or equals text built from those values), and a second error built from different values asserts a different text, so a message that omits a value fails; the two unit structs assert the invariant's name. Fails at basis: eight types have no impl.
- Next step: add the impls beside each type, following `src/rate.rs:62-72`, and the tests in the target that already names each type (`tests/species.rs`, `tests/stoichiometry.rs`, `tests/production.rs`, `tests/enthalpy.rs`, `tests/rate.rs`). `StateDimensionMismatch` (`src/network.rs:62`, built at `:165,:198`) is named by no test target, so its test is new, in `tests/integration.rs` beside the `ReactionNetwork::evaluate` tests.
- Open (ladder decision): `priority: verification` fits an additive public API gap poorly; the ladder judge sets the class.

<a id="PROMETHEUS-ORDER-SIGNED-EDGE-TESTS"></a>
## PROMETHEUS-ORDER-SIGNED-EDGE-TESTS — Pin signed zero, infinity and subnormal bases above order 2^31 [patch] — todo
- priority: verification; needs: none
- milestone: M-oracle-hardening.
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `src/rate.rs` (module `tests`)
- Gap (read at basis): the `reaction_power` tests cover bases -2, -0.5, -1, +0, +infinity and NaN at orders above `i32::MAX`. No test pins a `-0.0` base, a `-infinity` base at an odd order, or a subnormal base (positive and negative, `f64` and `f32` minimum subnormal) above `2^31`.
- Outcome: each case asserts its IEEE result bit for bit through `assert_exact_at_both`: `-0.0` gives `-0.0` at an odd order and `+0.0` at an even one, `-infinity` gives `-infinity` at an odd order and `+infinity` at an even one, and a subnormal base underflows to a zero carrying the parity sign.
- Acceptance: the new tests pass at head, and the `-0.0` and `-infinity` tests fail under a mutant that applies `base.abs()` only when the base is zero or infinite, which the existing tests (`+0.0` and `+infinity` only) do not kill. The subnormal tests pin IEEE underflow behavior and kill no distinct mutant, because the mutant leaves a nonzero finite base unchanged.
- Next step: append the cases to `src/rate.rs` `tests`, then run the mutants and record which test kills each.

<a id="PROMETHEUS-ORDER-SUM-SATURATION-TEST"></a>
## PROMETHEUS-ORDER-SUM-SATURATION-TEST — Kill the saturating_add to wrapping_add mutant in the order sum [patch] — todo
- priority: verification; needs: none
- milestone: M-oracle-hardening.
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `src/rate.rs` (`check_order` and module `tests`)
- Gap: `check_order` forms the reactant order sum as `fold(0_u64, |total, order| total.saturating_add(u64::from(order)))`. A mutant replacing `saturating_add` with `wrapping_add` was reported surviving the tests before filing (not rerun here). It is not equivalent: `2^32 + 1` orders of `u32::MAX` sum to `2^64 - 1`, and one further order 3 saturates to `u64::MAX` (an error against declared order 2) but wraps to `2` (accepted).
- Outcome: a value-semantic test pins the saturation, with a workload that fits the committed test budget.
- Acceptance: the test asserts `Err(RateOrderMismatch { reactant_order: u64::MAX, .. })` for declared order 2; it passes at head and `cargo mutants --file src/rate.rs` reports the `wrapping_add` mutant caught.
- Next step: run `check_order(2, core::iter::repeat_n(u32::MAX, (1 << 32) + 1).chain([3]))` under nextest; if it exceeds the 30 s slow period, extract the per-order step as a function over `(u64, Order)` and assert `u64::MAX - 1` plus order 3 gives `u64::MAX`, an extraction and never a smaller workload.

<a id="PROMETHEUS-DOCTEST-ERROR-CODES"></a>
## PROMETHEUS-DOCTEST-ERROR-CODES — Check the error codes of the compile_fail doctests [patch] — todo
- priority: verification; needs: none
- milestone: M-rate-typing.
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `.github/workflows/ci.yml`, `src/rate.rs`, `src/rate/dimension.rs`
- Gap (read at basis): twelve doctests pin an error code (`compile_fail,E0277` or `E0271`: `src/rate/dimension.rs` eleven, `src/rate.rs` one). Stable rustdoc 1.97.0, the pinned toolchain (`rust-toolchain.toml`), does not check the code: a probe doctest `compile_fail,E0277` whose body fails with E0308 passed as `ok` under stable (rustdoc 1.97.0) and failed under `rustdoc +nightly --test` (1.100.0-nightly, 2026-08-26), with and without `-Z unstable-options`, printing "Some expected error codes were not found". CI runs `cargo test --locked --doc --all-features` on stable only (`.github/workflows/ci.yml:53-54`), so a doctest can fail for another reason and stay green. Only the `src/rate/dimension.rs` rejections and the `src/rate.rs:83` doctest are code-pinned; the wrong-order rejection of `mass_action_rate` is the runtime `matches!` doctest (`src/rate.rs:190-200`) plus a `compile_fail` without a code (`src/rate.rs:205`). Whether all twelve pass the nightly check was not run.
- Outcome: CI fails when a pinned code is absent from the compiler's errors for its doctest.
- Acceptance: a CI step runs the crate's doctests under a pinned nightly (`cargo +nightly-<date> test --locked --doc --all-features`, no `-Z` flag: the rustdoc book lists error numbers under "Nightly-gated functionality", features that "just require a nightly build to operate", `src/doc/rustdoc/src/unstable-features.md` in rust-lang/rust) and exits 0 at head; with one doctest's code changed to one its body does not raise (`E0277` to `E0308`) it exits nonzero. The flag-free `rustdoc +nightly --test` behavior is observed; the `cargo` wrapper form is a hypothesis settled by that one run.
- Next step: run the nightly doctest check locally, correct any pin it exposes, then add the step as its own nightly job with the toolchain date pinned in the workflow.

<a id="PROMETHEUS-ADR-0002-SEAL-REWRITE"></a>
## PROMETHEUS-ADR-0002-SEAL-REWRITE — State the seal in ADR 0002 decision 1 [patch] — todo
- priority: verification; needs: none
- milestone: M-rate-typing.
- basis: d190167170c305957c266d4cbdf828eab55c685a
- scope: `docs/adr/0002-rate-constant-typing.md`
- Gap (read at basis): `RateConstantDimension` is sealed (`src/rate/dimension.rs:18-19,144`, commit `ef30785`), but ADR 0002 decision 1 (`docs/adr/0002-rate-constant-typing.md:41-53`) still reads "implemented for the aequitas dimension" with no supertrait; the seal appears only in a ten-line revision note (`:79-88`). A changed decision is rewritten in place with a one-line dated revision note.
- Outcome: decision 1 states that the trait is sealed and why; the seal's revision note is one line naming what, why and the evidence.
- Acceptance: `sed -n '/^1\. /,/^2\. /p' docs/adr/0002-rate-constant-typing.md | grep -c -i seal` prints at least 1 (0 at basis; `seal` occurs only at `:79` and `:88`); `sed -n '/^## Revisions/,/^## Rejected/p' docs/adr/0002-rate-constant-typing.md | grep -c -i '^- 20..-..-..: .*seal.*evidence'` prints 1 (0 at basis, where the entry spans ten lines), a single dated line naming what, why and evidence. The ADR stays within 150 lines and `docs/adr/README.md` still lists it as Accepted.
- Next step: add the supertrait sentence to decision 1, compress the revision note, and rerun the ADR index check.
