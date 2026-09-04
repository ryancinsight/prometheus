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
    let factor = reactants
        .iter()
        .fold(T::ONE, |acc, (order, concentration)| {
            acc * integer_power(*concentration.as_base(), *order)
        });
    ReactionRate::from_base(*coefficient.as_base() * factor)
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

/// `base ^ exponent` for a non-negative integer exponent, by repeated squaring.
///
/// Uses only multiplication and the multiplicative identity, so it stays valid
/// for every `T: RealField` without assuming a `powi` surface on the trait.
#[inline]
fn integer_power<T: RealField>(base: T, exponent: Order) -> T {
    let mut result = T::ONE;
    let mut square = base;
    let mut remaining = exponent;
    while remaining > 0 {
        if remaining & 1 == 1 {
            result *= square;
        }
        square *= square;
        remaining >>= 1;
    }
    result
}
