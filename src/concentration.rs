//! Validated molar-concentration boundary.

use aequitas::systems::si::quantities::MolarConcentration;
use eunomia::RealField;

/// A molar concentration guaranteed non-negative and finite.
///
/// This is the boundary a species state crosses everywhere it enters
/// Prometheus: a transport field, an initial condition, or an integrator step
/// hand back a concentration, and a negative or `NaN` value — the signature of
/// a stiff-integrator failure — is rejected here rather than allowed to
/// propagate into a rate law and manufacture a plausible wrong answer.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Concentration<T> {
    value: MolarConcentration<T>,
}

/// A concentration was negative, `NaN`, or infinite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidConcentration;

impl<T: RealField> Concentration<T> {
    /// Construct a concentration, rejecting any non-finite or negative value.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidConcentration`] for a negative, `NaN`, or infinite
    /// value.
    pub fn new(value: MolarConcentration<T>) -> Result<Self, InvalidConcentration> {
        // `is_finite` rejects `NaN` and infinity; the ordering rejects any
        // negative value. Zero is the empty-species state and is admitted.
        let inner = *value.as_base();
        if !inner.is_finite() || inner < T::ZERO {
            return Err(InvalidConcentration);
        }
        Ok(Self { value })
    }

    /// The validated concentration, in mol/m^3.
    #[must_use]
    pub const fn get(&self) -> MolarConcentration<T> {
        self.value
    }
}
