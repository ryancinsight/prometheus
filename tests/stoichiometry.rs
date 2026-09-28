//! Stoichiometry oracles: `transpose(nu) * M = 0` for a mass-conserving network.

pub mod common;

use prometheus::{InvalidStoichiometry, MisshapedMasses, MolarMass, StoichiometricMatrix};

#[test]
fn water_formation_conserves_mass_exactly_f64() {
    let (nu, masses) = common::water_formation::<f64>();
    let residuals = nu.mass_residuals(&masses).expect("matching species count");
    assert_eq!(residuals.len(), 1);
    // -2*2 + -1*32 + 2*18 = 0; integers are binary64-exact, so the bit
    // pattern is the structural zero rather than a rounded near-zero.
    assert_eq!(residuals[0].to_bits(), 0.0f64.to_bits());
    assert!(
        nu.conserves_mass(&masses, 0.0)
            .expect("matching species count")
    );
}

#[test]
fn water_formation_conserves_mass_exactly_f32() {
    let (nu, masses) = common::water_formation::<f32>();
    let residuals = nu.mass_residuals(&masses).expect("matching species count");
    assert_eq!(residuals[0].to_bits(), 0.0f32.to_bits());
    assert!(
        nu.conserves_mass(&masses, 0.0)
            .expect("matching species count")
    );
}

#[test]
fn unbalanced_network_reports_nonzero_residual() {
    // H2 → H2O without oxygen: mass cannot balance.
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
        .expect("in-range");
    let masses = [MolarMass::from_base(2.0), MolarMass::from_base(18.0)];
    let residual = nu.mass_residuals(&masses).expect("matching")[0];
    assert_eq!(residual.to_bits(), 16.0f64.to_bits());
    assert!(!nu.conserves_mass(&masses, 0.0).expect("matching"));
}

#[test]
fn rejects_empty_and_out_of_range_entries() {
    assert!(matches!(
        StoichiometricMatrix::<f64>::try_from_entries(0, 1, []),
        Err(InvalidStoichiometry::Empty)
    ));
    assert!(matches!(
        StoichiometricMatrix::<f64>::try_from_entries(1, 0, []),
        Err(InvalidStoichiometry::Empty)
    ));
    assert!(matches!(
        StoichiometricMatrix::<f64>::try_from_entries(1, 1, [(1, 0, 1.0)]),
        Err(InvalidStoichiometry::IndexOutOfRange {
            species: Some(1),
            reaction: None,
            n_species: 1,
            n_reactions: 1,
        })
    ));
}

#[test]
fn rejects_misshaped_molar_mass_vector() {
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
        .expect("in-range");
    assert_eq!(
        nu.mass_residuals(&[MolarMass::from_base(1.0)]),
        Err(MisshapedMasses {
            n_species: 2,
            supplied: 1,
        })
    );
}
