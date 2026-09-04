//! Homogeneous reaction systems as Horae explicit systems.
//!
//! Horae owns the time-stepping; Prometheus supplies the right-hand side. A
//! system implements [`ExplicitSystem`] by writing `dC/dt` into a derivative
//! slice, so Horae's stepper advances a caller-owned concentration vector
//! without owning the reaction network's structure.

use aequitas::systems::si::quantities::ReciprocalTime;
use eunomia::RealField;
use horae::system::ExplicitSystem;
use horae::time::Instant;

/// A first-order decay `A -> products`: `d[A]/dt = -k[A]` per species.
///
/// The rate constant is a typed [`ReciprocalTime`] (s⁻¹) kept at the system
/// boundary; Horae evaluates over bare concentration scalars, which is the
/// seam's contract — Prometheus keeps the constant typed and hands Horae the
/// componentwise derivative in Hand of the same canonical unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstOrderDecay<T: RealField> {
    rate: ReciprocalTime<T>,
}

impl<T: RealField> FirstOrderDecay<T> {
    /// Construct a first-order decay with rate constant `k` (s⁻¹).
    #[must_use]
    pub const fn new(rate: ReciprocalTime<T>) -> Self {
        Self { rate }
    }
}

impl<T: RealField> ExplicitSystem<T> for FirstOrderDecay<T> {
    type Error = core::convert::Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        let rate = *self.rate.as_base();
        for (slope, concentration) in derivative.iter_mut().zip(state) {
            *slope = -rate * *concentration;
        }
        Ok(())
    }
}
