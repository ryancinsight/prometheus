//! Mass-action rate-law oracles.
//!
//! Each case is a closed-form mass-action product verified at the value level;
//! the reaction orders are the free parameters a wrong product or a wrong
//! power would expose.

use prometheus::rate::mass_action_rate;

fn assert_close(actual: f64, expected: f64) {
    let tol = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!((actual - expected).abs() <= tol, "{actual} vs {expected}");
}

#[test]
fn zeroth_order_rate_is_the_coefficient() {
    // No participating species: the product is empty, rate = k.
    assert_close(mass_action_rate(3.5_f64, &[]), 3.5);
}

#[test]
fn first_order_rate_is_linear_in_concentration() {
    // A -> B: rate = k · c_A.
    let k = 0.5_f64;
    let c_a = 2.0;
    assert_close(mass_action_rate(k, &[(1, c_a)]), k * c_a);
}

#[test]
fn second_order_self_reaction_is_quadratic() {
    // 2 A -> B: rate = k · c_A^2.
    let k = 0.25_f64;
    let c_a = 3.0;
    assert_close(mass_action_rate(k, &[(2, c_a)]), k * c_a * c_a);
}

#[test]
fn second_order_cross_reaction_is_bilinear() {
    // A + B -> C: rate = k · c_A · c_B.
    let k = 0.1_f64;
    let c_a = 2.0;
    let c_b = 5.0;
    assert_close(mass_action_rate(k, &[(1, c_a), (1, c_b)]), k * c_a * c_b);
}

#[test]
fn third_order_rate_rides_a_cube() {
    // 3 A -> B: rate = k · c_A^3, exercising integer_power past square level.
    let k = 0.5_f64;
    let c_a = 2.0;
    assert_close(mass_action_rate(k, &[(3, c_a)]), k * c_a * c_a * c_a);
}

#[test]
fn generic_over_scalar_instantiation() {
    // The same law holds at f32; 0.5 · 4² = 8.0 is binary-exact, so a strict
    // tolerance is the whole point: it binds the f32 path without a ULP.
    let rate = mass_action_rate(0.5_f32, &[(2, 4.0_f32)]);
    assert!((rate - 8.0_f32).abs() <= f32::EPSILON);
}
