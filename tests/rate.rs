//! Mass-action rate-law oracles and the rate-constant order check.
//!
//! Each closed form is a mass-action product asserted in SI units
//! (`mol·m⁻³·s⁻¹`), at every shipped scalar type. The inputs are dyadic
//! rationals whose products are exactly representable in `f32` and `f64`, so
//! every product is exact and the derived tolerance is zero: a mismatch is a
//! wrong product or a wrong power, never rounding.
//!
//! The rate-constant dimension is `mol^(1-n)·m^(3n-3)·s⁻¹` for overall order
//! `n` (`[k] = [r]/[c]^n`); a coefficient whose order differs from the sum of
//! the reactant orders is rejected with a typed error, directly and through
//! `ReactionNetwork`. A dimension that is no rate constant does not compile
//! (the `compile_fail` doctests of `RateConstantDimension` and
//! `RateConstant`).

pub mod common;

use core::any::type_name;
use core::fmt::Debug;

use aequitas::Quantity;
use aequitas::dimension::{BaseSemantics, Dimension};
use aequitas::systems::si::quantities::{MolarConcentration, ReactionRate, ReciprocalTime, Time};
use eunomia::RealField;
use horae::system::ExplicitSystem;
use horae::time::Instant;
use prometheus::rate::{
    FirstOrderConstant, RateConstant, RateOrderMismatch, SecondOrderConstant, ThirdOrderConstant,
    first_order_rate, mass_action_rate,
};
use prometheus::{InvalidNetwork, ReactionNetwork, StoichiometricMatrix};
use typenum::{N1, N3, N6, N2147483647, P1, P2, P9, P2147483647, Prod, Sum, Z0};

/// Order 4: `mol⁻³·m⁹·s⁻¹`.
type FourthOrder = Quantity<f64, Dimension<P9, Z0, N1, Z0, Z0, N3, Z0>>;
/// Order -1: `mol²·m⁻⁶·s⁻¹`, an order no reaction has.
type NegativeOrder = Quantity<f64, Dimension<N6, Z0, N1, Z0, Z0, P2, Z0>>;

fn conc<T: RealField>(value: f64) -> MolarConcentration<T> {
    MolarConcentration::from_base(T::from_f64(value))
}

fn assert_exact<T: RealField + Debug>(rate: ReactionRate<T>, expected: f64) {
    assert_eq!(
        *rate.as_base(),
        T::from_f64(expected),
        "{}",
        type_name::<T>()
    );
}

fn closed_forms<T: RealField + Debug>() {
    // Zeroth order: no participating species, rate = k.
    let zeroth = mass_action_rate(ReactionRate::from_base(T::from_f64(3.5)), &[]);
    assert_exact(zeroth.expect("order 0 constant, no reactants"), 3.5);

    // A -> B: rate = k·c_A, k in s⁻¹; the typed form agrees with the general one.
    let k1: FirstOrderConstant<T> = ReciprocalTime::from_base(T::from_f64(0.5));
    assert_exact(first_order_rate(k1, conc(2.0)), 1.0);
    assert_exact(
        mass_action_rate(k1, &[(1, conc(2.0))]).expect("order 1 constant, order 1 sum"),
        1.0,
    );

    // 2 A -> B: rate = k·c_A², k in m³·mol⁻¹·s⁻¹.
    let k2: SecondOrderConstant<T> = Quantity::from_base(T::from_f64(0.25));
    assert_exact(
        mass_action_rate(k2, &[(2, conc(3.0))]).expect("order 2 constant, order 2 sum"),
        2.25,
    );

    // A + B -> C: orders 1 + 1 sum to 2, rate = k·c_A·c_B.
    let k2: SecondOrderConstant<T> = Quantity::from_base(T::from_f64(0.125));
    assert_exact(
        mass_action_rate(k2, &[(1, conc(2.0)), (1, conc(5.0))]).expect("1 + 1 = 2"),
        1.25,
    );

    // 3 A -> B: rate = k·c_A³, k in m⁶·mol⁻²·s⁻¹.
    let k3: ThirdOrderConstant<T> = Quantity::from_base(T::from_f64(0.5));
    assert_exact(
        mass_action_rate(k3, &[(3, conc(2.0))]).expect("order 3 constant, order 3 sum"),
        4.0,
    );
}

