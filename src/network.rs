//! Homogeneous reaction networks as Horae explicit systems.
//!
//! Horae owns the time-stepping; Prometheus supplies the right-hand side. A
//! [`ReactionNetwork`] implements [`ExplicitSystem`] by computing the net
//! production `ω = ν · r` from the stored stoichiometry and per-reaction
//! mass-action rates, writing `dC/dt` into the derivative slice Horae drives.

use alloc::vec::Vec;

use eunomia::RealField;
use horae::system::ExplicitSystem;
use horae::time::Instant;

use crate::rate::{Order, integer_power};
use crate::stoichiometry::StoichiometricMatrix;

/// A reaction was supplied with a rate coefficient that is not finite and
/// non-negative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidNetwork {
    /// The reaction list length differs from the matrix's reaction dimension.
    ReactionCountMismatch {
        /// Reaction count implied by the stoichiometric matrix.
        n_reactions: usize,
        /// Reaction count supplied to the constructor.
        supplied: usize,
    },
    /// A reactant indexed a species outside the declared range.
    ReactantOutOfRange {
        /// The reaction carrying the bad index.
        reaction: usize,
        /// The out-of-range species index.
        species: usize,
        /// Declared species count.
        n_species: usize,
    },
    /// A rate coefficient was negative, `NaN`, or infinite.
    NonFiniteRate {
        /// The reaction carrying the bad coefficient.
        reaction: usize,
    },
}

/// A system was evaluated with state or derivative slices that do not match
/// the network's species count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateDimensionMismatch {
    /// Species count declared by the network.
    pub n_species: usize,
    /// Length of the state slice supplied.
    pub state: usize,
    /// Length of the derivative slice supplied.
    pub derivative: usize,
}

/// A homogeneous reaction network: `dC/dt = ν · r` with mass-action rates.
///
/// Each reaction carries a rate coefficient (its order-dependent scalar value)
/// and the list of `(species, order)` reactant terms whose product the
/// coefficient scales. The derivative is the net production over the
/// stoichiometric matrix, computed allocation-free by accumulating directly
/// into the derivative slice.
#[derive(Clone, Debug)]
pub struct ReactionNetwork<T: RealField> {
    stoichiometry: StoichiometricMatrix<T>,
    reactions: Vec<(T, Vec<(usize, Order)>)>,
}

impl<T: RealField> ReactionNetwork<T> {
    /// Construct a network from a stoichiometric matrix and per-reaction
    /// mass-action data.
    ///
    /// `reactions[j]` is `(rate_coefficient, reactants)` where each reactant is
    /// a `(species_index, order)` pair; the coefficient is the reaction rate
    /// constant in its order-dependent unit (`s⁻¹` first order, `m³·mol⁻¹·s⁻¹`
    /// second order, and so on).
    ///
    /// # Errors
    ///
    /// Returns [`InvalidNetwork`] on a reaction-count mismatch, an
    /// out-of-range reactant index, or a non-finite rate coefficient.
    pub fn new(
        stoichiometry: StoichiometricMatrix<T>,
        reactions: Vec<(T, Vec<(usize, Order)>)>,
    ) -> Result<Self, InvalidNetwork> {
        let n_species = stoichiometry.n_species();
        let n_reactions = stoichiometry.n_reactions();

        if reactions.len() != n_reactions {
            return Err(InvalidNetwork::ReactionCountMismatch {
                n_reactions,
                supplied: reactions.len(),
            });
        }

        for (reaction, (coefficient, reactants)) in reactions.iter().enumerate() {
            if !coefficient.is_finite() || *coefficient < T::ZERO {
                return Err(InvalidNetwork::NonFiniteRate { reaction });
            }
            for (species, _) in reactants {
                if *species >= n_species {
                    return Err(InvalidNetwork::ReactantOutOfRange {
                        reaction,
                        species: *species,
                        n_species,
                    });
                }
            }
        }

        Ok(Self {
            stoichiometry,
            reactions,
        })
    }

    /// Number of species in the network.
    #[must_use]
    pub fn n_species(&self) -> usize {
        self.stoichiometry.n_species()
    }
}

impl<T: RealField> ExplicitSystem<T> for ReactionNetwork<T> {
    type Error = StateDimensionMismatch;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        let n_species = self.n_species();
        if state.len() != n_species || derivative.len() != n_species {
            return Err(StateDimensionMismatch {
                n_species,
                state: state.len(),
                derivative: derivative.len(),
            });
        }

        derivative.fill(T::ZERO);
        for (reaction, (coefficient, reactants)) in self.reactions.iter().enumerate() {
            let rate = reactants
                .iter()
                .fold(*coefficient, |acc, (species, order)| {
                    acc * integer_power(state[*species], *order)
                });
            for (species, production) in derivative.iter_mut().enumerate() {
                *production += self.stoichiometry.coefficient(species, reaction) * rate;
            }
        }
        Ok(())
    }
}
