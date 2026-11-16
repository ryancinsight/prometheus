# Migration

## Rate-constant order check

`mass_action_rate` checks the rate constant against the reaction order
([ADR 0002](adr/0002-rate-constant-typing.md)) and returns
`Result<ReactionRate<T>, RateOrderMismatch>`: handle the error or `?` it. The
coefficient must be a rate-constant quantity (`ReactionRate`,
`FirstOrderConstant`, `SecondOrderConstant`, `ThirdOrderConstant`, or another
dimension `mol^(1-n)·m^(3n-3)·s⁻¹`) whose order `n` equals the sum of the
reactant orders.

```ignore
// Before
let rate = mass_action_rate(k, &[(1, c)]);
// After
let rate = mass_action_rate(k, &[(1, c)])?;
```

`ReactionNetwork::new` takes a `RateConstant<T>` per reaction: convert the typed
quantity with `.into()`; a bare scalar no longer compiles. Match
`InvalidNetwork::NegativeRate` for a negative coefficient (`NonFiniteRate`
covers only `NaN` and infinity) and add `OrderMismatch` to exhaustive matches.

```ignore
// Before
ReactionNetwork::new(nu, vec![(0.04, vec![(0, 1)])])?;
// After
ReactionNetwork::new(
    nu,
    vec![(FirstOrderConstant::from_base(0.04).into(), vec![(0, 1)])],
)?;
```

## Eunomia 0.12 and Leto 0.47

This release changes the nominal scalar contract used by Prometheus public
generic types. Update a consumer as one dependency-cohort change:

1. Require Eunomia 0.12 and Leto 0.47 from the selected stack revisions.
2. Import `RealField` from Eunomia 0.12.
3. Recompile generic bounds that mention `prometheus::StoichiometricMatrix<T>`
   or another Prometheus public type with `T: eunomia::RealField`.
4. Regenerate the standalone lockfile and verify that one Eunomia source and
   one Leto source resolve throughout the consumer graph.

Before:

```ignore
// `RealField` resolves from Eunomia 0.11.
fn client<T: eunomia::RealField>(_: &prometheus::StoichiometricMatrix<T>) {}
```

After:

```rust
use eunomia::RealField; // Eunomia 0.12
use prometheus::StoichiometricMatrix;

fn client<T: RealField>(_: &StoichiometricMatrix<T>) {}
```

Prometheus provides no legacy trait adapter. Updating only Prometheus while a
consumer retains Eunomia 0.11 leaves two distinct `RealField` traits and fails
with `E0277` at the public type bound.
