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
//! rate constant does not compile (the `compile_fail` doctest of
//! `mass_action_rate`).

use core::any::type_name;
use core::fmt::Debug;

use aequitas::Quantity;
use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
use eunomia::RealField;
use prometheus::rate::{
    FirstOrderConstant, RateConstantDimension, RateOrderMismatch, SecondOrderConstant,
    ThirdOrderConstant, first_order_rate, mass_action_rate,
};

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
    fn order<D: RateConstantDimension>(_: Quantity<f64, D>) -> i32 {
        D::ORDER
    }
    assert_eq!(order(ReactionRate::from_base(1.0)), 0);
    assert_eq!(order(FirstOrderConstant::from_base(1.0)), 1);
    assert_eq!(order(SecondOrderConstant::from_base(1.0)), 2);
    assert_eq!(order(ThirdOrderConstant::from_base(1.0)), 3);
}
