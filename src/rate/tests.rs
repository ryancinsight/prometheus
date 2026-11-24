use super::{Order, accumulate_order, mass_action_product};
use eunomia::RealField;

/// First order above `i32::MAX`: the least order the former narrowing
/// clamped, and an even one.
const FIRST_UNSIGNED_ORDER: Order = 1 << 31;

/// Relative error bound of `base^u32::MAX` in `f64`.
///
/// Let `n = u32::MAX` and `u = 2⁻⁵³`. Copying the intermediate results that
/// the power shares unfolds it into a product tree of `n` factors with
/// `n − 1` rounded products, each a factor `1 + δ` with `|δ| ≤ u`. So the
/// computed power `c` and the exact power `t` satisfy
/// `|c − t| ≤ γ(n − 1)·|t|` with `γ(m) = m·u / (1 − m·u)`, defined here
/// because `(n − 1)·u < 1`. Numerically `(n − 1)·u = 4.7683716e-7` and
/// `γ(n − 1) = 4.7683739e-7`.
///
/// The test compares `c` with the reference `r = exp(y)`, where
/// `y = n·ln_1p(base − 1)`, through `|(c − r) / r|`. Because
/// `ln(1 + x) ≤ x`, `|y| ≤ n·(base − 1) = 429.4967` for `base = 1.0000001`.
/// Assume `ln_1p` and `exp` each err by at most `2u` relative (faithful
/// rounding). Then `y` carries an absolute error of at most
/// `|y|·(2u + u)`: `ln_1p` plus the rounding of the product by `n`. That is
/// `1.4305115e-13`, and `exp` adds its own `2u`, so `|r − t| ≤ ε·|t|` with
/// `ε = 1.4327320e-13` (rounded up from `1.43273192e-13`). It follows that
/// `|c − r| ≤ (γ + ε)·|t|` and `|r| ≥ (1 − ε)·|t|`, so
/// `|c − r| / |r| ≤ (γ + ε) / (1 − ε) = 4.7683753e-7`, rounded up to
/// `4.8e-7`. The subtraction `c − r` is exact (Sterbenz), and the rounded
/// quotient adds a relative `u`, which the rounding up of the displayed
/// value absorbs.
///
/// No guarantee backs the faithful-rounding assumption: the standard
/// library documents `exp` and `ln_1p` under "Unspecified precision"
/// (`std::primitive::f64`, Rust 1.97.0). The constant tolerates a relative
/// error `a·u` in each of them for any `a` up to about `6.6e4`, which is
/// the margin the assumption can lose. At `f32`,
/// `(n − 1)·u = (n − 1)·2⁻²⁴ ≈ 256 > 1`, so `γ(n − 1)` is undefined and
/// nothing is bounded; only `f64` carries it.
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
fn negative_zero_base_keeps_the_parity_sign() {
    // IEEE 754 multiplication: the sign of a product is the exclusive or
    // of its factors' signs and a zero magnitude stays zero, so
    // `(-0.0)^order` is `-0.0` for odd `order` and `+0.0` for even.
    assert_exact_at_both(-0.0, FIRST_UNSIGNED_ORDER, 0.0);
    assert_exact_at_both(-0.0, FIRST_UNSIGNED_ORDER + 1, -0.0);
    assert_exact_at_both(-0.0, u32::MAX, -0.0);
}

#[test]
fn negative_infinite_base_keeps_the_parity_sign() {
    // The same sign rule with an infinite magnitude, which no product of
    // infinities reduces: `(-∞)^order` is `-∞` for odd `order` and `+∞`
    // for even.
    assert_exact_at_both(f64::NEG_INFINITY, FIRST_UNSIGNED_ORDER, f64::INFINITY);
    assert_exact_at_both(
        f64::NEG_INFINITY,
        FIRST_UNSIGNED_ORDER + 1,
        f64::NEG_INFINITY,
    );
    assert_exact_at_both(f64::NEG_INFINITY, u32::MAX, f64::NEG_INFINITY);
}

#[test]
fn subnormal_base_underflows_to_a_parity_signed_zero() {
    // The least positive subnormal of each type, `2⁻¹⁰⁷⁴` and `2⁻¹⁴⁹`,
    // raised to an order of at least `2³¹` has magnitude below
    // `2^(-149·2³¹)`, under the least subnormal of either type, so the
    // product rounds to zero and keeps the sign `(-1)^order`. `f32` reads
    // `2⁻¹⁰⁷⁴` as a zero of the same sign.
    for least_subnormal in [f64::from_bits(1), f64::from(f32::from_bits(1))] {
        assert_exact_at_both(least_subnormal, FIRST_UNSIGNED_ORDER, 0.0);
        assert_exact_at_both(least_subnormal, FIRST_UNSIGNED_ORDER + 1, 0.0);
        assert_exact_at_both(-least_subnormal, FIRST_UNSIGNED_ORDER, 0.0);
        assert_exact_at_both(-least_subnormal, FIRST_UNSIGNED_ORDER + 1, -0.0);
        assert_exact_at_both(-least_subnormal, u32::MAX, -0.0);
    }
}

#[test]
fn order_sum_saturates_at_the_largest_total() {
    let largest = u64::MAX;
    let widest = u64::from(u32::MAX);
    // `(2³² + 1)·(2³² − 1) = 2⁶⁴ − 1`: `2³² + 1` orders of `u32::MAX`
    // sum exactly to `largest`; one more order 3 makes the exact sum
    // `2⁶⁴ + 2`, which wraps to 2 (a declared order 2 accepts it) and
    // saturates to `largest`; from `largest − 1` it wraps to 1.
    assert_eq!(accumulate_order(largest, 3), largest);
    assert_eq!(accumulate_order(largest - 1, 3), largest);
    assert_eq!(accumulate_order(largest, u32::MAX), largest);
    // An exact fit does not saturate, and one below it stays exact.
    assert_eq!(accumulate_order(largest - widest, u32::MAX), largest);
    assert_eq!(
        accumulate_order(largest - widest - 1, u32::MAX),
        largest - 1
    );
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
