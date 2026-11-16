//! Mass-action rate-law oracles and the rate-constant order check.
//!
//! Each closed form is a mass-action product asserted in SI units
//! (`mol·m⁻³·s⁻¹`) at every shipped scalar type. The inputs are dyadic
//! rationals whose products are exact in `f32` and `f64`, so the derived
//! tolerance is zero: a mismatch is a wrong product or a wrong power, never
//! rounding.
//!
//! The rate-constant dimension is `mol^(1-n)·m^(3n-3)·s⁻¹` for overall order
//! `n` (`[k] = [r]/[c]^n`); a coefficient whose order differs from the sum of
//! the reactant orders is rejected with a typed error. A dimension that is no
//! rate constant does not compile (the `compile_fail` doctests of
//! `RateConstantDimension` and `RateConstant`).

use core::any::type_name;
use core::fmt::Debug;

use aequitas::Quantity;
use aequitas::dimension::{BaseSemantics, Dimension};
use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
use eunomia::RealField;
use prometheus::rate::{
    FirstOrderConstant, RateConstant, RateConstantDimension, RateOrderMismatch,
    SecondOrderConstant, ThirdOrderConstant, first_order_rate, mass_action_rate,
};
use typenum::{N1, N3, N6, N2147483647, P1, P2, P9, P2147483647, Prod, Sum, Z0};

/// Order 4: `mol⁻³·m⁹·s⁻¹`.
type FourthOrder = Quantity<f64, Dimension<P9, Z0, N1, Z0, Z0, N3, Z0>>;
/// Order -1: `mol²·m⁻⁶·s⁻¹`, an order no reaction has.
type NegativeOrder = Quantity<f64, Dimension<N6, Z0, N1, Z0, Z0, P2, Z0>>;

fn conc<T: RealField>(value: f64) -> MolarConcentration<T> {
    MolarConcentration::from_base(T::from_f64(value))
}

fn assert_exact<T: RealField + Debug>(rate: ReactionRate<T>, expected: f64) {
    assert_eq!(
        *rate.as_base(),
        T::from_f64(expected),
        "{}",
        type_name::<T>()
    );
}

fn closed_forms<T: RealField + Debug>() {
    // Zeroth order: no participating species, rate = k.
    let zeroth = mass_action_rate(ReactionRate::from_base(T::from_f64(3.5)), &[]);
    assert_exact(zeroth.expect("order 0 constant, no reactants"), 3.5);

    // A -> B: rate = k·c_A, k in s⁻¹; the typed form agrees with the general one.
    let k1: FirstOrderConstant<T> = ReciprocalTime::from_base(T::from_f64(0.5));
    assert_exact(first_order_rate(k1, conc(2.0)), 1.0);
    assert_exact(
        mass_action_rate(k1, &[(1, conc(2.0))]).expect("order 1 constant, order 1 sum"),
        1.0,
    );

    // 2 A -> B: rate = k·c_A², k in m³·mol⁻¹·s⁻¹.
    let k2: SecondOrderConstant<T> = Quantity::from_base(T::from_f64(0.25));
    assert_exact(
        mass_action_rate(k2, &[(2, conc(3.0))]).expect("order 2 constant, order 2 sum"),
        2.25,
    );

    // A + B -> C: orders 1 + 1 sum to 2, rate = k·c_A·c_B.
    let k2: SecondOrderConstant<T> = Quantity::from_base(T::from_f64(0.125));
    assert_exact(
        mass_action_rate(k2, &[(1, conc(2.0)), (1, conc(5.0))]).expect("1 + 1 = 2"),
        1.25,
    );

    // 3 A -> B: rate = k·c_A³, k in m⁶·mol⁻²·s⁻¹.
    let k3: ThirdOrderConstant<T> = Quantity::from_base(T::from_f64(0.5));
    assert_exact(
        mass_action_rate(k3, &[(3, conc(2.0))]).expect("order 3 constant, order 3 sum"),
        4.0,
    );
}

#[test]
fn mass_action_rate_matches_closed_forms_at_every_scalar() {
    closed_forms::<f32>();
    closed_forms::<f64>();
}

