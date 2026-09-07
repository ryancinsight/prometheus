//! Zero-dimensional integration oracles.
//!
//! Each case integrates a reaction network through Horae's fixed-step RK4 and
//! checks a closed form: first-order decay, second-order decay (self reaction),
//! and reversible equilibrium. The network computes `dC/dt = ν · r` from its
//! stoichiometry and per-reaction mass-action rates.

use aequitas::systems::si::quantities::Time;
use horae::integration::tableau::Rk4;
use horae::integration::{
    BackwardEuler, ImplicitWorkspace, StepWorkspace, step_implicit_into, step_into,
};
use horae::system::{ExplicitSystem, ImplicitSystem};
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

/// Advance `state` with Horae's allocation-free damped-Newton backward Euler
/// stepper. The output and implicit workspace are allocated once for the
/// complete march, not inside the stepping loop.
fn integrate_implicit_fixed<System>(
    system: &System,
    state: &mut [f64],
    n_steps: usize,
    h: f64,
    tolerance: f64,
) where
    System: ImplicitSystem<f64>,
    System::Error: core::fmt::Debug,
{
    let start = Instant::new(Time::from_base(0.0)).expect("finite start");
    let step = StepSize::new(Time::from_base(h)).expect("finite step");
    let mut workspace = ImplicitWorkspace::<f64>::new(state.len()).expect("nonzero dimension");

    let mut output = vec![0.0; state.len()];
    let mut time = start;
    for _ in 0..n_steps {
        let report = step_implicit_into(
            system,
            BackwardEuler,
            time,
            step,
            state,
            &mut output,
            &mut workspace,
            tolerance,
        )
        .expect("implicit step advances");
        state.copy_from_slice(&output);
        time = report.end();
    }
}

