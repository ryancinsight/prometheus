# ADR 0001: Advance the public provider cohort

- Status: Accepted
- Date: 2026-11-09
- Item: ATLAS-VERSION-COHERENCE-SWEEP-2026-09-30
- Delivery: [PR 64](https://github.com/ryancinsight/prometheus/pull/64)

## Context

Public Prometheus types, including `StoichiometricMatrix<T>`, bind `T` to
Eunomia's `RealField`. Rust treats a trait from Eunomia 0.11 and the same-named
trait from Eunomia 0.12 as distinct nominal contracts. Advancing Prometheus to
Eunomia 0.12 therefore rejects a generic consumer whose bound still names the
0.11 trait, even when its scalar type implements both versions.

The selected stack cohort also advances Leto from 0.46 to 0.47. Prometheus must
resolve Eunomia and Leto as one coherent provider set because its public sparse
matrix contains Leto storage parameterized by Eunomia scalars.

## Decision

Prometheus advances its public scalar contract to Eunomia 0.12 and its sparse
storage contract to Leto 0.47 in one breaking change. Consumers update their
Eunomia bound and imports to 0.12 and select the Leto 0.47 cohort before using
Prometheus public generic types.

No compatibility trait, forwarding wrapper, or duplicate matrix type remains.
Those alternatives would preserve two nominal scalar contracts and split the
canonical `RealField` seam.

## Consequences

Code generic over Eunomia 0.11 `RealField` no longer satisfies Prometheus's
public type bounds. [The migration guide](../migration.md) gives the complete
consumer change. Consumers already using concrete scalar types continue to use
the same Prometheus API after their dependency graph selects the new cohort.

The native consumer witness rejects the old 0.11 bound with `E0277` and accepts
the 0.12 bound. `tests/stoichiometry.rs` retains the accepted current-bound
case. Cargo SemVer analysis cannot render a Rustdoc verdict because rebuilding
the historical Git baseline resolves the moving Eunomia source at 0.12; Atlas
ADR 0069 records that limitation. Locked compilation of both revisions proves
their selected graphs build, not that their public APIs are compatible.

## Rejected alternatives

- A compatibility shim or parallel legacy type retains the obsolete nominal
  contract and violates the single provider seam.
- Keeping Eunomia 0.11 while advancing Leto postpones the same transition and
  leaves the selected stack cohort incoherent.
- A version-only declaration without consumer migration evidence hides the
  source-level break.

## Overturning evidence

Revisit this decision only if Eunomia supplies a stable cross-version nominal
contract that both provider lines implement without adapters, or Prometheus no
longer exposes Eunomia-bound public types.
