# ADR 0003: Prometheus release scope for 1.0

- Status: Accepted
- Date: 2026-11-15
- Item: PROMETHEUS-RELEASE-LADDER
- Delivery: PR #79, merged under the `judge-required` label; no judge
  verdict is recorded for it (open: `PROMETHEUS-RELEASE-LADDER`)

## Context

Atlas ADR 0058 charters Prometheus as the owner of species mass balance and
reaction kinetics, entering at Phase 0: homogeneous reaction networks and
their zero-dimensional integration. ADR 0055 places Prometheus on the balance
axis (conserved quantity: species mass) with the boundary rules R1–R7; the
README scope section (Phase 0 homogeneous networks; reactive transport,
combustion closure, heterogeneous/surface reactions, plasma chemistry,
electrochemistry, phase equilibrium deferred to later phases, none
scaffolded) and the Phase 0 oracle table in ADR 0058 are the scope inputs.
ADR 0001 advanced the public provider cohort to Eunomia 0.12 / Leto 0.47.

The backlog carries five open items and no milestone ladder: the
rate-constant dimension defect (major, pre-1.0), two oracle/test hardenings,
one changelog-drift repair, and this ladder item. No source determines the
1.0 API-freeze line for a 0.1.0 crate, so that line is a veto point below.

## Decision

Adopt a five-milestone ladder to 1.0. The ladder opens `backlog.md`; every
open item carries exactly one `milestone:` assignment.

- **M-rate-typing** (0.2.0): the rate-constant dimension is enforced.
  Exit: a coefficient whose dimension does not match the reaction's total
  order is rejected at compile time or as a typed error, for
  `mass_action_rate` and `ReactionNetwork::evaluate`
  (typed coefficient delivered and the dimension trait sealed,
  [ADR 0002](0002-rate-constant-typing.md), the latter by PR #92);
  runnable check
  `cargo nextest run --locked --all-features -- rate` plus the wrong-order
  rejection test (a `compile_fail` doctest or `matches!` on the error
  variant) — the check fails on a build where a first-order constant with
  order-2 reactants compiles and returns a rate.
- **M-oracle-hardening** (0.2.0): the network oracle and power edges are
  pinned. Exit: narrowing delivered (PR #80: the dense reference raises each
  base by repetition); the `-0.0`/`-infinity`/subnormal edge tests pass and
  the `-0.0`/`-infinity` cases kill the `base.abs()` mutant
  (`negative_zero_base_keeps_the_parity_sign`,
  `negative_infinite_base_keeps_the_parity_sign` and
  `subnormal_base_underflows_to_a_parity_signed_zero`, PR #106).
- **M-board-hygiene** (0.2.0): the changelog states the shipped cohort.
  Exit: every provider version named in `CHANGELOG.md` equals the
  `Cargo.toml` requirement at the head — the grep comparison fails on drift.
- **M-1.0** (1.0.0): stable kinetics release (**authority-gated**). Exit:
  `cargo semver-checks`, publish dry-run, `cargo doc`, the full
  nextest/doctest suite green at all features, and zero open correctness
  items. **Veto point:** no source fixes the 1.0 API-freeze contents; the
  freeze list is set under release authority, and this milestone never
  forces `publish = true` without it.
- **M-ecosystem**: consumer-owned migrations against the stable surface
  (kwavers species-kinetics consolidation, CFDrs efficiency-metric
  replacement). Exit: consumer receipts on their own boards; does not gate
  1.0.

### Capability disposition

| Area | Disposition |
| --- | --- |
| `species` (identity, molar mass) | Delivered; exit: `tests/species.rs` green |
| `concentration` (validated `MolarConcentration` boundary) | Delivered; exit: `concentration_accepts_non_negative_and_rejects_invalid` green |
| `stoichiometry` (sparse matrix over Leto) | Delivered; exit: `tests/stoichiometry.rs` green |
| `rate` (mass-action laws, Arrhenius via `proteus::TemperatureResponse`) | In scope; exit: M-rate-typing |
| `network` (assembly, net production, analytic Jacobian) | Delivered modulo M-rate-typing; exit: `tests/integration.rs` green |
| `enthalpy` (heat release per reaction) | Delivered; exit: `tests/enthalpy.rs` green |
| Horae-driven 0-D integration (batch/CSTR, stiff Robertson path) | Delivered; exit: `first/second_order_decay_matches_closed_form`, `reversible_equilibrium_reaches_keq`, `backward_euler_validates_robertson_stiff_trajectory` green |
| Reactive transport, combustion closure, heterogeneous/surface, plasma, electrochemistry, phase equilibrium, transport properties | Non-goals (ADR 0058; README later phases, none scaffolded) |

### Veto points

1. The 1.0 API-freeze contents (no source determines them; release
   authority sets them).

The typed-coefficient design is not a veto point: ADR 0002 is Accepted and
implemented (PRs #84-#87).

## Rejected alternatives

- Freezing 1.0 on the untyped coefficient: the defect ADR 0002 addresses let
  a dimensionally wrong network compile, which a 1.0 stability promise cannot
  cover.
- Carrying the later-phase capabilities as stretch goals inside 1.0: no
  charter authorizes them and none is scaffolded; they stay non-goals until
  a Phase 1+ charter with its own driver exists.

## Overturning evidence

A consumer failure traceable to a frozen API, or a charter authorizing a
later phase, reopens the affected milestone. The closed-form oracles in
ADR 0058 are the tripwire: any of them failing at head overturns the
corresponding Delivered row.

## Revisions

- 2026-11-16: ADR 0002 and the typed rate constant landed (PRs #84-#87), so
  the M-rate-typing exit cites ADR 0002 and names the open seal item
  instead of calling the dimension rule delivered (a downstream impl of the
  unsealed trait defeats it); the typed-coefficient veto point is dropped
  because ADR 0002 is Accepted; the delivery line cites PR #79 and the
  missing judge verdict; no exit check changed.
- 2026-11-16: PR #92 sealed `RateConstantDimension`, so the M-rate-typing
  exit parenthetical states the seal as delivered instead of naming its open
  item; no exit check changed, and whether the seal's doctest joins the exit
  stays with `PROMETHEUS-RELEASE-LADDER`.
- 2026-11-24: the M-board-hygiene exit no longer cites a board item id in parentheses, since an exit check names a runnable check, not an item, and PR #103 (`a7904b9`) delivered the cited item; the condition text is unchanged, so no exit check is weakened.
