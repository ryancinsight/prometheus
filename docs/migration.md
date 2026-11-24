# Migration

## Rate-constant order check

`mass_action_rate` checks the rate constant against the reaction order
([ADR 0002](adr/0002-rate-constant-typing.md)) and returns
`Result<ReactionRate<T>, RateOrderMismatch>`: handle the error or `?` it. The
coefficient must be a rate-constant quantity (`ReactionRate`,
`FirstOrderConstant`, `SecondOrderConstant`, `ThirdOrderConstant`, or another
dimension `mol^(1-n)·m^(3n-3)·s⁻¹`) whose order `n` equals the sum of the
reactant orders. A generic caller adds the bound `D: RateConstantDimension` to
the dimension parameter of the quantity it passes; the amount exponent of such
a dimension lies in `-(2³¹-2)` to `2³¹-1`.
`RateConstantDimension` is sealed: a type of your own cannot implement it, so
a coefficient reaches `mass_action_rate` only as an aequitas quantity of one of
those dimensions.

```ignore
// Before
let rate = mass_action_rate(k, &[(1, c)]);
// After
let rate = mass_action_rate(k, &[(1, c)])?;
```

`ReactionNetwork::new` takes a `RateConstant<T>` per reaction: convert the typed
quantity with `.into()`; a bare scalar no longer compiles. Match
`InvalidNetwork::NegativeRate` for a negative coefficient (`NonFiniteRate`
covers only `NaN` and infinity) and `OrderMismatch` for a coefficient of
another order. `InvalidNetwork` is `#[non_exhaustive]`, so a `match` on it
needs a wildcard arm.

```ignore
// Before
ReactionNetwork::new(nu, vec![(0.04, vec![(0, 1)])])?;
// After
ReactionNetwork::new(
    nu,
    vec![(FirstOrderConstant::from_base(0.04).into(), vec![(0, 1)])],
)?;
```

## Provider cohort: Eunomia 0.12, Aequitas 0.5 and Leto 0.52

Prometheus public generic types bind their scalar to Eunomia's `RealField`,
and its public signatures use Aequitas quantity types. Update a consumer as one
dependency-cohort change. The manifest requires `aequitas` 0.5.0, `eunomia`
0.12.1, the optional `leto` 0.52.0 and the optional `horae-time` 0.4.0.

1. Require the provider lines the manifest names: Eunomia 0.12 and Aequitas
   0.5. Quantity types such as `ReactionRate<T>`, `MolarEnergy<T>` and
   `VolumetricPowerDensity<T>` appear in public signatures, and an Aequitas
   line other than 0.5 supplies different nominal types, as an Eunomia line
   other than 0.12 supplies a different `RealField`. A consumer that names
   `horae-time` directly requires Horae Time 0.4, the line behind the
   `ExplicitSystem` and `ImplicitSystem` implementations of
   `ReactionNetwork<T>`. The sparse storage behind `StoichiometricMatrix<T>`
   is Leto 0.52; no Leto type appears in a public signature, so a consumer
   that names `leto` selects the same line as cohort policy.
2. Import `RealField` from Eunomia 0.12.
3. Recompile generic bounds that mention `prometheus::StoichiometricMatrix<T>`
   or another Prometheus public type with `T: eunomia::RealField`.
4. Regenerate the standalone lockfile and verify that one Eunomia, one
   Aequitas, one Horae Time and one Leto source each resolve throughout the
   consumer graph.
   The Prometheus lockfile takes Aequitas, Eunomia and Horae Time from the
   crates.io registry and Leto from one revision of its git repository.

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
