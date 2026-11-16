# ADR 0002: Check the rate-constant dimension against the reaction order

- Status: Accepted
- Date: 2026-11-16
- Item: PROMETHEUS-RATE-CONSTANT-UNITS
- Delivery: [PR 82](https://github.com/ryancinsight/prometheus/pull/82)
  (decisions 1 and 2); decisions 3 and 4 follow in the item's later PRs,
  listed by `git log --grep='^Item: PROMETHEUS-RATE-CONSTANT-UNITS'`.

## Context

`mass_action_rate<T, KD>` accepted a `Quantity<T, KD>` of any dimension and
wrapped the scalar product as a `ReactionRate`, so a first-order constant with
order-2 reactants compiled and returned a rate. `ReactionNetwork` stores each
coefficient as a bare `T`. The reactant orders are runtime `u32` values in a
slice and in the network's reaction list, so the check cannot be a type-level
relation between the constant's dimension and those orders.

## Derivation

The mass-action law is `r = k·∏ᵢ cᵢ^νᵢ`. With `[r] = mol·m⁻³·s⁻¹` and
`[c] = mol·m⁻³`, the overall order `n = Σᵢ νᵢ` gives

`[k] = [r]/[c]ⁿ = mol^(1-n) · m^(3n-3) · s⁻¹`.

The SI exponents are amount `1-n`, length `3n-3 = -3·(1-n)`, time `-1`, and
zero for mass, current, temperature and luminosity. `n = 0` is
`mol·m⁻³·s⁻¹`, `n = 1` is `s⁻¹`, `n = 2` is `m³·mol⁻¹·s⁻¹`, `n = 3` is
`m⁶·mol⁻²·s⁻¹`. The amount exponent fixes `n` and the length exponent follows,
so a dimension with another length exponent is no rate constant.

Source read this session: Wikipedia, "Rate equation", sections "Power laws"
(rate `v = k[A]ˣ[B]ʸ…`, overall order the sum of the exponents) and "Summary
for reaction orders 0, 1, 2, and n" (units `M^(1-n)·s⁻¹`, `M` in mol·dm⁻³,
which agrees with the formula at `n = 0, 1, 2`). That page has no third-order
row; `n = 3` follows from the formula. The IUPAC Gold Book entries for rate
constant and order of reaction (`R05156`, `O04321`) returned HTTP 403 to
automated clients and were not read.

## Decision

1. `trait RateConstantDimension { const ORDER: i32 }` is implemented for the
   aequitas dimension with amount exponent `N`, length exponent `N·(-3)`, time
   exponent `-1`, no other base exponent, and base semantics; `ORDER = 1 - N`.
   Every other dimension has no implementation and is rejected at compile
   time. Exponents are read through `typenum`, a new direct dependency already
   in the lock as an `aequitas` dependency. `ORDER` is an `i32`, so the amount
   exponent is bounded at the type level by `N > -(2³¹-1)` and `N ≤ 2³¹-1`
   (`IsGreater`, `IsLessOrEqual`), which is where `typenum`'s `I32`, built by
   shifting, is exact and `1 - N` does not overflow. A dimension outside the
   range has no implementation. The orders a rate-constant dimension denotes
   are therefore `-(2³¹-2)` to `2³¹-1`; a reaction of order `2³¹` or more has
   no rate-constant dimension, and `mass_action_rate` returns
   `RateOrderMismatch` for it.
2. `mass_action_rate` takes `D: RateConstantDimension` and returns
   `Result<ReactionRate<T>, RateOrderMismatch>` carrying the coefficient order
   and the reactant order sum. The sum is formed in `u64`, saturating, so no
   input wraps; a negative `ORDER` equals no sum.
3. `ReactionNetwork::new` takes an order-erased `RateConstant<T>` built from
   a typed quantity and returns `InvalidNetwork::OrderMismatch` when its order
   differs from the reactant sum. A mismatched network is unconstructible, so
   `evaluate` and `jacobian` run only on consistent reactions.
4. `InvalidNetwork::NonFiniteRate` means `NaN` or infinite; a new
   `NegativeRate` means a negative finite coefficient.

The change is a public break under the pre-1.0 leftmost-nonzero rule, 0.1.0 to
0.2.0 ([major]); `docs/migration.md` gives the caller change. The return type
changes and a generic bound is added, and cargo-semver-checks 0.51.0 has no
lint for either, so it reports no break; the class is set by inspection.

## Revisions

- 2026-11-16: the first draft read `ORDER` as `1 - N::I32` with no bound on
  `N`. A judge built an amount exponent of `-(2³²+2)`, whose `I32` is `-2`,
  and `ORDER` read 3; the exponent range above replaces that read. The
  `compile_fail` doctests on `RateConstantDimension` pin every constrained
  axis, each range edge and that exponent.

## Rejected alternatives

- Type-level reaction orders: a network is a runtime list of reactions of
  different orders, built from runtime data, and one `Vec` cannot hold
  per-order types.
- Constants for orders 0 to 3 only: narrows the shipped `u32` order range far
  below the `2³¹` of the chosen design.
- A wider `ORDER` (`i64`) admitting every `u32` order: no consumer has a
  reaction of order near `2³¹`, and `i32` matches the exponent type of
  `powi`. The same type-level bound applies at the wider range. Reopen if a
  consumer needs an order above `i32::MAX`.
- A caller-declared order tag, `RateConstant::new(value, order)`: it checks
  nothing about the dimension, which is the defect.
- A `debug_assert!` or panic on mismatch: the input is caller data and
  library code returns typed errors.
- An exponent-reading trait exported by aequitas: a cross-repository change.
  Reopen if aequitas exports one; the trait here then delegates to it.

## Overturning evidence

A consumer whose reaction orders are fractional or fitted: the rate law then
has non-integer exponents and `Order = u32` itself is wrong, so a new decision
replaces this one.