#[test]
fn mass_action_rate_matches_closed_forms_at_every_scalar() {
    closed_forms::<f32>();
    closed_forms::<f64>();
}

/// A mismatch is reported in both directions with both orders.
fn mismatches<T: RealField + Debug>() {
    let c = conc::<T>(2.0);
    let first: FirstOrderConstant<T> = ReciprocalTime::from_base(T::ONE);
    let second: SecondOrderConstant<T> = Quantity::from_base(T::ONE);
    let third: ThirdOrderConstant<T> = Quantity::from_base(T::ONE);
    let zeroth = ReactionRate::from_base(T::ONE);
    let mismatch = |coefficient_order, reactant_order| {
        Some(RateOrderMismatch {
            coefficient_order,
            reactant_order,
        })
    };

    assert_eq!(mass_action_rate(first, &[(2, c)]).err(), mismatch(1, 2));
    assert_eq!(mass_action_rate(second, &[(1, c)]).err(), mismatch(2, 1));
    assert_eq!(mass_action_rate(first, &[]).err(), mismatch(1, 0));
    assert_eq!(mass_action_rate(zeroth, &[(1, c)]).err(), mismatch(0, 1));
    assert_eq!(
        mass_action_rate(third, &[(1, c), (1, c)]).err(),
        mismatch(3, 2)
    );
    // The reactant sum is formed in `u64`: two `u32::MAX` orders do not wrap.
    assert_eq!(
        mass_action_rate(second, &[(u32::MAX, c), (u32::MAX, c)]).err(),
        mismatch(2, 2 * u64::from(u32::MAX))
    );
}

#[test]
fn rate_constant_order_mismatch_is_a_typed_error_at_every_scalar() {
    mismatches::<f32>();
    mismatches::<f64>();
}

#[test]
fn rate_constant_order_is_read_from_the_dimension() {
    let order = |constant: RateConstant<f64>| constant.order();
    assert_eq!(order(ReactionRate::from_base(1.0).into()), 0);
    assert_eq!(order(FirstOrderConstant::from_base(1.0).into()), 1);
    assert_eq!(order(SecondOrderConstant::from_base(1.0).into()), 2);
    assert_eq!(order(ThirdOrderConstant::from_base(1.0).into()), 3);

    // The value is kept in the base unit of its dimension, bit for bit.
    let second = RateConstant::from(SecondOrderConstant::from_base(0.25_f64));
    assert_eq!(second.base().to_bits(), 0.25_f64.to_bits());

    assert_eq!(order(FourthOrder::from_base(1.0).into()), 4);

    // A negative order matches no reactant sum, the empty one included.
    let negative = RateConstant::from(NegativeOrder::from_base(1.0));
    assert_eq!(negative.order(), -1);
    let network = ReactionNetwork::new(
        matrix::<f64>(1, &[(0, 0, 1.0)]),
        vec![(negative, Vec::new())],
    );
    assert_eq!(
        network.err(),
        Some(InvalidNetwork::OrderMismatch {
            reaction: 0,
            mismatch: RateOrderMismatch {
                coefficient_order: -1,
                reactant_order: 0,
            },
        })
    );
}

fn matrix<T: RealField>(
    n_species: usize,
    entries: &[(usize, usize, f64)],
) -> StoichiometricMatrix<T> {
    let n_reactions = entries.iter().map(|(_, reaction, _)| reaction + 1).max();
    StoichiometricMatrix::try_from_entries(
        n_species,
        n_reactions.unwrap_or(1),
        entries
            .iter()
            .map(|(species, reaction, nu)| (*species, *reaction, T::from_f64(*nu))),
    )
    .expect("in-range stoichiometry")
}

