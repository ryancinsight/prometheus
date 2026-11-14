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
/// factors whatever its shape, so, absent overflow and underflow, the relative
/// error is at most `γ(order − 1)` with `γ(m) = m·u / (1 − m·u)` and `u` the
/// unit roundoff.
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
mod tests {
    use super::{Order, mass_action_product};
    use eunomia::RealField;

    /// First order above `i32::MAX`: the least order the former narrowing
    /// clamped, and an even one.
    const FIRST_UNSIGNED_ORDER: Order = 1 << 31;

    /// Relative error bound of `base^u32::MAX` in `f64`.
    ///
    /// Forming the power takes `n − 1` products for `n = u32::MAX`, each
    /// rounding by a factor `1 + δ` with `|δ| ≤ u = 2⁻⁵³`, so the error is at
    /// most `γ(n − 1) = (n − 1)·u / (1 − (n − 1)·u)`. Here
    /// `(n − 1)·u = 4.7683716e-7` and `γ(n − 1) = 4.7683739e-7`.
    ///
    /// The reference is `exp(y)` with `y = n·ln_1p(base − 1)`, and
    /// `|y| ≤ n·(base − 1) = 429.4967` for `base = 1.0000001` because
    /// `ln(1 + x) ≤ x`. Assuming `ln_1p` and `exp` are faithful (error below
    /// one ulp, at most `2u` relative), `y` carries an absolute error of at
    /// most `|y|·(2u + u)`: `ln_1p` plus the rounding of the product by `n`.
    /// That is `1.4305115e-13`, and `exp` adds its own `2u`, so the reference
    /// relative error is `1.4327319e-13`. Combined with `γ(n − 1)` as
    /// `(γ + ε) / (1 − ε)`, the bound is `4.7683753e-7`, rounded up to
    /// `4.8e-7`. At `f32` the same bound is `(n − 1)·2⁻²⁴ ≈ 256`, which bounds
    /// nothing, so only `f64` carries it.
    const RELATIVE_BOUND_AT_U32_MAX: f64 = 4.8e-7;

    fn power<T: RealField>(base: f64, order: Order) -> T {
        mass_action_product(T::ONE, [(T::from_f64(base), order)])
    }

    /// Assert that both shipped scalar types return exactly `expected`, bit
    /// for bit, so the sign of zero and of infinity is checked. `f32` widens
    /// to `f64` exactly. Both are evaluated before the assertion so a failure
    /// reports each type.
    fn assert_exact_at_both(base: f64, order: Order, expected: f64) {
        let single = f64::from(power::<f32>(base, order));
        let double = power::<f64>(base, order);
        assert!(
            single.to_bits() == expected.to_bits() && double.to_bits() == expected.to_bits(),
            "base {base}, order {order}: f32 gave {single}, f64 gave {double}, expected {expected}"
        );
    }

    #[test]
    fn even_order_overflow_is_positive_infinity() {
        assert_exact_at_both(-2.0, FIRST_UNSIGNED_ORDER, f64::INFINITY);
    }

    #[test]
    fn even_order_underflow_is_positive_zero() {
        assert_exact_at_both(-0.5, FIRST_UNSIGNED_ORDER, 0.0);
    }

    #[test]
    fn even_order_of_unit_magnitude_is_one() {
        assert_exact_at_both(-1.0, FIRST_UNSIGNED_ORDER, 1.0);
    }

    #[test]
    fn odd_order_overflow_is_negative_infinity() {
        assert_exact_at_both(-2.0, FIRST_UNSIGNED_ORDER + 1, f64::NEG_INFINITY);
    }

    #[test]
    fn odd_order_underflow_is_negative_zero() {
        assert_exact_at_both(-0.5, FIRST_UNSIGNED_ORDER + 1, -0.0);
    }

    #[test]
    fn odd_order_of_unit_magnitude_is_minus_one() {
        assert_exact_at_both(-1.0, u32::MAX, -1.0);
    }

    #[test]
    fn signed_range_edge_is_continuous() {
        // `i32::MAX` is the last order `powi` takes directly, and is odd.
        assert_exact_at_both(-1.0, u32::MAX >> 1, -1.0);
        assert_exact_at_both(-1.0, (u32::MAX >> 1) + 1, 1.0);
    }

    #[test]
    fn zero_and_infinite_bases_stay_defined() {
        assert_exact_at_both(0.0, 0, 1.0);
        assert_exact_at_both(0.0, u32::MAX, 0.0);
        assert_exact_at_both(f64::INFINITY, FIRST_UNSIGNED_ORDER, f64::INFINITY);
    }

    #[test]
    fn nan_base_stays_nan() {
        assert!(power::<f32>(f64::NAN, FIRST_UNSIGNED_ORDER).is_nan());
        assert!(power::<f64>(f64::NAN, FIRST_UNSIGNED_ORDER).is_nan());
    }

    #[test]
    fn largest_order_matches_the_exponential_form() {
        // Reference `exp(n·ln_1p(base − 1))` shares no code with the product;
        // `base − 1` is exact (Sterbenz) so `ln_1p` sees the true increment.
        let base = 1.000_000_1_f64;
        let order = u32::MAX;
        let expected = (f64::from(order) * (base - 1.0).ln_1p()).exp();
        let actual = power::<f64>(base, order);
        let relative = ((actual - expected) / expected).abs();
        assert!(
            relative <= RELATIVE_BOUND_AT_U32_MAX,
            "relative error {relative} exceeds {RELATIVE_BOUND_AT_U32_MAX}: {actual} vs {expected}"
        );
    }
}
