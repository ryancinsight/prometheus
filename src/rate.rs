//! Mass-action reaction rates.
//!
//! A mass-action rate is `r = k · ∏ᵢ cᵢ^(νᵢ)`, taken over the reactants only,
//! where `νᵢ` is the reaction order of species `i` and `k` is the rate
//! coefficient **evaluated at the current temperature**. Prometheus owns the
//! product and the orders; it does not own how `k` varies with temperature —
//! that is Proteus's contract (atlas ADR 0055 R4), and it is never
//! reimplemented here.

use eunomia::RealField;

/// A reaction order: the non-negative integer exponent on one species
/// concentration in the mass-action product.
pub type Order = u32;

/// Compute a mass-action reaction rate over canonical SI base-unit scalars.
///
/// `rate = k · ∏ᵢ cᵢ^(νᵢ)`, where each `(νᵢ, cᵢ)` term is a reaction order
/// paired with that species' molar concentration in mol·m⁻³, and `k` is the
/// rate coefficient. A species absent from the reaction has order zero and is
/// simply omitted. The returned value is a production rate in mol·m⁻³·s⁻¹.
///
/// The rate coefficient's units depend on the total order — `s⁻¹` for first
/// order, `m³·mol⁻¹·s⁻¹` for second order — so a *dimension-carrying* wrapper
/// must select the coefficient type from the order; the scalar core here keeps
/// the units as the caller's contract and the typed boundary thin.
///
/// # Examples
///
/// ```
/// use prometheus::rate::mass_action_rate;
///
/// // First order: A -> B, rate = k·c_A.
/// let rate = mass_action_rate(0.5, &[(1, 2.0)]);
/// assert_eq!(rate, 1.0);
/// ```
pub fn mass_action_rate<T: RealField>(coefficient: T, terms: &[(Order, T)]) -> T {
    coefficient
        * terms.iter().fold(T::ONE, |acc, (order, concentration)| {
            acc * integer_power(*concentration, *order)
        })
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
