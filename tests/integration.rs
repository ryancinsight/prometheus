//! Zero-dimensional integration oracles.
//!
//! Each case integrates a reaction network through Horae's fixed-step RK4 and
//! checks a closed form: first-order decay, second-order decay (self reaction),
//! and reversible equilibrium. The network computes `dC/dt = ν · r` from its
//! stoichiometry and per-reaction mass-action rates.

use aequitas::systems::si::quantities::Time;
use horae::integration::tableau::Rk4;
use horae::integration::{StepWorkspace, step_into};
use horae::system::ExplicitSystem;
use horae::time::{Instant, StepSize};
use prometheus::{ReactionNetwork, StoichiometricMatrix};

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let error = (actual - expected).abs();
    assert!(
        error <= tolerance,
        "integrated {actual:.6} vs expected {expected:.6} (error {error:.2e})"
    );
}

/// Advance `state` by `n_steps` fixed steps of size `h` through RK4.
///
/// `state` and the internal output buffer are caller-owned `f64` slices; the
/// stepper's workspace is borrowed across the loop so no step allocates.
fn integrate_fixed<System>(system: &System, state: &mut [f64], n_steps: usize, h: f64)
where
    System: ExplicitSystem<f64>,
    System::Error: core::fmt::Debug,
{
    let start = Instant::new(Time::from_base(0.0)).expect("finite start");
    let step = StepSize::new(Time::from_base(h)).expect("finite step");
    let mut workspace = StepWorkspace::<f64, 4>::new(state.len()).expect("nonzero dimension");

    let mut output = vec![0.0; state.len()];
    let mut time = start;
    for _ in 0..n_steps {
        let report = step_into(system, Rk4, time, step, state, &mut output, &mut workspace)
            .expect("step advances");
        state.copy_from_slice(&output);
        time = report.end();
    }
}

#[test]
fn first_order_decay_matches_closed_form() {
    // A -> B: d[A]/dt = -k[A], so c_A(t) = c_A0 exp(-k t).
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
        .expect("in-range stoichiometry");
    let network = ReactionNetwork::new(nu, vec![(1.0, vec![(0, 1)])]).expect("valid network");

    let mut state = [1.0, 0.0];
    integrate_fixed(&network, &mut state, 100, 0.01);
    assert_close(state[0], (-1.0_f64).exp(), 1e-6);
}

#[test]
fn second_order_decay_matches_closed_form() {
    // 2A -> B: d[A]/dt = -2k[A]^2, so 1/c_A(t) = 1/c_A0 + 2 k t.
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -2.0), (1, 0, 1.0)])
        .expect("in-range stoichiometry");
    let network = ReactionNetwork::new(nu, vec![(1.0, vec![(0, 2)])]).expect("valid network");

    let mut state = [1.0, 0.0];
    integrate_fixed(&network, &mut state, 100, 0.01);
    // c_A(1) = 1 / (1 + 2·1·1) = 1/3.
    assert_close(state[0], 1.0 / 3.0, 1e-6);
}

#[test]
fn reversible_equilibrium_reaches_keq() {
    // A <-> B with forward k_f = 1 and reverse k_r = 0.5: K_eq = 2.
    let nu = StoichiometricMatrix::<f64>::try_from_entries(
        2,
        2,
        [(0, 0, -1.0), (1, 0, 1.0), (0, 1, 1.0), (1, 1, -1.0)],
    )
    .expect("in-range stoichiometry");
    let network = ReactionNetwork::new(nu, vec![(1.0, vec![(0, 1)]), (0.5, vec![(1, 1)])])
        .expect("valid network");

    let mut state = [1.0, 0.0];
    // t = 20 with k_f = 1 is many equilibration time scales; the integrator
    // drives A + B toward the closed-system equilibrium A:B = 1:2.
    integrate_fixed(&network, &mut state, 2000, 0.01);
    assert_close(state[1] / state[0], 2.0, 1e-4);
}

#[test]
fn network_rejects_out_of_range_reactant() {
    let nu = StoichiometricMatrix::<f64>::try_from_entries(1, 1, [(0, 0, -1.0)])
        .expect("in-range stoichiometry");
    assert!(matches!(
        ReactionNetwork::new(nu, vec![(1.0, vec![(1, 1)])]),
        Err(prometheus::InvalidNetwork::ReactantOutOfRange {
            reaction: 0,
            species: 1,
            n_species: 1,
        })
    ));
}
