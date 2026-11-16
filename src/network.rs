//! Homogeneous reaction networks as Horae explicit and implicit systems.
//!
//! Horae owns the time-stepping; Prometheus supplies the right-hand side. A
//! [`ReactionNetwork`] implements Horae's system seams by computing the net
//! production `ω = ν · r` from the stored stoichiometry and per-reaction
//! mass-action rates. Its analytic Jacobian is assembled from the same
//! mass-action representation, so implicit consumers do not need a separate
//! reaction-system implementation.

use alloc::vec::Vec;

use eunomia::RealField;
use horae::system::{ExplicitSystem, ImplicitSystem};
use horae::time::Instant;

use crate::rate::{Order, RateConstant, RateOrderMismatch, mass_action_product};
use crate::stoichiometry::StoichiometricMatrix;

/// A reaction network was supplied with an inconsistent reaction list.
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
    /// A rate coefficient was `NaN` or infinite.
    NonFiniteRate {
        /// The reaction carrying the bad coefficient.
        reaction: usize,
    },
    /// A rate coefficient was negative.
    NegativeRate {
        /// The reaction carrying the bad coefficient.
        reaction: usize,
    },
    /// A rate coefficient's dimension belongs to a different overall order
    /// than the sum of its reactant orders.
    OrderMismatch {
        /// The reaction carrying the mismatched coefficient.
        reaction: usize,
        /// The coefficient's order and the reactant order sum.
        mismatch: RateOrderMismatch,
    },
}

/// A system was evaluated with state or output slices that do not match the
/// network's species count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateDimensionMismatch {
    /// Species count declared by the network.
    pub n_species: usize,
    /// Length of the state slice supplied.
    pub state: usize,
    /// Length of the derivative or Jacobian output slice supplied.
    pub output: usize,
}

/// A homogeneous reaction network: `dC/dt = ν · r` with mass-action rates.
///
/// Each reaction carries a typed [`RateConstant`] (its value and the overall
/// order its dimension belongs to) and the list of `(species, order)` reactant terms whose product the
/// coefficient scales. The derivative is the net production over the
/// stoichiometric matrix, computed allocation-free by accumulating directly
/// into the derivative slice.
#[derive(Clone, Debug)]
pub struct ReactionNetwork<T: RealField> {
    stoichiometry: StoichiometricMatrix<T>,
    reactions: Vec<(RateConstant<T>, Vec<(usize, Order)>)>,
}

impl<T: RealField> ReactionNetwork<T> {
    /// Construct a network from a stoichiometric matrix and per-reaction
    /// mass-action data.
    ///
    /// `reactions[j]` is `(rate_coefficient, reactants)` where each reactant is
    /// a `(species_index, order)` pair; the coefficient is the reaction rate
    /// constant converted from its typed quantity (`s⁻¹` first order,
    /// `m³·mol⁻¹·s⁻¹` second order, and so on), whose order must equal the sum
    /// of the reactant orders.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidNetwork`] on a reaction-count mismatch, an
    /// out-of-range reactant index, a non-finite or negative rate coefficient,
    /// or a coefficient whose order differs from its reactant order sum.
    pub fn new(
        stoichiometry: StoichiometricMatrix<T>,
        reactions: Vec<(RateConstant<T>, Vec<(usize, Order)>)>,
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
            let value = coefficient.base();
            if !value.is_finite() {
                return Err(InvalidNetwork::NonFiniteRate { reaction });
            }
            if value < T::ZERO {
                return Err(InvalidNetwork::NegativeRate { reaction });
            }
            coefficient
                .check_order(reactants.iter().map(|(_, order)| *order))
                .map_err(|mismatch| InvalidNetwork::OrderMismatch { reaction, mismatch })?;
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
                output: derivative.len(),
            });
        }

        derivative.fill(T::ZERO);
        for (reaction, (coefficient, reactants)) in self.reactions.iter().enumerate() {
            let rate = mass_action_product(
                coefficient.base(),
                reactants
                    .iter()
                    .map(|(species, order)| (state[*species], *order)),
            );
            for (species, nu) in self.stoichiometry.column(reaction) {
                derivative[species] += *nu * rate;
            }
        }
        Ok(())
    }
}

impl<T: RealField> ImplicitSystem<T> for ReactionNetwork<T> {
    fn jacobian(
        &self,
        _time: Instant<T>,
        state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        let n_species = self.n_species();
        let expected_jacobian = n_species * n_species;
        if state.len() != n_species || jacobian.len() != expected_jacobian {
            return Err(StateDimensionMismatch {
                n_species,
                state: state.len(),
                output: jacobian.len(),
            });
        }

        jacobian.fill(T::ZERO);
        for (reaction, (coefficient, reactants)) in self.reactions.iter().enumerate() {
            for (term, (differentiated_species, order)) in reactants.iter().enumerate() {
                if *order == 0 {
                    continue;
                }

                // Differentiate the product directly instead of dividing by a
                // concentration. This remains defined at the zero-concentration
                // boundary where a valid kinetic state commonly starts.
                let derivative_rate = mass_action_product(
                    coefficient.base() * T::from_f64(f64::from(*order)),
                    reactants
                        .iter()
                        .enumerate()
                        .map(|(factor, (species, exponent))| {
                            let exponent = if factor == term {
                                order.saturating_sub(1)
                            } else {
                                *exponent
                            };
                            (state[*species], exponent)
                        }),
                );

                for (species, nu) in self.stoichiometry.column(reaction) {
                    jacobian[species * n_species + *differentiated_species] +=
                        *nu * derivative_rate;
                }
            }
        }
        Ok(())
    }
}
