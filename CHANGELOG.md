# Changelog

All notable changes are documented here, following a keep-a-changelog shape.

## [Unreleased]

### Added

- `rate::RateConstantDimension`: the dimensions `mol^(1-n)·m^(3n-3)·s⁻¹` of a
  mass-action rate constant of overall order `n`, with `ORDER` read from the
  amount exponent. The exponent lies in `-(2³¹-2)` to `2³¹-1`, so the order is
  an `i32`; every other dimension has no implementation. See
  [ADR 0002](docs/adr/0002-rate-constant-typing.md).
- `RateConstant<T>`: a rate constant that keeps its value and the reaction
  order read from its dimension, built from a typed quantity with `.into()`.
  See [ADR 0002](docs/adr/0002-rate-constant-typing.md).

### Changed

- Check the rate-constant dimension against the reaction order in
  `mass_action_rate`. A rate constant's dimension is
  `mol^(1-n)·m^(3n-3)·s⁻¹` for overall order `n`; the function now returns
  `Result<ReactionRate<T>, RateOrderMismatch>`, rejects a coefficient of
  another order, and no longer compiles for a quantity that is no rate
  constant. A rate-constant dimension has an amount exponent from `-(2³¹-2)`
  to `2³¹-1`, so its order is an `i32`. This is a breaking change (0.1.0 to
  0.2.0): the return type changes and the dimension is a new bound. See
  [ADR 0002](docs/adr/0002-rate-constant-typing.md) and the
  [migration guide](docs/migration.md).
- Advance the public provider cohort: Prometheus binds its scalar contract to
  Eunomia 0.12 and its sparse-storage contract to Leto 0.47, so code generic
  over the Eunomia 0.11 `RealField` bound no longer satisfies Prometheus's
  public bounds. See
  [ADR 0001](docs/adr/0001-provider-cohort-contract.md) and the
  [migration guide](docs/migration.md).
- Follow the current provider cohort. The manifest requires `aequitas` 0.3.1,
  `eunomia` 0.12.0, the optional `leto` `>=0.47.1, <0.51.0`, and the optional
  `horae-time` 0.2.2; the lock resolves each at its repository's current
  default head. One git source resolves to one revision, so every edge that
  names a provider repository has to agree on a version or the resolver
  refuses the pair. The public Prometheus surface is otherwise unchanged: no
  type from a moved provider appears in a Prometheus signature.

- The optional time-integration dependency is declared as the registry package
  `horae-time` (the `horae` name belongs to another crates.io owner). Its library
  name stays `horae`, so every `use horae::` path is unchanged.

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