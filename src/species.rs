//! Chemical species identity and molar mass.

use core::fmt;

use aequitas::Quantity;
use aequitas::dimension::DivideDimension;
use aequitas::systems::si::dimensions::{AmountOfSubstance as AmountDim, Mass as MassDim};
use eunomia::RealField;

/// Molar mass dimension: mass per amount of substance (kg/mol in SI base).
///
/// Derived by dividing [`MassDim`] by [`AmountDim`] at the type level, so the
/// dimension is correct by construction rather than hand-written. Multiplying
/// a [`MolarMass`] by an amount of substance yields a [`Mass`](aequitas::systems::si::quantities::Mass),
/// which the test `molar_mass_times_amount_is_mass` asserts as its type-level
/// oracle.
pub type MolarMassDimension = <MassDim as DivideDimension<AmountDim>>::Output;

/// Molar mass quantity, in the canonical SI base unit (kg/mol).
pub type MolarMass<T> = Quantity<T, MolarMassDimension>;

/// A chemical species: a name and a strictly positive molar mass.
///
/// The molar mass is the single physical constant a species carries in Phase 0;
/// kinetic parameters (rate coefficients, formation enthalpy) belong to the
/// reaction and are supplied by Proteus, never stored here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Species<T> {
    name: &'static str,
    molar_mass: MolarMass<T>,
}

/// A species was constructed with a molar mass that is not strictly positive.
///
/// Zero and negative molar masses are physically meaningless, and a `NaN`
/// value would poison every derived quantity, so all three are rejected at the
/// construction boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonPositiveMolarMass;

impl<T: RealField> Species<T> {
    /// Construct a species, rejecting a non-positive or non-finite molar mass.
    ///
    /// # Errors
    ///
    /// Returns [`NonPositiveMolarMass`] when `molar_mass` is zero, negative, or
    /// `NaN`.
    pub fn new(name: &'static str, molar_mass: MolarMass<T>) -> Result<Self, NonPositiveMolarMass> {
        // `is_finite` rejects both `NaN` and infinity; the ordering rejects
        // zero and negative. Every route through here yields a strict-positive
        // finite molar mass.
        let value = *molar_mass.as_base();
        if !value.is_finite() || value <= T::ZERO {
            return Err(NonPositiveMolarMass);
        }
        Ok(Self { name, molar_mass })
    }

    /// The species name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The species molar mass, in kg/mol.
    #[must_use]
    pub const fn molar_mass(&self) -> MolarMass<T> {
        self.molar_mass
    }
}

impl<T: RealField> fmt::Display for Species<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)
    }
}
