//! Stoichiometry oracles: `transpose(nu) * M = 0` for a mass-conserving network.

pub mod common;

use eunomia::RealField;
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

#[test]
fn public_matrix_accepts_current_eunomia_bound() {
    fn client<T: RealField>(matrix: &StoichiometricMatrix<T>) -> usize {
        matrix.n_species()
    }

    let matrix = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [])
        .expect("invariant: positive dimensions and an empty sparse payload form a valid matrix");
    assert_eq!(client(&matrix), 2);
}

#[test]
fn misshaped_masses_display_both_lengths() {
    let first = MisshapedMasses {
        n_species: 3,
        supplied: 5,
    };
    let second = MisshapedMasses {
        n_species: 7,
        supplied: 2,
    };
    assert_eq!(
        common::error_text(&first),
        "mass balance received 5 molar masses for 3 species"
    );
    assert_eq!(
        common::error_text(&second),
        "mass balance received 2 molar masses for 7 species"
    );
}

#[test]
fn empty_stoichiometry_displays_its_invariant() {
    assert_eq!(
        common::error_text(&InvalidStoichiometry::Empty),
        "a stoichiometric matrix needs at least one species and one reaction"
    );
}

#[test]
fn out_of_range_stoichiometry_displays_the_faulting_indices() {
    let out_of_range =
        |species, reaction, n_species, n_reactions| InvalidStoichiometry::IndexOutOfRange {
            species,
            reaction,
            n_species,
            n_reactions,
        };
    assert_eq!(
        common::error_text(&out_of_range(Some(4), None, 3, 2)),
        "entry outside the 3-species, 2-reaction matrix: species 4"
    );
    assert_eq!(
        common::error_text(&out_of_range(None, Some(9), 6, 8)),
        "entry outside the 6-species, 8-reaction matrix: reaction 9"
    );
    assert_eq!(
        common::error_text(&out_of_range(Some(11), Some(12), 5, 4)),
        "entry outside the 5-species, 4-reaction matrix: species 11, reaction 12"
    );
}
