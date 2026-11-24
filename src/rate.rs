//! Mass-action reaction rates.
//!
//! A mass-action rate is `r = k · ∏ᵢ cᵢ^(νᵢ)`, taken over the reactants only.
//! With `r` in `mol·m⁻³·s⁻¹` and each `cᵢ` in `mol·m⁻³`, the overall order
//! `n = Σᵢ νᵢ` fixes the rate-constant dimension `[k] = [r]/[c]ⁿ =
//! mol¹⁻ⁿ·m³ⁿ⁻³·s⁻¹`: `s⁻¹` first order, `m³·mol⁻¹·s⁻¹` second order, and so
//! on. `k` is therefore a *typed* aequitas quantity, never a bare scalar, and
//! its dimension is checked against the reaction's total order
//! ([ADR 0002]; atlas ADR 0058): a temperature cannot be passed where a rate
//! constant is meant, and a first-order constant cannot drive a second-order
//! reaction.
//!
//! The temperature dependence that produces `k(T)` is Proteus's contract
//! (ADR 0055 R4) and is never reimplemented here.
//!
//! [ADR 0002]: https://github.com/ryancinsight/prometheus/blob/main/docs/adr/0002-rate-constant-typing.md

use aequitas::Quantity;
use aequitas::dimension::DivideDimension;
use aequitas::systems::si::dimensions::{
    MolarConcentration as MolarConcentrationDim, ReciprocalTime as ReciprocalTimeDim,
};
use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
use eunomia::RealField;

mod dimension;

pub use dimension::RateConstantDimension;

/// A reaction order: the non-negative integer exponent on one species
/// concentration in the mass-action product.
pub type Order = u32;

/// Second-order rate-constant dimension: `ReactionRate / MolarConcentration²`,
/// composed as `(ReactionRate / MolarConcentration) / MolarConcentration`
/// (the intermediate is `ReciprocalTime`), which needs only the dimension
/// algebra aequitas already exports.
pub type SecondOrderConstantDimension =
    <ReciprocalTimeDim as DivideDimension<MolarConcentrationDim>>::Output;

/// Third-order rate-constant dimension: `ReactionRate / MolarConcentration³`.
pub type ThirdOrderConstantDimension =
    <SecondOrderConstantDimension as DivideDimension<MolarConcentrationDim>>::Output;

/// First-order rate constant, in s⁻¹.
pub type FirstOrderConstant<T> = ReciprocalTime<T>;
/// Second-order rate constant, in m³·mol⁻¹·s⁻¹.
pub type SecondOrderConstant<T> = Quantity<T, SecondOrderConstantDimension>;
/// Third-order rate constant, in m⁶·mol⁻²·s⁻¹.
pub type ThirdOrderConstant<T> = Quantity<T, ThirdOrderConstantDimension>;

/// A rate constant was dimensioned for an overall order that differs from the
/// sum of the reactant orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateOrderMismatch {
    /// Overall order the coefficient's dimension belongs to.
    pub coefficient_order: i32,
    /// Sum of the reactant orders, saturating at `u64::MAX`.
    pub reactant_order: u64,
}

impl core::fmt::Display for RateOrderMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "rate constant of order {} cannot drive a reaction of order {}",
            self.coefficient_order, self.reactant_order
        )
    }
}

impl core::error::Error for RateOrderMismatch {}

/// A rate constant whose reaction order was read from its dimension.
///
/// Built from a typed [`Quantity`] whose dimension implements
/// [`RateConstantDimension`], it erases the order-dependent dimension into a
/// runtime order so that constants of different orders can share one
/// container, such as the reaction list of a network. A dimension outside the
/// range
/// [`RateConstantDimension`] admits does not convert:
///
/// ```compile_fail,E0271
/// # use aequitas::Quantity;
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstant;
/// # use typenum::{N1, N3, Negate, PInt, Prod, Shleft, Sum, U1, U2, U32, Z0};
/// type Amount = Negate<PInt<Sum<Shleft<U1, U32>, U2>>>;
/// type Huge = Quantity<f64, Dimension<Prod<Amount, N3>, Z0, N1, Z0, Z0, Amount, Z0, BaseSemantics>>;
/// let _ = RateConstant::from(Huge::from_base(1.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RateConstant<T> {
    coefficient: T,
    order: i32,
}

