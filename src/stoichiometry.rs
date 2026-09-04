//! Sparse stoichiometric matrix over Leto.
//!
//! Convention: `nu` is species-by-reaction, with a negative coefficient for a
//! reactant of reaction `j` and a positive coefficient for a product. Net
//! production rates are then `omega = nu * r`, and a mass-conserving network
//! satisfies `transpose(nu) * M = 0` exactly, where `M` is the vector of
//! species molar masses.

use alloc::vec::Vec;

use aequitas::systems::si::quantities::ReactionRate;
use eunomia::RealField;
use leto::{CooArray, SparseStorage, SparseStorageMut};

use crate::species::MolarMass;

/// A sparse stoichiometric matrix with shape species-by-reaction.
///
/// Stored as a Leto COO array so construction is entry-wise and the sparse
/// layout is the substrate's rather than a hand-rolled parallel.
#[derive(Clone, Debug)]
pub struct StoichiometricMatrix<T: RealField> {
    matrix: CooArray<T>,
}

/// A stoichiometric matrix was rejected at construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidStoichiometry {
    /// Zero species or zero reactions is not a network.
    Empty,
    /// An entry indexed a species or reaction outside the declared shape.
    IndexOutOfRange {
        /// Species index that was out of range, when that was the fault.
        species: Option<usize>,
        /// Reaction index that was out of range, when that was the fault.
        reaction: Option<usize>,
        /// Declared species count.
        n_species: usize,
        /// Declared reaction count.
        n_reactions: usize,
    },
}

/// A mass-conservation check received a molar-mass vector whose length does
/// not match the matrix's species dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MisshapedMasses {
    /// Species count implied by the matrix.
    pub n_species: usize,
    /// Length of the molar-mass slice supplied by the caller.
    pub supplied: usize,
}

/// A net-production call received a rate vector whose length does not match
/// the matrix's reaction dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MisshapedRates {
    /// Reaction count implied by the matrix.
    pub n_reactions: usize,
    /// Length of the reaction-rate slice supplied by the caller.
    pub supplied: usize,
}

impl<T: RealField> StoichiometricMatrix<T> {
    /// Build the matrix from `(species, reaction, coefficient)` triplets.
    ///
    /// Duplicate entries for the same `(species, reaction)` are summed by the
    /// Leto COO setter, so a caller that emits the same slot twice does not
    /// get a silently doubled coefficient from an append-only construction.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidStoichiometry::Empty`] when either dimension is zero,
    /// and [`InvalidStoichiometry::IndexOutOfRange`] when any index falls
    /// outside the declared shape.
    pub fn try_from_entries<I>(
        n_species: usize,
        n_reactions: usize,
        entries: I,
    ) -> Result<Self, InvalidStoichiometry>
    where
        I: IntoIterator<Item = (usize, usize, T)>,
    {
        if n_species == 0 || n_reactions == 0 {
            return Err(InvalidStoichiometry::Empty);
        }

        let mut matrix = CooArray::new(n_species, n_reactions);
        for (species, reaction, coefficient) in entries {
            if species >= n_species || reaction >= n_reactions {
                return Err(InvalidStoichiometry::IndexOutOfRange {
                    species: (species >= n_species).then_some(species),
                    reaction: (reaction >= n_reactions).then_some(reaction),
                    n_species,
                    n_reactions,
                });
            }
            // Sum with any prior contribution at this slot so construction is
            // associative in the order of the iterator.
            let prior = matrix.get(species, reaction).unwrap_or(T::ZERO);
            matrix.set(species, reaction, prior + coefficient);
        }

        Ok(Self { matrix })
    }

    /// Number of species (row count).
    #[must_use]
    pub fn n_species(&self) -> usize {
        self.matrix.nrows()
    }

    /// Number of reactions (column count).
    #[must_use]
    pub fn n_reactions(&self) -> usize {
        self.matrix.ncols()
    }

    /// The coefficient at `(species, reaction)`, or zero when the slot is
    /// absent from the sparse store.
    #[must_use]
    pub fn coefficient(&self, species: usize, reaction: usize) -> T {
        self.matrix.get(species, reaction).unwrap_or(T::ZERO)
    }

    /// Mass-conservation residual per reaction: sum over species of
    /// `nu[species, reaction] * molar_mass[species]`.
    ///
    /// A balanced network yields the zero vector exactly when the coefficients
    /// and molar masses are representable without rounding; otherwise the
    /// residual is bounded by the floating-point accumulation.
    ///
    /// # Errors
    ///
    /// Returns [`MisshapedMasses`] when `molar_masses.len()` differs from
    /// [`Self::n_species`].
    pub fn mass_residuals(&self, molar_masses: &[MolarMass<T>]) -> Result<Vec<T>, MisshapedMasses> {
        if molar_masses.len() != self.n_species() {
            return Err(MisshapedMasses {
                n_species: self.n_species(),
                supplied: molar_masses.len(),
            });
        }

        let mut residuals = alloc::vec![T::ZERO; self.n_reactions()];
        for (species, reaction, coefficient) in self.matrix.entries() {
            let mass = *molar_masses[species].as_base();
            residuals[reaction] += *coefficient * mass;
        }
        Ok(residuals)
    }

    /// Whether every reaction's mass residual is within `tolerance` of zero.
    ///
    /// # Errors
    ///
    /// Propagates [`MisshapedMasses`] from [`Self::mass_residuals`].
    pub fn conserves_mass(
        &self,
        molar_masses: &[MolarMass<T>],
        tolerance: T,
    ) -> Result<bool, MisshapedMasses> {
        let residuals = self.mass_residuals(molar_masses)?;
        Ok(residuals.iter().all(|residual| residual.abs() <= tolerance))
    }

    /// Net production rate of every species: `omega_i = sum_j nu_ij * r_j`.
    ///
    /// `rates` holds one reaction rate per reaction column, in mol·m⁻³·s⁻¹.
    /// Each stoichiometric coefficient is dimensionless, scaling its reaction's
    /// rate into that species' production, so the result is again mol·m⁻³·s⁻¹
    /// and carries [`ReactionRate`] at the type level rather than a bare
    /// scalar.
    ///
    /// # Errors
    ///
    /// Returns [`MisshapedRates`] when `rates.len()` differs from
    /// [`Self::n_reactions`].
    pub fn net_production(
        &self,
        rates: &[ReactionRate<T>],
    ) -> Result<Vec<ReactionRate<T>>, MisshapedRates> {
        if rates.len() != self.n_reactions() {
            return Err(MisshapedRates {
                n_reactions: self.n_reactions(),
                supplied: rates.len(),
            });
        }

        let mut omega = alloc::vec![ReactionRate::from_base(T::ZERO); self.n_species()];
        for (species, reaction, coefficient) in self.matrix.entries() {
            omega[species] += rates[reaction] * *coefficient;
        }
        Ok(omega)
    }
}
