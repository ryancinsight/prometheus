//! Mass-action rate-law oracles.
//!
//! Each case is a closed-form mass-action product verified at the value level;
//! the reaction orders are the free parameters a wrong product or a wrong
//! power would expose. The constants are typed: a first-order constant cannot
//! be passed to a second-order product without a deliberate dimension change.

use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime};
use prometheus::rate::{
    FirstOrderConstant, SecondOrderConstant, ThirdOrderConstant, first_order_rate, mass_action_rate,
};

fn assert_close(actual: f64, expected: f64) {
    let tol = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!((actual - expected).abs() <= tol, "{actual} vs {expected}");
}

#[test]
fn zeroth_order_rate_is_the_coefficient() {
    // No participating species: the product is empty, rate = k in mol·m⁻³·s⁻¹.
    let k = ReactionRate::from_base(3.5);
    let rate = mass_action_rate(k, &[]);
    assert_close(*rate.as_base(), 3.5);
}

#[test]
fn first_order_rate_is_linear_and_dimension_derived() {
    // A -> B: rate = k · c_A, k in s⁻¹.
    let k = ReciprocalTime::from_base(0.5);
    let c_a = MolarConcentration::from_base(2.0);

    // `first_order_rate` binds `ReciprocalTime × MolarConcentration` to
    // `ReactionRate`, so this compiles only because the dimensions balance.
    let rate: ReactionRate<f64> = first_order_rate(k, c_a);
    assert_close(*rate.as_base(), 1.0);

    // The general form agrees with the typed form on the same values.
    let general = mass_action_rate(k, &[(1, c_a)]);
    assert_close(*general.as_base(), 1.0);
}

#[test]
fn second_order_self_reaction_is_quadratic() {
    // 2 A -> B: rate = k · c_A², k in m³·mol⁻¹·s⁻¹.
    let k = SecondOrderConstant::from_base(0.25);
    let c_a = MolarConcentration::from_base(3.0);
    let rate: ReactionRate<f64> = mass_action_rate(k, &[(2, c_a)]);
    assert_close(*rate.as_base(), 2.25);
}

#[test]
fn second_order_cross_reaction_is_bilinear() {
    // A + B -> C: rate = k · c_A · c_B.
    let k = SecondOrderConstant::from_base(0.1);
    let c_a = MolarConcentration::from_base(2.0);
    let c_b = MolarConcentration::from_base(5.0);
    let rate: ReactionRate<f64> = mass_action_rate(k, &[(1, c_a), (1, c_b)]);
    assert_close(*rate.as_base(), 1.0);
}

#[test]
fn third_order_rate_rides_a_cube() {
    // 3 A -> B: rate = k · c_A³, exercising integer_power past square level.
    let k = ThirdOrderConstant::from_base(0.5);
    let c_a = MolarConcentration::from_base(2.0);
    let rate: ReactionRate<f64> = mass_action_rate(k, &[(3, c_a)]);
    assert_close(*rate.as_base(), 4.0);
}

#[test]
fn generic_over_scalar_instantiation() {
    // The same law holds at f32; 0.5 · 4² = 8.0 is binary-exact.
    let k: FirstOrderConstant<f32> = ReciprocalTime::from_base(0.5);
    let c = MolarConcentration::from_base(4.0_f32);
    let rate = mass_action_rate(k, &[(2, c)]);
    assert!((*rate.as_base() - 8.0_f32).abs() <= f32::EPSILON);
}