/// Derivative of `A -> B` (`k = 0.5 s⁻¹`, `c_A = 2`) and `2 A -> B`
/// (`k = 0.25 m³·mol⁻¹·s⁻¹`, `c_A = 3`) in SI units: `r = 0.5·2 = 1` and
/// `r = 0.25·3² = 2.25`, with `dC/dt = ν·r`.
fn network_derivatives<T: RealField + Debug>() {
    let time = Instant::new(Time::from_base(T::ZERO)).expect("finite start");
    let mut derivative = [T::ZERO; 2];

    let first = ReactionNetwork::new(
        matrix(2, &[(0, 0, -1.0), (1, 0, 1.0)]),
        vec![(
            FirstOrderConstant::from_base(T::from_f64(0.5)).into(),
            vec![(0, 1)],
        )],
    )
    .expect("order 1 constant, order 1 reactants");
    first
        .evaluate(time, &[T::from_f64(2.0), T::ZERO], &mut derivative)
        .expect("dimensions match");
    let expected = [T::from_f64(-1.0), T::from_f64(1.0)];
    assert_eq!(derivative, expected, "{}", type_name::<T>());

    let second = ReactionNetwork::new(
        matrix(2, &[(0, 0, -2.0), (1, 0, 1.0)]),
        vec![(
            SecondOrderConstant::from_base(T::from_f64(0.25)).into(),
            vec![(0, 2)],
        )],
    )
    .expect("order 2 constant, order 2 reactants");
    second
        .evaluate(time, &[T::from_f64(3.0), T::ZERO], &mut derivative)
        .expect("dimensions match");
    let expected = [T::from_f64(-4.5), T::from_f64(2.25)];
    assert_eq!(derivative, expected, "{}", type_name::<T>());
}

#[test]
fn network_rate_evaluates_closed_forms_in_si_units_at_every_scalar() {
    network_derivatives::<f32>();
    network_derivatives::<f64>();
}

/// A coefficient whose order differs from its reactant order sum is refused
/// at construction, in both directions and with the index of the offending
/// reaction, so `evaluate` is only ever reached with consistent reactions.
fn network_rejects_wrong_orders<T: RealField>() {
    let nu = || matrix::<T>(1, &[(0, 0, -1.0), (0, 1, -1.0)]);
    let first = || -> RateConstant<T> { FirstOrderConstant::from_base(T::ONE).into() };
    let second = || -> RateConstant<T> { SecondOrderConstant::from_base(T::ONE).into() };
    let mismatch = |reaction, coefficient_order, reactant_order| {
        Some(InvalidNetwork::OrderMismatch {
            reaction,
            mismatch: RateOrderMismatch {
                coefficient_order,
                reactant_order,
            },
        })
    };

    let low = ReactionNetwork::new(
        nu(),
        vec![(second(), vec![(0, 1)]), (second(), vec![(0, 2)])],
    );
    assert_eq!(low.err(), mismatch(0, 2, 1), "{}", type_name::<T>());
    let high = ReactionNetwork::new(nu(), vec![(first(), vec![(0, 1)]), (first(), vec![(0, 2)])]);
    assert_eq!(high.err(), mismatch(1, 1, 2), "{}", type_name::<T>());
}

#[test]
fn network_rejects_a_rate_constant_of_the_wrong_order_at_every_scalar() {
    network_rejects_wrong_orders::<f32>();
    network_rejects_wrong_orders::<f64>();
}

/// A negative or non-finite coefficient is refused under its own variant. A
/// zero coefficient builds, and `evaluate` then yields a zero derivative at a
/// nonzero state.
fn network_rate_constant_values<T: RealField + Debug>() {
    let build = |value: f64| {
        ReactionNetwork::new(
            matrix::<T>(1, &[(0, 0, -1.0)]),
            vec![(
                FirstOrderConstant::from_base(T::from_f64(value)).into(),
                vec![(0, 1)],
            )],
        )
    };
    assert_eq!(
        build(-1.0).err(),
        Some(InvalidNetwork::NegativeRate { reaction: 0 }),
        "{}",
        type_name::<T>()
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            build(value).err(),
            Some(InvalidNetwork::NonFiniteRate { reaction: 0 }),
            "{} {value}",
            type_name::<T>()
        );
    }

    let network = build(0.0).expect("a zero coefficient is valid");
    let time = Instant::new(Time::from_base(T::ZERO)).expect("finite start");
    let mut derivative = [T::ONE];
    network
        .evaluate(time, &[T::from_f64(2.0)], &mut derivative)
        .expect("dimensions match");
    assert_eq!(derivative, [T::ZERO], "{}", type_name::<T>());
}

#[test]
fn network_rate_constant_value_errors_name_their_variant_at_every_scalar() {
    network_rate_constant_values::<f32>();
    network_rate_constant_values::<f64>();
}

