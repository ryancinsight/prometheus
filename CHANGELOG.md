# Changelog

All notable changes are documented here, following a keep-a-changelog shape.

## [Unreleased]

### Added

- Phase 0 core: a `Species` type carrying a strictly positive molar mass (typed
  as `Mass / AmountOfSubstance`, verified by a type-level oracle), and a
  validated `Concentration` boundary that rejects negative, `NaN`, and infinite
  molar concentrations.
- Sparse `StoichiometricMatrix` over Leto COO storage, with the mass-conservation
  oracle νᵀ M = 0 exercised on the water-formation network at `f32` and `f64`.
- Mass-action rate laws with arbitrary integer reaction order, verified
  against zeroth-, first-, second-, and third-order closed forms. Rate
  constants are typed aequitas quantities (`FirstOrderConstant`,
  `SecondOrderConstant`, `ThirdOrderConstant`) rather than bare scalars, and
  `first_order_rate` derives its `ReactionRate` result by aequitas
  multiplication so a wrong-order constant is a compile error. The rate
  coefficient's temperature dependence is Proteus's contract (ADR 0055 R4)
  and is not reimplemented here.
- Net production `omega = nu · r` on the stoichiometric matrix, returning
  typed `ReactionRate` values, with a hand-computed water-formation network
  and the element-conservation identity `sum M_i omega_i = 0` as oracles.
- Reaction enthalpy `q = Σ rⱼ ΔHⱼ` (`heat_release`) as the typed coupling term
  a thermal balance consumes; the `ReactionRate × MolarEnergy ==
  VolumetricPowerDensity` dimension is derived, not asserted.
- Zero-dimensional integration through Horae: `ReactionNetwork` implements
  `horae::ExplicitSystem` computing `dC/dt = ν·r` from its stoichiometric
  matrix and per-reaction mass-action rates, with fixed-step RK4 recovering
  the first-order, second-order (`2A → B`), and reversible-equilibrium
  (`c_B/c_A → K_eq`) closed forms; the refinement oracle recovers RK4's fourth
  order, and the balanced-network invariants (mass conservation and
  non-negativity) hold under integration.