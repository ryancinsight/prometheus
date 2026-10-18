//! Mass-action reaction rates.
//!
//! A mass-action rate is `r = k · ∏ᵢ cᵢ^(νᵢ)`, taken over the reactants only.
//! The rate constant `k` carries its own dimension — `s⁻¹` first order,
//! `m³·mol⁻¹·s⁻¹` second order, and so on — so it is a *typed* aequitas
//! quantity, never a bare scalar: a temperature or a concentration cannot be
//! passed where a rate constant is meant (atlas ADR 0058; the anti-aliasing
//! rule in the stack standards).
//!
//! The temperature dependence that produces `k(T)` is Proteus's contract
//! (ADR 0055 R4) and is never reimplemented here.

use aequitas::Quantity;
use aequitas::dimension::DivideDimension;
use aequitas::systems::si::dimensions::{
    MolarConcentration as MolarConcentrationDim, ReciprocalTime as ReciprocalTimeDim,
};
use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
use eunomia::RealField;

#[cfg(test)]
mod tests;

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
/// coefficient's dimension must match the reaction's total order — the aliases
/// above make that correct by construction — because the scalar product is
/// assembled in base units and wrapped as a reaction rate.
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
/// let rate: ReactionRate<f64> = mass_action_rate(k, &[(1, c)]);
/// ```
pub fn mass_action_rate<T, KD>(
    coefficient: Quantity<T, KD>,
    reactants: &[(Order, MolarConcentration<T>)],
) -> ReactionRate<T>
where
    T: RealField,
{
    let factor = mass_action_product(
        *coefficient.as_base(),
        reactants
            .iter()
            .map(|(order, concentration)| (*concentration.as_base(), *order)),
    );
    ReactionRate::from_base(factor)
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

/// Retain the complete unsigned order without changing the ordinary path.
/// For n > `i32::MAX`, n = 2q + r with q <= `i32::MAX` and r in {0,1}.
/// The provider evaluates q in native precision, then p^2 * base^r gives
/// base^n. This needs at most 31 provider iterations and two extra products,
/// no allocation, scalar widening, or loop proportional to the order.
#[inline]
fn reaction_power<T: RealField>(base: T, order: Order) -> T {
    if let Ok(exponent) = i32::try_from(order) {
        return T::powi(base, exponent);
    }
    high_order_power(base, order)
}

#[cold]
#[inline(never)]
fn high_order_power<T: RealField>(base: T, order: Order) -> T {
    let half = i32::try_from(order >> 1).expect("half a u32 fits a nonnegative i32");
    let power = T::powi(base, half);
    let squared = power * power;
    if order & 1 == 0 {
        squared
    } else {
        squared * base
    }
}
