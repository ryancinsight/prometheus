# Prometheus Backlog

Open items in priority order. The meta item
`ATLAS-PROMETHEUS-PROMOTION-2026-09-03` (atlas `backlog.md#prometheus-promotion`)
owns registration and first publication and is not copied here. Items carry no
`milestone:` until `PROMETHEUS-RELEASE-LADDER` lands a ladder. Evidence below
was read at `basis:` unless stated otherwise; no test was executed.

<a id="PROMETHEUS-RATE-CONSTANT-UNITS"></a>
## PROMETHEUS-RATE-CONSTANT-UNITS — Enforce the rate-constant dimension against reaction order [major] — todo
- priority: correctness; needs: none
- basis: 87ecf5b8ba26c13d77affaca028e8a32c87acb34
- scope: `src/rate.rs`, `src/network.rs`, `src/lib.rs`, `tests/rate.rs`, `tests/integration.rs`, `tests/production.rs`, `README.md`, `CHANGELOG.md`, `docs/adr/`
- Defect (read at basis): `mass_action_rate<T, KD>` (`src/rate.rs:68-72`) accepts `Quantity<T, KD>` for any `KD` and wraps the scalar product with `ReactionRate::from_base` (`src/rate.rs:81`), so the reactant orders in the slice are not related to `KD`; a first-order constant with order-2 reactants compiles and returns a `ReactionRate`. `ReactionNetwork` stores each coefficient as bare `T` (`src/network.rs:68`, `src/network.rs:84-87`), documented as order-dependent (`src/network.rs:75-78`). Origin: atlas `gap_audit.md`, finding of 2026-10-26.
- Outcome: a coefficient whose dimension does not match the reaction's total order is rejected at compile time or returns a typed error, for the free function and the network.
- Acceptance: for at least two reaction orders (for example 1 and 2) and every shipped scalar type (`f32`, `f64`), the closed-form rate is asserted in SI units (`mol·m⁻³·s⁻¹`) with a derived tolerance, and a wrong-order coefficient is rejected (`compile_fail` doctest or `matches!` on the error variant, never bare `is_err()`); the same holds through `ReactionNetwork::evaluate`.
- API: public break, pre-1.0 leftmost-nonzero rule (0.1.0 to 0.2.0); SemVer gate [major] classification and a migration note in `docs/migration.md`.
- Adjacent (read at basis): `ReactionNetwork::new` rejects a negative coefficient under the variant `NonFiniteRate` (`src/network.rs:99-100`) while its Rustdoc lists only non-finite coefficients (`src/network.rs:82-83`); settle with the new error shape.
- Next step: draft `docs/adr/0002-rate-constant-typing.md` choosing the typed-coefficient design (alternatives and rejected options recorded there; this item does not pick one), then implement under that ADR with every in-repo caller migrated in the same change.

<a id="PROMETHEUS-RELEASE-LADDER"></a>
## PROMETHEUS-RELEASE-LADDER — Draft the release scope and milestone ladder [patch] — todo
- priority: verification; needs: none
- basis: 87ecf5b8ba26c13d77affaca028e8a32c87acb34
- scope: `docs/adr/` (release-scope ADR; ADR 0002 is reserved by `PROMETHEUS-RATE-CONSTANT-UNITS`), `docs/adr/README.md`, `backlog.md`
- Outcome: release-scope ADR and milestone ladder per the stack roadmap rule (sprint: roadmap). The ladder opens this board; every item then carries `milestone:`.
- Acceptance: the ADR records each capability area of the crate (`species`, `concentration`, `stoichiometry`, `rate`, `network`, `enthalpy`, the Horae-driven zero-dimensional integration) as delivered, in scope or a non-goal; each milestone exit check is a runnable check that fails without its feature; an independent judge verdict maps every capability area and open feature item to a check or non-goal.
- Inputs: `README.md` scope section (reactive transport, combustion closure, heterogeneous and surface reactions, plasma chemistry, electrochemistry and phase equilibrium named later phases), atlas ADRs 0055 and 0058.
- Next step: draft the ADR from those inputs with a recommended scope, name any scope no source determines as a veto point, and file the ladder under its own `judge-required` PR.

