//! Net-production oracles: hand-computed networks and element conservation.
//!
//! The net production is `omega = nu · r`, so its correctness is fixed by a
//! hand-computed network (each species' production stated from the
//! stoichiometry) and by the conservation identity `sum M_i omega_i = 0`,
//! which holds for a mass-balanced network regardless of the chosen rate.

use aequitas::systems::si::quantities::ReactionRate;
use eunomia::RealField;
use prometheus::{MisshapedRates, MolarMass, Species, StoichiometricMatrix};

/// 2 H₂ + O₂ → 2 H₂O with molar masses 2, 32, 18 (binary-exact, the same
/// convention the stoichiometry oracles use).
fn water_formation<T: RealField>() -> (StoichiometricMatrix<T>, [MolarMass<T>; 3]) {
    let h2 =
        Species::new("H2", MolarMass::from_base(T::from_f64(2.0))).expect("positive molar mass");
    let o2 =
        Species::new("O2", MolarMass::from_base(T::from_f64(32.0))).expect("positive molar mass");
    let h2o =
        Species::new("H2O", MolarMass::from_base(T::from_f64(18.0))).expect("positive molar mass");

    let matrix = StoichiometricMatrix::try_from_entries(
        3,
        1,
        [
            (0, 0, T::from_f64(-2.0)), // 2 H2 consumed
            (1, 0, T::from_f64(-1.0)), // 1 O2 consumed
            (2, 0, T::from_f64(2.0)),  // 2 H2O produced
        ],
    )
    .expect("in-range stoichiometric entries");

    (matrix, [h2.molar_mass(), o2.molar_mass(), h2o.molar_mass()])
}

fn assert_close(actual: f64, expected: f64) {
    let tol = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!((actual - expected).abs() <= tol, "{actual} vs {expected}");
}

#[test]
fn net_production_matches_hand_computed_network() {
    let (nu, _) = water_formation::<f64>();
    let rate = ReactionRate::from_base(3.0); // reaction proceeds at 3 mol·m⁻³·s⁻¹

    let omega: Vec<ReactionRate<f64>> = nu.net_production(&[rate]).expect("one reaction column");

    assert_eq!(omega.len(), 3);
    assert_close(*omega[0].as_base(), -6.0); // H2: -2 · 3
    assert_close(*omega[1].as_base(), -3.0); // O2: -1 · 3
    assert_close(*omega[2].as_base(), 6.0); // H2O: +2 · 3
}

#[test]
fn net_production_conserves_total_mass() {
    let (nu, masses) = water_formation::<f64>();
    let rate = ReactionRate::from_base(3.0);

    let omega = nu.net_production(&[rate]).expect("one reaction column");

    // sum_i M_i · omega_i = rate · (sum_i M_i · nu_i1), and the parenthesized
    // term is the mass residual: -2·2 - 1·32 + 2·18 = 0.
    let total: f64 = omega
        .iter()
        .zip(&masses)
        .map(|(production, mass)| *production.as_base() * *mass.as_base())
        .sum();
    assert_close(total, 0.0);
}

#[test]
fn net_production_is_generic_over_scalar() {
    let (nu, _) = water_formation::<f32>();
    let rate = ReactionRate::from_base(2.0_f32);
    let omega = nu.net_production(&[rate]).expect("one reaction column");
    // -2 · 2 = -4 is binary-exact.
    assert!((*omega[0].as_base() + 4.0_f32).abs() <= f32::EPSILON);
}

#[test]
fn rejects_misshaped_rate_vector() {
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
        .expect("in-range");
    assert_eq!(
        nu.net_production(&[]),
        Err(MisshapedRates {
            n_reactions: 1,
            supplied: 0,
        })
    );
    assert_eq!(
        nu.net_production(&[ReactionRate::from_base(1.0), ReactionRate::from_base(2.0)]),
        Err(MisshapedRates {
            n_reactions: 1,
            supplied: 2,
        })
    );
}