impl<T> RateConstant<T> {
    /// Overall reaction order this constant is dimensioned for.
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }
}

impl<T: Copy> RateConstant<T> {
    /// Scalar value in the base SI unit of its order-dependent dimension.
    ///
    /// # Examples
    ///
    /// ```
    /// use aequitas::systems::si::quantities::ReciprocalTime;
    /// use prometheus::rate::RateConstant;
    ///
    /// let k = RateConstant::from(ReciprocalTime::from_base(0.5_f64));
    /// assert_eq!(k.order(), 1);
    /// assert_eq!(k.base().to_bits(), 0.5_f64.to_bits());
    /// ```
    #[must_use]
    pub const fn base(&self) -> T {
        self.coefficient
    }
}

impl<T, D: RateConstantDimension> From<Quantity<T, D>> for RateConstant<T> {
    fn from(coefficient: Quantity<T, D>) -> Self {
        Self {
            coefficient: coefficient.into_base(),
            order: D::ORDER,
        }
    }
}

/// Add one reactant order to a running sum, saturating at `u64::MAX`.
///
/// Its own function because the sum saturates only past `2³²` orders, too
/// many to iterate in a test.
#[inline]
fn accumulate_order(total: u64, order: Order) -> u64 {
    total.saturating_add(u64::from(order))
}

/// Compare a declared overall order with the sum of the reactant orders.
///
/// The sum is formed in `u64`, saturating, so no input wraps; a negative
/// declared order equals no sum.
pub(crate) fn check_order(
    declared: i32,
    reactant_orders: impl IntoIterator<Item = Order>,
) -> Result<(), RateOrderMismatch> {
    let reactant_order = reactant_orders.into_iter().fold(0_u64, accumulate_order);
    if u64::try_from(declared).is_ok_and(|declared| declared == reactant_order) {
        Ok(())
    } else {
        Err(RateOrderMismatch {
            coefficient_order: declared,
            reactant_order,
        })
    }
}

/// Compute a mass-action reaction rate: `r = k · ∏ᵢ cᵢ^(νᵢ)`.
///
/// `coefficient` is the already-evaluated rate constant, typed with its
/// order-dependent dimension (`FirstOrderConstant`, `SecondOrderConstant`,
/// `ThirdOrderConstant`, or `ReactionRate` itself for zero order). Each
/// `(νᵢ, cᵢ)` term is a reaction order paired with that species' molar
/// concentration. The result is a [`ReactionRate`], the net production in
/// mol·m⁻³·s⁻¹.
///
/// A species absent from the reaction has order zero and is omitted. The
/// coefficient's dimension must be that of the reaction's total order
/// `Σ νᵢ`: a dimension that is no rate constant does not compile, and one of
/// another order returns an error.
///
/// # Errors
///
/// Returns [`RateOrderMismatch`] when the order of `coefficient`'s dimension
/// differs from the sum of the reactant orders.
///
/// # Examples
///
/// ```
/// use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
/// use prometheus::rate::mass_action_rate;
///
/// // First order: A -> B, rate = k·c_A with k in s⁻¹.
/// let k = ReciprocalTime::from_base(0.5);
/// let c = MolarConcentration::from_base(2.0);
/// let rate: ReactionRate<f64> = mass_action_rate(k, &[(1, c)])?;
/// assert_eq!(*rate.as_base(), 1.0);
/// # Ok::<(), prometheus::rate::RateOrderMismatch>(())
/// ```
///
/// A first-order constant cannot drive a second-order reaction:
///
/// ```
/// use aequitas::systems::si::quantities::{MolarConcentration, ReciprocalTime};
/// use prometheus::rate::{RateOrderMismatch, mass_action_rate};
///
/// let k = ReciprocalTime::from_base(0.5);
/// let c = MolarConcentration::from_base(2.0);
/// assert!(matches!(
///     mass_action_rate(k, &[(2, c)]),
///     Err(RateOrderMismatch { coefficient_order: 1, reactant_order: 2 }),
/// ));
/// ```
///
/// A quantity that is no rate constant does not compile, where the
/// first-order constant above does:
///
/// ```compile_fail
/// use aequitas::systems::si::quantities::{MolarConcentration, Time};
/// use prometheus::rate::mass_action_rate;
///
/// let not_a_constant = Time::from_base(0.5);
/// let c = MolarConcentration::from_base(2.0);
/// let _ = mass_action_rate(not_a_constant, &[(1, c)]);
/// ```
pub fn mass_action_rate<T, D>(
    coefficient: Quantity<T, D>,
    reactants: &[(Order, MolarConcentration<T>)],
) -> Result<ReactionRate<T>, RateOrderMismatch>
where
    T: RealField,
    D: RateConstantDimension,
{
    check_order(D::ORDER, reactants.iter().map(|(order, _)| *order))?;
    let factor = mass_action_product(
        *coefficient.as_base(),
        reactants
            .iter()
            .map(|(order, concentration)| (*concentration.as_base(), *order)),
    );
    Ok(ReactionRate::from_base(factor))
}