<a id="PROMETHEUS-CHANGELOG-DRIFT"></a>
## PROMETHEUS-CHANGELOG-DRIFT — Bring the Unreleased changelog in line with the manifest [patch] — todo
- priority: verification; needs: none
- basis: 87ecf5b8ba26c13d77affaca028e8a32c87acb34
- scope: `CHANGELOG.md`, `docs/migration.md`
- Defect (read at basis): `CHANGELOG.md` `## [Unreleased]` states `eunomia` moves to 0.10.0 and the optional `leto` to 0.46.4; `Cargo.toml` requires `eunomia` 0.12.0 and `leto` `>=0.47.1, <0.49.0`, and `docs/migration.md` and `README.md` describe the Eunomia 0.12 and Leto 0.47 cohort. The Unreleased section does not mention the 0.12 cohort break that ADR 0001 records.
- Outcome: the Unreleased section states only what the manifest and ADR 0001 state, in the repository's changelog convention.
- Acceptance: each provider version named in `CHANGELOG.md` equals the `Cargo.toml` requirement at the head (`git grep -n 'eunomia\|leto' CHANGELOG.md Cargo.toml`), and the ADR 0001 break is listed under a `Changed` entry with its migration link.
- Next step: edit the entry, then run the docs and link checks of the committed gate.

<a id="PROMETHEUS-ORDER-ORACLE-NARROWING"></a>
## PROMETHEUS-ORDER-ORACLE-NARROWING — Remove the order narrowing from the network test oracle [patch] — todo
- priority: verification; needs: none
- basis: 769faa950130e2e07ea86e6a395edd220dde8234
- scope: `tests/integration.rs`
- Defect (read at basis): `sparse_large_network_evaluate_matches_dense_reference` forms its reference with `powi(i32::try_from(*order).unwrap_or(i32::MAX))` (`tests/integration.rs:269`), the clamp that `reaction_power` in `src/rate.rs` replaced. An order above `i32::MAX` would be compared against a saturated power, so the oracle would reject the correct `reaction_power` value.
- Outcome: the dense reference raises each base to its `u32` order without narrowing.
- Acceptance: `git grep -n 'unwrap_or(i32::MAX)' -- src tests` is empty, and the test still passes for its orders 2 and 1 against the production assembler.
- Next step: replace the `powi` call with a repeated product over `0..*order`, sharing no code with `reaction_power`.

<a id="PROMETHEUS-ORDER-SIGNED-EDGE-TESTS"></a>
## PROMETHEUS-ORDER-SIGNED-EDGE-TESTS — Pin signed zero, infinity and subnormal bases above order 2^31 [patch] — todo
- priority: verification; needs: none
- basis: 769faa950130e2e07ea86e6a395edd220dde8234
- scope: `src/rate.rs` (module `tests`)
- Gap (read at basis): the `reaction_power` tests cover bases -2, -0.5, -1, +0, +infinity and NaN at orders above `i32::MAX`. No test pins a `-0.0` base, a `-infinity` base at an odd order, or a subnormal base (positive and negative, `f64` and `f32` minimum subnormal) above `2^31`.
- Outcome: each case asserts its IEEE result bit for bit through `assert_exact_at_both`: `-0.0` gives `-0.0` at an odd order and `+0.0` at an even one, `-infinity` gives `-infinity` at an odd order and `+infinity` at an even one, and a subnormal base underflows to a zero carrying the parity sign.
- Acceptance: the new tests pass at head and fail under a mutant that applies `base.abs()` only when the base is zero or infinite, which the existing tests (`+0.0` and `+infinity` only) do not kill.
- Next step: append the cases to `src/rate.rs` `tests`, then run the mutants and record which test kills each.
