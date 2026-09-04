//! Species mass balance: homogeneous reaction networks and zero-dimensional
//! kinetics.
//!
//! Prometheus owns the transformation side of chemical species — their
//! identity, stoichiometry, rate laws, and the net production and enthalpy a
//! transport equation consumes — and nothing about the *transport* of those
//! species: that discretization belongs to whichever balance domain carries
//! the field (atlas [ADR 0055] and [ADR 0058]).
//!
//! Phase 0 is zero-dimensional on purpose: the network integrates a species
//! vector forward from an initial composition, a temperature, and a reaction
//! network, using aequitas quantities for units, eunomia scalars for the
//! numeric dimension, and Leto sparse arrays for the stoichiometric matrix.
//! Every claim carries an analytical oracle — closed form, conservation law,
//! or published benchmark — because, as new construction, there is no
//! reference implementation to difference against.
//!
//! [ADR 0055]: https://github.com/ryancinsight/atlas/blob/main/docs/adr/0055-continuum-domain-decomposition.md
//! [ADR 0058]: https://github.com/ryancinsight/atlas/blob/main/docs/adr/0058-prometheus-phase-0-charter.md

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(unreachable_pub)]

extern crate alloc;

pub mod concentration;
pub mod enthalpy;
pub mod rate;
pub mod species;
#[cfg(feature = "std")]
pub mod stoichiometry;
#[cfg(feature = "std")]
pub mod system;

pub use concentration::{Concentration, InvalidConcentration};
pub use enthalpy::{MisshapedEnthalpies, heat_release};
pub use rate::{
    FirstOrderConstant, SecondOrderConstant, ThirdOrderConstant, first_order_rate, mass_action_rate,
};
pub use species::{MolarMass, NonPositiveMolarMass, Species};
#[cfg(feature = "std")]
pub use stoichiometry::{
    InvalidStoichiometry, MisshapedMasses, MisshapedRates, StoichiometricMatrix,
};
#[cfg(feature = "std")]
pub use system::FirstOrderDecay;
