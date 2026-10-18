//! Public solver and validated-concentration contracts at power boundaries.
use aequitas::systems::si::quantities::{MolarConcentration, Time};
use eunomia::RealField;
use horae::system::{ExplicitSystem, ImplicitSystem};
use horae::time::Instant;
use prometheus::{Concentration, InvalidConcentration, ReactionNetwork, StoichiometricMatrix};

fn contracts<T: RealField>(nan: T, infinity: T) {
    for value in [T::ZERO, -T::ZERO, T::ONE] {
        assert_eq!(
            *Concentration::new(MolarConcentration::from_base(value))
                .expect("finite nonnegative concentration")
                .get()
                .as_base(),
            value
        );
    }
    for value in [-T::ONE, nan, infinity, -infinity] {
        assert_eq!(
            Concentration::new(MolarConcentration::from_base(value)),
            Err(InvalidConcentration)
        );
    }
    let time = Instant::new(Time::from_base(T::ZERO)).expect("finite time");
    let m = u32::try_from(i32::MAX).expect("positive maximum");
    for order in [0, 1, 2, m - 1, m, m + 1, m + 2, u32::MAX - 1, u32::MAX] {
        let matrix =
            StoichiometricMatrix::try_from_entries(2, 1, [(0, 0, -T::ONE), (1, 0, T::ONE)])
                .expect("valid matrix");
        let network =
            ReactionNetwork::new(matrix, vec![(T::ONE, vec![(0, order), (1, 1), (0, 0)])])
                .expect("accepted order and duplicate term");
        let mut rate = [T::ZERO; 2];
        let mut jacobian = [T::ZERO; 4];
        // At B=0, only the B derivative survives, including when A=0.
        for a in [T::ZERO, T::ONE, -T::ONE] {
            network
                .evaluate(time, &[a, T::ZERO], &mut rate)
                .expect("valid shape");
            network
                .jacobian(time, &[a, T::ZERO], &mut jacobian)
                .expect("valid shape");
            assert_eq!(rate, [T::ZERO; 2]);
            let power = if order == 0 {
                T::ONE
            } else if a == T::ZERO {
                T::ZERO
            } else if a < T::ZERO && order & 1 != 0 {
                -T::ONE
            } else {
                T::ONE
            };
            assert_eq!(jacobian, [T::ZERO, -power, T::ZERO, power]);
        }
    }
}
#[test]
fn derivative_and_concentration_boundaries() {
    contracts(f32::NAN, f32::INFINITY);
    contracts(f64::NAN, f64::INFINITY);
}

#[test]
fn signed_transition_rates_and_derivatives_remain_continuous() {
    let c = 1.0 + 1.0 / 2_147_483_648.0;
    let m = u32::try_from(i32::MAX).expect("positive maximum");
    let time = Instant::new(Time::from_base(0.0)).expect("finite time");
    let mut previous_rate = None;
    for order in [m - 1, m, m + 1, m + 2, u32::MAX - 1, u32::MAX] {
        let matrix = StoichiometricMatrix::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
            .expect("valid matrix");
        let single = ReactionNetwork::new(matrix.clone(), vec![(1.0, vec![(0, order)])])
            .expect("accepted order");
        // This independent public representation uses only signed-domain powers.
        let terms = if order == u32::MAX {
            vec![(0, m), (0, m), (0, 1)]
        } else {
            let lower = order / 2;
            vec![(0, lower), (0, order - lower)]
        };
        assert!(terms.iter().all(|(_, exponent)| *exponent <= m));
        assert_eq!(
            terms
                .iter()
                .map(|(_, exponent)| u64::from(*exponent))
                .sum::<u64>(),
            u64::from(order)
        );
        let factored = ReactionNetwork::new(matrix, vec![(1.0, terms)])
            .expect("same monomial with signed-domain factors");
        let mut rate = [0.0; 2];
        let mut reference = rate;
        let mut jacobian = [0.0; 4];
        let mut reference_jacobian = jacobian;
        single
            .evaluate(time, &[c, 0.0], &mut rate)
            .expect("valid shape");
        factored
            .evaluate(time, &[c, 0.0], &mut reference)
            .expect("valid shape");
        single
            .jacobian(time, &[c, 0.0], &mut jacobian)
            .expect("valid shape");
        factored
            .jacobian(time, &[c, 0.0], &mut reference_jacobian)
            .expect("valid shape");
        // gamma_(n+8) < 2^-20 for each multiplication tree; 2^-18
        // bounds their relative disagreement with an additional factor two.
        assert!((rate[1] / reference[1] - 1.0_f64).abs() < 2.0_f64.powi(-18));
        assert!((jacobian[2] / reference_jacobian[2] - 1.0_f64).abs() < 2.0_f64.powi(-18));
        if let Some(previous) = previous_rate {
            assert!(rate[1] >= previous);
        }
        previous_rate = Some(rate[1]);
        let upper = if order <= m + 2 { 3.0 } else { 8.0 };
        let lower = if order <= m + 2 { 2.0 } else { 7.0 };
        assert!(rate[1] > lower && rate[1] < upper);
        assert!(jacobian[2] / f64::from(order) > lower);
        assert!(jacobian[2] / f64::from(order) < upper);
    }
}