/// A mismatch is reported in both directions with both orders.
fn mismatches<T: RealField + Debug>() {
    let c = conc::<T>(2.0);
    let first: FirstOrderConstant<T> = ReciprocalTime::from_base(T::ONE);
    let second: SecondOrderConstant<T> = Quantity::from_base(T::ONE);
    let third: ThirdOrderConstant<T> = Quantity::from_base(T::ONE);
    let zeroth = ReactionRate::from_base(T::ONE);
    let mismatch = |coefficient_order, reactant_order| {
        Some(RateOrderMismatch {
            coefficient_order,
            reactant_order,
        })
    };

    assert_eq!(mass_action_rate(first, &[(2, c)]).err(), mismatch(1, 2));
    assert_eq!(mass_action_rate(second, &[(1, c)]).err(), mismatch(2, 1));
    assert_eq!(mass_action_rate(first, &[]).err(), mismatch(1, 0));
    assert_eq!(mass_action_rate(zeroth, &[(1, c)]).err(), mismatch(0, 1));
    assert_eq!(
        mass_action_rate(third, &[(1, c), (1, c)]).err(),
        mismatch(3, 2)
    );
    // The reactant sum is formed in `u64`: two `u32::MAX` orders do not wrap.
    assert_eq!(
        mass_action_rate(second, &[(u32::MAX, c), (u32::MAX, c)]).err(),
        mismatch(2, 2 * u64::from(u32::MAX))
    );
}

#[test]
fn rate_constant_order_mismatch_is_a_typed_error_at_every_scalar() {
    mismatches::<f32>();
    mismatches::<f64>();
}

#[test]
fn rate_constant_order_is_read_from_the_dimension() {
    let order = |constant: RateConstant<f64>| constant.order();
    assert_eq!(order(ReactionRate::from_base(1.0).into()), 0);
    assert_eq!(order(FirstOrderConstant::from_base(1.0).into()), 1);
    assert_eq!(order(SecondOrderConstant::from_base(1.0).into()), 2);
    assert_eq!(order(ThirdOrderConstant::from_base(1.0).into()), 3);

    // The value is kept in the base unit of its dimension, bit for bit.
    let second = RateConstant::from(SecondOrderConstant::from_base(0.25_f64));
    assert_eq!(second.base().to_bits(), 0.25_f64.to_bits());

    assert_eq!(order(FourthOrder::from_base(1.0).into()), 4);

    // A negative order is no reaction's order, and is still read.
    assert_eq!(order(NegativeOrder::from_base(1.0).into()), -1);
}

/// The amount exponents at both ends of the admitted range, `-(2³¹-2)` and
/// `2³¹-1`, with the length exponent `-3·N` each fixes.
type LowestAmount = Sum<N2147483647, P1>;
type HighestAmount = P2147483647;
type AtLowest = Dimension<Prod<LowestAmount, N3>, Z0, N1, Z0, Z0, LowestAmount, Z0, BaseSemantics>;
type AtHighest =
    Dimension<Prod<HighestAmount, N3>, Z0, N1, Z0, Z0, HighestAmount, Z0, BaseSemantics>;

#[test]
fn rate_constant_order_is_exact_at_the_amount_exponent_range_edges() {
    // `1 - N` is `i32::MAX` at the lowest exponent and `i32::MIN + 2` at the
    // highest; `i32` holds both, so no bit of either is dropped.
    assert_eq!(<AtLowest as RateConstantDimension>::ORDER, i32::MAX);
    assert_eq!(<AtHighest as RateConstantDimension>::ORDER, i32::MIN + 2);

    // The greatest order drives a reaction of that order end to end.
    let k: Quantity<f64, AtLowest> = Quantity::from_base(2.0);
    let rate = mass_action_rate(k, &[(2_147_483_647, conc(1.0))]).expect("order i32::MAX");
    assert_exact(rate, 2.0);
}

#[test]
fn rate_order_mismatch_displays_both_orders_and_is_an_error() {
    let mismatch = RateOrderMismatch {
        coefficient_order: 1,
        reactant_order: 2,
    };
    let error: &dyn core::error::Error = &mismatch;
    assert_eq!(
        error.to_string(),
        "rate constant of order 1 cannot drive a reaction of order 2"
    );
}