fn robertson() -> ReactionNetwork<f64> {
    // A -> B; 2B -> B + C; B + C -> A + C. This is the reaction scheme in
    // Robertson (1966), as reproduced by the INdAM-Bari ROBER test report.
    let nu = StoichiometricMatrix::try_from_entries(
        3,
        3,
        [
            (0, 0, -1.0),
            (1, 0, 1.0),
            (1, 1, -1.0),
            (2, 1, 1.0),
            (0, 2, 1.0),
            (1, 2, -1.0),
        ],
    )
    .expect("in-range Robertson stoichiometry");
    ReactionNetwork::new(
        nu,
        vec![
            (0.04, vec![(0, 1)]),
            (3e7, vec![(1, 2)]),
            (1e4, vec![(1, 1), (2, 1)]),
        ],
    )
    .expect("valid Robertson network")
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

#[test]
fn rk4_recovers_fourth_order() {
    // The first-order decay is smooth, so refining the step size must recover
    // RK4's declared fourth order: err(h) / err(h/2) = 2^4 = 16.
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
        .expect("in-range stoichiometry");
    let network = ReactionNetwork::new(nu, vec![(1.0, vec![(0, 1)])]).expect("valid network");
    let exact = (-1.0_f64).exp();

    let mut errors = [0.0_f64; 3];
    for (slot, (h, steps)) in [(0.02, 50), (0.01, 100), (0.005, 200)]
        .into_iter()
        .enumerate()
    {
        let mut state = [1.0, 0.0];
        integrate_fixed(&network, &mut state, steps, h);
        errors[slot] = (state[0] - exact).abs();
    }

    // The leading term is C.h^4, so each halving divides the error by sixteen;
    // the recovered order is the base-2 logarithm of the ratio.
    let order_coarse = (errors[0] / errors[1]).log2();
    let order_fine = (errors[1] / errors[2]).log2();
    assert!(
        (order_coarse - 4.0).abs() < 0.5,
        "expected order 4, saw {order_coarse}"
    );
    assert!(
        (order_fine - 4.0).abs() < 0.5,
        "expected order 4, saw {order_fine}"
    );
}

/// 2 H2 + O2 -> 2 H2O forward network with commanding molar masses 2, 32, 18
/// (the binary-exact convention the stoichiometry oracles use).
fn water_formation() -> (ReactionNetwork<f64>, [f64; 3]) {
    let nu = StoichiometricMatrix::<f64>::try_from_entries(
        3,
        1,
        [(0, 0, -2.0), (1, 0, -1.0), (2, 0, 2.0)],
    )
    .expect("in-range stoichiometry");
    let network = ReactionNetwork::new(nu, vec![(0.5, vec![(0, 2), (1, 1)])]).expect("valid");
    (network, [2.0, 32.0, 18.0])
}

#[test]
fn balanced_network_conserves_mass_under_integration() {
    // sum_i M_i omega_i = 2(-2r) + 32(-r) + 18(2r) = 0 holds at every stage,
    // so the linear invariant sum M_i c_i is preserved by the RK step up to
    // rounding.
    let (network, masses) = water_formation();
    let mut state = [2.0, 1.0, 0.0];
    let initial: f64 = state.iter().zip(&masses).map(|(c, m)| c * m).sum();

    // t = 10 exhausts the stoichiometric charge; the integrator slows as the
    // rate approaches zero.
    integrate_fixed(&network, &mut state, 1000, 0.01);

    let final_mass: f64 = state.iter().zip(&masses).map(|(c, m)| c * m).sum();
    assert_close(initial, 36.0, 0.0);
    assert_close(final_mass, initial, 1e-12);
}

#[test]
fn concentrations_remain_nonnegative() {
    let (network, _) = water_formation();
    let mut state = [2.0, 1.0, 0.0];
    integrate_fixed(&network, &mut state, 1000, 0.01);

    for concentration in state {
        assert!(
            concentration >= 0.0,
            "concentration went negative: {concentration}"
        );
    }
}

#[test]
fn reaction_network_jacobian_matches_robertson_equations() {
    let network = robertson();
    let time = Instant::new(Time::from_base(0.0)).expect("finite fixture");
    let state = [0.7, 1e-5, 0.29999];
    let mut jacobian = [0.0_f64; 9];
    network
        .jacobian(time, &state, &mut jacobian)
        .expect("Jacobian forms");

    // The expected matrix is ∂f/∂y for the published Robertson equations.
    let expected = [
        -0.04,
        1e4 * state[2],
        1e4 * state[1],
        0.04,
        -6e7 * state[1] - 1e4 * state[2],
        -1e4 * state[1],
        0.0,
        6e7 * state[1],
        0.0,
    ];
    for (actual, expected) in jacobian.into_iter().zip(expected) {
        let scale = actual.abs().max(expected.abs()).max(1.0);
        assert!((actual - expected).abs() <= 16.0 * f64::EPSILON * scale);
    }
}

#[test]
fn backward_euler_validates_robertson_stiff_trajectory() {
    // The INdAM-Bari ROBER report (§10.2–§10.4) identifies Robertson (1966),
    // gives the reaction equations, and records the original interval as
    // 0 ≤ t ≤ 40. The reference below is an independent Radau integration of
    // that cited problem at tighter tolerances; the fixed-step method is
    // checked at t = 40 with its first-order discretization bound visible.
    //
    // Sources:
    // https://archimede.uniba.it/~testset/report/rober.pdf
    // https://scipython.com/books/book2/chapter-8-scipy/examples/solving-a-system-of-stiff-odes/
    let network = robertson();
    let mut state = [1.0, 0.0, 0.0];
    integrate_implicit_fixed(&network, &mut state, 40_000, 0.001, 1e-10);

    let reference = [
        0.715_827_068_719_463_6,
        9.185_534_764_559_855e-6,
        0.284_163_745_745_772_9,
    ];
    // Backward Euler is first order. At h = 10⁻³ the derived O(h) global
    // discretization bound is below 5·10⁻⁴ on this finite interval; the
    // reference digits themselves are much more precise.
    for (actual, expected) in state.into_iter().zip(reference) {
        assert_close(actual, expected, 5e-4);
    }
    assert_close(state.iter().sum(), 1.0, 1e-12);
    assert!(state.into_iter().all(|concentration| concentration >= 0.0));
}
