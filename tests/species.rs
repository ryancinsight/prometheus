//! The Species and Concentration boundary oracles.
//!
//! The first ensures the molar-mass dimension is correct by construction: the
//! binding `let mass: Mass<f64>` only compiles if `MolarMass` is exactly
//! `Mass / AmountOfSubstance`, so the type-level law is the oracle and the
//! numeric assertion is a value-semantic check on top of it.

use aequitas::systems::si::quantities::{AmountOfSubstance, Mass, MolarConcentration};
use prometheus::{Concentration, InvalidConcentration, MolarMass, NonPositiveMolarMass, Species};

/// Assert two `f64` values agree within eight ULPs of the larger magnitude.
///
/// `0.018 kg/mol` is not binary-exact, so a product off it can differ from its
/// decimal literal by at most one ULP; a bound of eight is an order of
/// magnitude of headroom that still catches any dimension or plumbing error
/// (which would be off by many digits, not by a ULP).
fn assert_close(actual: f64, expected: f64) {
    let tol = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= tol,
        "actual {actual} differs from expected {expected} by more than {tol}"
    );
}

#[test]
fn molar_mass_times_amount_is_mass() {
    // Water: 18 g/mol = 0.018 kg/mol in the SI base unit.
    let water = Species::<f64>::new("water", MolarMass::from_base(0.018))
        .expect("0.018 kg/mol is a valid molar mass");
    let amount = AmountOfSubstance::from_base(2.0); // 2 mol

    // Type-level oracle: this binding compiles only when `MolarMass` is the
    // quotient `Mass / AmountOfSubstance`.
    let mass: Mass<f64> = water.molar_mass() * amount;

    assert_close(*mass.as_base(), 0.036);
}

#[test]
fn species_rejects_non_positive_molar_mass() {
    assert_eq!(
        Species::<f64>::new("zero", MolarMass::from_base(0.0)),
        Err(NonPositiveMolarMass)
    );
    assert_eq!(
        Species::<f64>::new("negative", MolarMass::from_base(-0.001)),
        Err(NonPositiveMolarMass)
    );
    assert_eq!(
        Species::<f64>::new("nan", MolarMass::from_base(f64::NAN)),
        Err(NonPositiveMolarMass)
    );
    assert_eq!(
        Species::<f64>::new("inf", MolarMass::from_base(f64::INFINITY)),
        Err(NonPositiveMolarMass)
    );
}

#[test]
fn concentration_accepts_non_negative_and_rejects_invalid() {
    let zero = Concentration::<f64>::new(MolarConcentration::from_base(0.0))
        .expect("zero concentration is the empty-species state");
    assert_close(*zero.get().as_base(), 0.0);

    let positive = Concentration::<f64>::new(MolarConcentration::from_base(1.5))
        .expect("positive concentration is valid");
    assert_close(*positive.get().as_base(), 1.5);

    assert_eq!(
        Concentration::<f64>::new(MolarConcentration::from_base(-0.1)),
        Err(InvalidConcentration)
    );
    assert_eq!(
        Concentration::<f64>::new(MolarConcentration::from_base(f64::NAN)),
        Err(InvalidConcentration)
    );
    assert_eq!(
        Concentration::<f64>::new(MolarConcentration::from_base(f64::INFINITY)),
        Err(InvalidConcentration)
    );
}
