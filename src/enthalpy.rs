//! Reaction enthalpy and volumetric heat release.
//!
//! A reaction's molar enthalpy `ΔHⱼ` (J/mol) times its rate `rⱼ`
//! (mol·m⁻³·s⁻¹) is a volumetric power (W/m³); summing over reactions gives
//! the total heat-release rate — the coupling term a thermal balance consumes.
//! Both terms are typed aequitas quantities, and the product dimension is
//! derived by aequitas multiplication rather than asserted.

use aequitas::systems::si::quantities::{MolarEnergy, ReactionRate, VolumetricPowerDensity};
use eunomia::RealField;

/// A heat-release call received rate and enthalpy vectors of different
/// lengths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MisshapedEnthalpies {
    /// Number of reactions implied by the rate vector.
    pub n_rates: usize,
    /// Number of enthalpies supplied.
    pub n_enthalpies: usize,
}

/// Total volumetric heat-release rate: `q = Σⱼ rⱼ ΔHⱼ`.
///
/// `rates` and `enthalpies` are parallel, one entry per reaction. The result
/// has dimension power per volume, derived as `ReactionRate × MolarEnergy`, so
/// an enthalpy with the wrong dimension is a compile error rather than a unit
/// convention (the typed counterpart of the mass-action and molar-mass
/// oracles).
///
/// # Errors
///
/// Returns [`MisshapedEnthalpies`] when the two slices' lengths differ.
pub fn heat_release<T: RealField>(
    rates: &[ReactionRate<T>],
    enthalpies: &[MolarEnergy<T>],
) -> Result<VolumetricPowerDensity<T>, MisshapedEnthalpies> {
    if rates.len() != enthalpies.len() {
        return Err(MisshapedEnthalpies {
            n_rates: rates.len(),
            n_enthalpies: enthalpies.len(),
        });
    }

    let total = rates.iter().zip(enthalpies).fold(
        VolumetricPowerDensity::from_base(T::ZERO),
        |acc, (rate, enthalpy)| acc + *rate * *enthalpy,
    );

    Ok(total)
}
