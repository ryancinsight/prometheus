//! Zero-dimensional integration oracles.
//!
//! The first-order decay `d[A]/dt = -k[A]` integrates to the closed form
//! `c_A(t) = c_A0 · exp(-k t)`, asserted after fixed-step RK4 through Horae's
//! stepper — the end-to-end proof that Prometheus's system rides the Horae seam.

use aequitas::systems::si::quantities::{ReciprocalTime, Time};
use horae::integration::tableau::Rk4;
use horae::integration::{StepWorkspace, step_into};
use horae::time::{Instant, StepSize};
use prometheus::FirstOrderDecay;

#[test]
fn first_order_decay_matches_closed_form() {
    let k = 1.0_f64;
    let system = FirstOrderDecay::new(ReciprocalTime::from_base(k));

    let start = Instant::new(Time::from_base(0.0)).expect("finite start");
    let step = StepSize::new(Time::from_base(0.01)).expect("finite step");
    let mut workspace = StepWorkspace::<f64, 4>::new(1).expect("nonzero dimension");

    let mut state = [1.0_f64]; // c_A0
    let mut output = [0.0_f64];
    let mut time = start;

    for _ in 0..100 {
        let report = step_into(
            &system,
            Rk4,
            time,
            step,
            &state,
            &mut output,
            &mut workspace,
        )
        .expect("step advances");
        state = output;
        time = report.end();
    }

    // t = 1.0 s, k = 1.0: c_A(1) = e^-1. RK4's fixed-step global error is
    // O(h^4) ≈ 1e-8 here, so the tolerance is far looser than the error while
    // still catching a wrong derivative or a broken step update.
    let expected = (-1.0_f64).exp();
    let error = (state[0] - expected).abs();
    assert!(
        error <= 1e-6,
        "integrated {:.6} vs closed form {expected:.6} (error {error:.2e})",
        state[0]
    );
}