/// The amount exponents at both ends of the admitted range, `-(2³¹-2)` and
/// `2³¹-1`, with the length exponent `-3·N` each fixes.
type LowestAmount = Sum<N2147483647, P1>;
type HighestAmount = P2147483647;
type AtLowest = Dimension<Prod<LowestAmount, N3>, Z0, N1, Z0, Z0, LowestAmount, Z0, BaseSemantics>;
type AtHighest =
    Dimension<Prod<HighestAmount, N3>, Z0, N1, Z0, Z0, HighestAmount, Z0, BaseSemantics>;

#[test]
fn the_order_check_holds_at_the_amount_exponent_range_edges() {
    // The greatest order drives a reaction of that order end to end.
    let k: Quantity<f64, AtLowest> = Quantity::from_base(2.0);
    let rate = mass_action_rate(k, &[(2_147_483_647, conc(1.0))]).expect("order i32::MAX");
    assert_exact(rate, 2.0);

    // The least order, `i32::MIN + 2`, is no reaction's order: it equals no sum.
    let k: Quantity<f64, AtHighest> = Quantity::from_base(2.0);
    assert_eq!(
        mass_action_rate(k, &[(1, conc(1.0))]).err(),
        Some(RateOrderMismatch {
            coefficient_order: i32::MIN + 2,
            reactant_order: 1,
        })
    );
}

#[test]
fn rate_order_mismatch_displays_both_orders_and_is_an_error() {
    let mismatch = RateOrderMismatch {
        coefficient_order: 1,
        reactant_order: 2,
    };
    let error: &dyn core::error::Error = &mismatch;
    assert_eq!(
        error.to_string(),
        "rate constant of order 1 cannot drive a reaction of order 2"
    );
}

#[test]
fn reaction_count_mismatch_displays_both_counts() {
    let count = |n_reactions, supplied| InvalidNetwork::ReactionCountMismatch {
        n_reactions,
        supplied,
    };
    assert_eq!(
        common::error_text(&count(3, 5)),
        "the reaction list has 5 entries but the stoichiometric matrix has 3 reactions"
    );
    assert_eq!(
        common::error_text(&count(7, 2)),
        "the reaction list has 2 entries but the stoichiometric matrix has 7 reactions"
    );
}

#[test]
fn reactant_out_of_range_displays_reaction_species_and_count() {
    let out_of_range = |reaction, species, n_species| InvalidNetwork::ReactantOutOfRange {
        reaction,
        species,
        n_species,
    };
    assert_eq!(
        common::error_text(&out_of_range(2, 9, 4)),
        "reaction 2 has a reactant on species 9 but the network declares 4 species"
    );
    assert_eq!(
        common::error_text(&out_of_range(6, 12, 8)),
        "reaction 6 has a reactant on species 12 but the network declares 8 species"
    );
}

#[test]
fn non_finite_rate_displays_its_reaction() {
    assert_eq!(
        common::error_text(&InvalidNetwork::NonFiniteRate { reaction: 6 }),
        "reaction 6 has a non-finite rate coefficient"
    );
    assert_eq!(
        common::error_text(&InvalidNetwork::NonFiniteRate { reaction: 10 }),
        "reaction 10 has a non-finite rate coefficient"
    );
}

#[test]
fn negative_rate_displays_its_reaction() {
    assert_eq!(
        common::error_text(&InvalidNetwork::NegativeRate { reaction: 8 }),
        "reaction 8 has a negative rate coefficient"
    );
    assert_eq!(
        common::error_text(&InvalidNetwork::NegativeRate { reaction: 13 }),
        "reaction 13 has a negative rate coefficient"
    );
}

#[test]
fn order_mismatch_displays_its_reaction_and_both_orders() {
    let order_mismatch =
        |reaction, coefficient_order, reactant_order| InvalidNetwork::OrderMismatch {
            reaction,
            mismatch: RateOrderMismatch {
                coefficient_order,
                reactant_order,
            },
        };
    assert_eq!(
        common::error_text(&order_mismatch(5, 1, 2)),
        "reaction 5: rate constant of order 1 cannot drive a reaction of order 2"
    );
    assert_eq!(
        common::error_text(&order_mismatch(11, 3, 4)),
        "reaction 11: rate constant of order 3 cannot drive a reaction of order 4"
    );
}
