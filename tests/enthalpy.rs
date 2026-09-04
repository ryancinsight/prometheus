//! Reaction-enthalpy oracles: a hand-computed heat release and the derived
//! dimension `ReactionRate × MolarEnergy == VolumetricPowerDensity`.

use aequitas::systems::si::quantities::{MolarEnergy, ReactionRate, VolumetricPowerDensity};
use prometheus::{MisshapedEnthalpies, heat_release};

fn assert_close(actual: f64, expected: f64) {
    let tol = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!((actual - expected).abs() <= tol, "{actual} vs {expected}");
}

#[test]
fn heat_release_is_rate_times_enthalpy() {
    // A single exothermic reaction: ΔH = -100 J/mol at r = 2 mol·m⁻³·s⁻¹.
    let delta_h = MolarEnergy::from_base(-100.0);
    let rate = ReactionRate::from_base(2.0);

    // The binding is the type-level oracle: it compiles only because
    // ReactionRate × MolarEnergy is exactly VolumetricPowerDensity.
    let q: VolumetricPowerDensity<f64> =
        heat_release(&[rate], &[delta_h]).expect("parallel lengths");

    assert_close(*q.as_base(), -200.0);
}

#[test]
fn heat_release_sums_across_reactions() {
    let rates = [ReactionRate::from_base(1.0), ReactionRate::from_base(3.0)];
    let enthalpies = [MolarEnergy::from_base(10.0), MolarEnergy::from_base(-20.0)];
    let q = heat_release(&rates, &enthalpies).expect("parallel lengths");
    // 1·10 + 3·(-20) = 10 - 60 = -50 W/m³.
    assert_close(*q.as_base(), -50.0);
}

#[test]
fn zero_enthalpy_cancels_the_heat_release() {
    let rates = [ReactionRate::from_base(5.0), ReactionRate::from_base(5.0)];
    let enthalpies = [MolarEnergy::from_base(7.0), MolarEnergy::from_base(-7.0)];
    let q = heat_release(&rates, &enthalpies).expect("parallel lengths");
    assert_close(*q.as_base(), 0.0);
}

#[test]
fn rejects_misshaped_enthalpies() {
    let rates = [ReactionRate::from_base(1.0)];
    assert_eq!(
        heat_release(&rates, &[]),
        Err(MisshapedEnthalpies {
            n_rates: 1,
            n_enthalpies: 0,
        })
    );
}
