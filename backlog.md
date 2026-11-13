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

<a id="PROMETHEUS-ORDER-SATURATION"></a>
## PROMETHEUS-ORDER-SATURATION — Remove silent saturation of large reaction orders [patch] — todo
- priority: correctness; needs: none
- basis: 87ecf5b8ba26c13d77affaca028e8a32c87acb34
- scope: `src/rate.rs`, `CHANGELOG.md`
- Defect (read at basis): `mass_action_product` narrows the `u32` order with `i32::try_from(order).unwrap_or(i32::MAX)`
  (`src/rate.rs:125-126`; `Order = u32`, `:23`), so every order above `i32::MAX` yields b^2147483647, not b^order,
  for any base the public `mass_action_rate` accepts. Witnesses (stand-alone rustc 1.97.0 `-O` program, 2026-11-13):
  f64 base 1.0000001 at `u32::MAX`: 1.8366e93 vs 3.3733e186; base -2.0 at order 2^31: -inf vs +inf.
- Outcome: the factor for any `u32` order equals b^order, sign included, for every base, or the order is rejected with a typed error.
- Acceptance: a unit test co-located in `src/rate.rs` (`mass_action_product` is `pub(crate)`), failing at basis, at `f32` and `f64`:
  base -2.0 at order 2^31 gives +inf, -0.5 gives +0.0 (sign bit), -1.0 gives 1.0; at `f64`, base 1.0000001 at `u32::MAX` is within
  relative 4.8e-7 = (n-1)*2^-53 of `exp(n*ln_1p(b-1))` (the bound exceeds 1 at `f32`); the zeroth- to third-order oracles keep their assertions.
- Next step: replace the narrowing with exponentiation by squaring over `u32` in `T` (multiplication-only, keeping
  zero concentration defined), then add these cases.

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
