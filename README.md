# Prometheus

Species mass balance for the [Atlas](https://github.com/ryancinsight/atlas)
stack. Prometheus owns the transformation side of chemical kinetics —
species identity, stoichiometry, mass-action rate laws, net production, and
reaction enthalpy — and its zero-dimensional integration. It owns neither the
transport of species (the balance domain carries that discretization) nor the
rate-coefficient temperature response (Proteus owns `k(T)`; atlas ADR 0055
and ADR 0058).

The registry name is `prometheus-kinetics` because `prometheus` on crates.io
is the metrics client; the import path is restored to `prometheus` via
`[lib] name`.

## Scope

Phase 0 is homogeneous reaction networks and their zero-dimensional
integration. Reactive-transport discretization, combustion closure,
heterogeneous and surface reactions, plasma chemistry, electrochemistry, and
phase equilibrium are later phases, and none is scaffolded.

## Verification

Every claim carries an analytical oracle — a closed form, a conservation law,
or a published benchmark — because new construction has no reference
implementation to difference against. The Phase 0 oracles are enumerated in
ADR 0058, including the stiff Robertson benchmark and the non-negativity and
mass-conservation invariants.

## License

MIT OR Apache-2.0.