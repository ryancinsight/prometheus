//! Public-network checks for the declared unsigned reaction-order contract.
use aequitas::systems::si::quantities::Time;
use eunomia::RealField;
use horae::system::{ExplicitSystem, ImplicitSystem};
use horae::time::Instant;
use prometheus::rate::Order;
use prometheus::{InvalidNetwork, ReactionNetwork, StateDimensionMismatch, StoichiometricMatrix};

fn network<T: RealField>(terms: Vec<(usize, Order)>) -> ReactionNetwork<T> {
    let nu = StoichiometricMatrix::try_from_entries(2, 1, [(0, 0, -T::ONE), (1, 0, T::ONE)])
        .expect("two species, one in-range reaction");
    ReactionNetwork::new(nu, vec![(T::ONE, terms)])
        .expect("finite nonnegative coefficient and in-range terms")
}
fn evaluate<T: RealField>(system: &ReactionNetwork<T>, base: T) -> ([T; 2], [T; 4]) {
    let time = Instant::new(Time::from_base(T::ZERO)).expect("finite time");
    let mut rate = [T::from_integer(17); 2];
    let mut jacobian = [T::from_integer(19); 4];
    system
        .evaluate(time, &[base, T::ZERO], &mut rate)
        .expect("valid state/output dimensions");
    system
        .jacobian(time, &[base, T::ZERO], &mut jacobian)
        .expect("valid state/Jacobian dimensions");
    assert_eq!(
        rate[0] + rate[1],
        T::ZERO,
        "equal molar masses conserve total production"
    );
    assert_eq!(jacobian[0] + jacobian[2], T::ZERO);
    assert_eq!(jacobian[1], T::ZERO);
    assert_eq!(jacobian[3], T::ZERO);
    (rate, jacobian)
}
fn exact_controls<T: RealField>() {
    let m = Order::try_from(i32::MAX).expect("positive signed maximum");
    for order in [0, 1, 2, 3, m, m + 1, Order::MAX] {
        let system = network::<T>(vec![(0, order)]);
        let (rate, jacobian) = evaluate(&system, T::ONE);
        assert_eq!(rate, [-T::ONE, T::ONE]);
        let derivative = T::from_integer(i64::from(order));
        assert_eq!(jacobian, [-derivative, T::ZERO, derivative, T::ZERO]);
        let (rate, jacobian) = evaluate(&system, T::ZERO);
        let expected = if order == 0 { T::ONE } else { T::ZERO };
        assert_eq!(rate, [-expected, expected]);
        let derivative = if order == 1 { T::ONE } else { T::ZERO };
        assert_eq!(jacobian, [-derivative, T::ZERO, derivative, T::ZERO]);
    }
}
#[test]
fn exact_zero_unit_and_unsigned_boundary_controls() {
    exact_controls::<f32>();
    exact_controls::<f64>();
}

#[test]
fn constructor_and_shape_error_controls() {
    let nu = StoichiometricMatrix::<f64>::try_from_entries(2, 1, [(0, 0, -1.0), (1, 0, 1.0)])
        .expect("valid matrix");
    assert_eq!(
        ReactionNetwork::new(nu.clone(), vec![]).expect_err("reaction list must match"),
        InvalidNetwork::ReactionCountMismatch {
            n_reactions: 1,
            supplied: 0
        }
    );
    assert_eq!(
        ReactionNetwork::new(nu.clone(), vec![(1.0, vec![(2, Order::MAX)])])
            .expect_err("bad species index"),
        InvalidNetwork::ReactantOutOfRange {
            reaction: 0,
            species: 2,
            n_species: 2
        }
    );
    for coefficient in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            ReactionNetwork::new(nu.clone(), vec![(coefficient, vec![(0, 1)])])
                .expect_err("bad coefficient"),
            InvalidNetwork::NonFiniteRate { reaction: 0 }
        );
    }
    let system = network::<f64>(vec![(0, 1)]);
    let time = Instant::new(Time::from_base(0.0)).expect("finite time");
    let mut output = [17.0, 19.0, 23.0];
    let before = output.map(f64::to_bits);
    assert_eq!(
        system.evaluate(time, &[1.0, 0.0], &mut output),
        Err(StateDimensionMismatch {
            n_species: 2,
            state: 2,
            output: 3
        })
    );
    assert_eq!(output.map(f64::to_bits), before);
    assert_eq!(
        system.jacobian(time, &[1.0, 0.0], &mut output),
        Err(StateDimensionMismatch {
            n_species: 2,
            state: 2,
            output: 3
        })
    );
    assert_eq!(output.map(f64::to_bits), before);
}
fn base() -> f64 {
    1.0 + 1.0 / 2_147_483_648.0
}
fn factorization() -> ReactionNetwork<f64> {
    let m = Order::try_from(i32::MAX).expect("signed boundary fits Order");
    // MAX=2*m+1, so these legal terms describe exactly the same monomial
    // as a single MAX-order term, without invoking any unsigned overflow.
    network(vec![(0, m), (0, m), (0, 1)])
}
#[test]
fn signed_boundary_and_equivalent_factorization_controls() {
    let m = Order::try_from(i32::MAX).expect("signed maximum");
    let (lower, jacobian) = evaluate(&network(vec![(0, m)]), base());
    assert!(
        lower[1] > 2.0 && lower[1] < 3.0,
        "signed-boundary rate: {}",
        lower[1]
    );
    let scaled = jacobian[2] / f64::from(m);
    assert!(
        scaled > 2.0 && scaled < 3.0,
        "signed-boundary derivative: {scaled}"
    );
    let (full, jacobian) = evaluate(&factorization(), base());
    assert!(
        full[1] > 7.0 && full[1] < 8.0,
        "equivalent full-order rate: {}",
        full[1]
    );
    let scaled = jacobian[2] / f64::from(Order::MAX);
    assert!(
        scaled > 7.0 && scaled < 8.0,
        "equivalent full-order derivative: {scaled}"
    );
}
#[test]
fn full_unsigned_order_rate_retains_the_declared_exponent() {
    let (actual, _) = evaluate(&network(vec![(0, Order::MAX)]), base());
    // Independent rational/binomial bounds (saved in order-oracle.json)
    // place both exact c^MAX and its native multiplication result in (7,8).
    assert!(
        actual[1] > 7.0 && actual[1] < 8.0,
        "accepted order {} must yield a rate in (7,8); got {}",
        Order::MAX,
        actual[1]
    );
}
#[test]
fn full_unsigned_order_jacobian_retains_the_declared_exponent() {
    let (_, actual) = evaluate(&network(vec![(0, Order::MAX)]), base());
    let scaled = actual[2] / f64::from(Order::MAX);
    assert!(
        scaled > 7.0 && scaled < 8.0,
        "Jacobian/n must retain exponent n-1 and lie in (7,8); got {scaled}"
    );
}
