//! Shared fixtures for the prometheus-kinetics integration oracles.
//!
//! Every file in `tests/` compiles as its own crate, so the shared fixtures
//! live in one module included with `pub mod common;`. The `pub` path keeps a
//! fixture reachable from the crate root of a file that uses only the other
//! one, which the dead-code pass would otherwise flag on the unused half.

use eunomia::RealField;
use prometheus::{MolarMass, Species, StoichiometricMatrix};

/// Assert two `f64` values agree within eight ULPs of the larger magnitude.
///
/// `0.018 kg/mol` and other such constants are not binary-exact, so a derived
/// value can differ from its decimal expectation by at most one ULP; a bound
/// of eight is an order of magnitude of headroom that still catches any
/// dimension or plumbing error (which would be off by many digits, not a ULP).
///
/// # Panics
///
/// Panics when the two values differ by more than the computed tolerance.
pub fn assert_close(actual: f64, expected: f64) {
    let tolerance = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual} differs from expected {expected} by more than {tolerance}"
    );
}

/// 2 H₂ + O₂ → 2 H₂O with molar masses that are exact in binary floating
/// point (2, 32, 18), so a mass residual is a structural zero rather than a
/// rounded one.
///
/// # Panics
///
/// Panics only if the fixture's own fixed constants were rejected, which they
/// are not.
#[must_use]
pub fn water_formation<T: RealField>() -> (StoichiometricMatrix<T>, [MolarMass<T>; 3]) {
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