/// First-order mass-action rate: `r = k·c`, with the result dimension derived
/// by aequitas multiplication rather than asserted.
///
/// `ReciprocalTime × MolarConcentration` is exactly `ReactionRate`, so this
/// binding compiles only when both arguments carry those dimensions — a
/// temperature or a wrong-order constant is a compile error, the typed
/// counterpart of the molar-mass oracle.
///
/// # Examples
///
/// ```
/// use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
/// use prometheus::rate::first_order_rate;
///
/// let k = ReciprocalTime::from_base(0.5);
/// let c = MolarConcentration::from_base(2.0);
/// let rate: ReactionRate<f64> = first_order_rate(k, c);
/// ```
pub fn first_order_rate<T: RealField>(
    coefficient: ReciprocalTime<T>,
    concentration: MolarConcentration<T>,
) -> ReactionRate<T> {
    coefficient * concentration
}

/// The mass-action product `k · ∏ᵢ cᵢ^νᵢ`: a rate coefficient scaled by each
/// reactant base raised to its reaction order.
///
/// Each term is a `(base, order)` pair, where the base is a molar
/// concentration or a component of the integrated state vector, so the same
/// product serves the typed [`mass_action_rate`], the reaction network's
/// derivative, and its analytic Jacobian (which re-forms the product with one
/// exponent decremented). Integer exponentiation is multiplication-only, so
/// the product stays defined at zero concentration and the Jacobian never
/// divides.
///
/// Every `u32` order is honoured exactly: the factor is `base^order`, sign
/// included, never a power of a clamped exponent.
#[inline]
pub(crate) fn mass_action_product<T, I>(coefficient: T, terms: I) -> T
where
    T: RealField,
    I: IntoIterator<Item = (T, Order)>,
{
    terms.into_iter().fold(coefficient, |acc, (base, order)| {
        acc * reaction_power(base, order)
    })
}

/// `base^order` for the full `u32` order range, by multiplication only.
///
/// [`eunomia::FloatElement::powi`], a supertrait method of [`RealField`],
/// takes an `i32`, which cannot hold an order above
/// `i32::MAX`. For such an order, `order = 2·half + parity` with
/// `half ≤ i32::MAX`, so the factor is `(base^half)² · base^parity`; the
/// recursion therefore ends after one halving. Every step is a product in `T`,
/// so a zero base stays defined and the parity of a negative base is kept.
///
/// A chain of products forming `base^order` carries `order − 1` rounding
/// factors whatever its shape, so, absent overflow and underflow and when
/// `(order − 1)·u < 1`, the relative error is at most `γ(order − 1)` with
/// `γ(m) = m·u / (1 − m·u)` and `u` the unit roundoff. `γ(m)` is undefined at
/// `m·u = 1` and negative above it, so no bound exists for `m·u ≥ 1`: at
/// `f32`, `u = 2⁻²⁴` excludes every order above `2²⁴`.
#[inline]
fn reaction_power<T: RealField>(base: T, order: Order) -> T {
    if let Ok(exponent) = i32::try_from(order) {
        T::powi(base, exponent)
    } else {
        let half = reaction_power(base, order >> 1);
        let square = half * half;
        if order & 1 == 0 {
            square
        } else {
            square * base
        }
    }
}

#[cfg(test)]
mod tests;
