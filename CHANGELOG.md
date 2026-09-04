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