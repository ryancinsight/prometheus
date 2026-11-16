//! The dimension a mass-action rate constant must have for its reaction order.

use core::ops::Mul;

use aequitas::dimension::{BaseSemantics, Dimension};
use typenum::{Integer, IsGreater, IsLessOrEqual, N1, N3, N2147483647, P2147483647, True, Z0};

/// A dimension that is the rate constant of a mass-action reaction of a
/// definite overall order.
///
/// Implemented for exactly the dimensions `mol^(1-n)·m^(3n-3)·s⁻¹`, that is
/// the aequitas dimension with amount exponent `N`, length exponent `-3·N`,
/// time exponent `-1`, no mass, current, temperature or luminosity exponent,
/// and the base semantics marker. The overall order is `1 - N`. Every other
/// dimension, a temperature or a `MolarConcentration` included, has no
/// implementation, so it is rejected at compile time.
///
/// The trait is sealed: its supertrait lives in a private module, so no type
/// outside this crate can implement it and declare an order its dimension
/// does not have ([ADR 0002](https://github.com/ryancinsight/prometheus/blob/main/docs/adr/0002-rate-constant-typing.md)).
///
/// ```compile_fail,E0277
/// # use prometheus::rate::RateConstantDimension;
/// struct Local;
/// impl RateConstantDimension for Local {
///     const ORDER: i32 = 2;
/// }
/// ```
///
/// `N` lies in `-(2³¹-2) ..= 2³¹-1`, which is the range where both `N` and
/// `1 - N` are `i32` values; an exponent outside it has no implementation
/// instead of being read through a truncating conversion. Orders from `2³¹`
/// upward are therefore not expressible as rate-constant dimensions.
///
/// The second-order constant is read. Each example after it varies what its
/// caption names, from one exponent to the range edges and the semantics
/// marker, and is rejected:
///
/// ```
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P3, Z0};
/// type SecondOrder = Dimension<P3, Z0, N1, Z0, Z0, N1, Z0, BaseSemantics>;
/// assert_eq!(<SecondOrder as RateConstantDimension>::ORDER, 2);
/// ```
///
/// Length `m²` where order 2 needs `m³`:
///
/// ```compile_fail,E0271
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P2, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P2, Z0, N1, Z0, Z0, N1, Z0, BaseSemantics>>();
/// ```
///
/// A mass, current, temperature or luminosity exponent of one:
///
/// ```compile_fail,E0277
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P1, P3, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P3, P1, N1, Z0, Z0, N1, Z0, BaseSemantics>>();
/// ```
///
/// ```compile_fail,E0277
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P1, P3, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P3, Z0, N1, P1, Z0, N1, Z0, BaseSemantics>>();
/// ```
///
/// ```compile_fail,E0277
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P1, P3, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P3, Z0, N1, Z0, P1, N1, Z0, BaseSemantics>>();
/// ```
///
/// ```compile_fail,E0277
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P1, P3, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P3, Z0, N1, Z0, Z0, N1, P1, BaseSemantics>>();
/// ```
///
/// Time exponent `-2` instead of `-1`:
///
/// ```compile_fail,E0277
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, N2, P3, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P3, Z0, N2, Z0, Z0, N1, Z0, BaseSemantics>>();
/// ```
///
/// Right exponents under another semantics marker:
///
/// ```compile_fail,E0277
/// # use aequitas::dimension::{Dimension, MolarConcentrationSemantics};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, P3, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// needs::<Dimension<P3, Z0, N1, Z0, Z0, N1, Z0, MolarConcentrationSemantics>>();
/// ```
///
/// An amount exponent of `-(2³¹-1)`, one below the range, and of `2³¹`, one
/// above it, each with the length exponent `-3·N` that goes with it, so amount
/// and length both differ from the second-order constant:
///
/// ```compile_fail,E0271
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, N2147483647, N3, Prod, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// type Amount = N2147483647;
/// needs::<Dimension<Prod<Amount, N3>, Z0, N1, Z0, Z0, Amount, Z0, BaseSemantics>>();
/// ```
///
/// ```compile_fail,E0271
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, N3, P2147483648, Prod, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// type Amount = P2147483648;
/// needs::<Dimension<Prod<Amount, N3>, Z0, N1, Z0, Z0, Amount, Z0, BaseSemantics>>();
/// ```
///
/// An amount exponent of `-(2³²+2)`, whose low 32 bits read as `-2`, with its
/// matching length exponent:
///
/// ```compile_fail,E0271
/// # use aequitas::dimension::{BaseSemantics, Dimension};
/// # use prometheus::rate::RateConstantDimension;
/// # use typenum::{N1, N3, Negate, PInt, Prod, Shleft, Sum, U1, U2, U32, Z0};
/// # fn needs<D: RateConstantDimension>() {}
/// type Amount = Negate<PInt<Sum<Shleft<U1, U32>, U2>>>;
/// needs::<Dimension<Prod<Amount, N3>, Z0, N1, Z0, Z0, Amount, Z0, BaseSemantics>>();
/// ```
pub trait RateConstantDimension: sealed::Sealed {
    /// Overall reaction order whose rate constant has this dimension: `1 - N`
    /// for amount exponent `N`, which lies in `-(2³¹-2) ..= 2³¹-1`, so the
    /// order runs from `i32::MIN + 2` to `i32::MAX`. It is negative when the
    /// amount exponent exceeds one, an order no reaction has.
    const ORDER: i32;
}

/// The supertrait that closes [`RateConstantDimension`]: the module is private,
/// so only this crate can name `Sealed` and write an implementation of it.
mod sealed {
    pub trait Sealed {}
}

/// Exclusive lower bound of the amount exponent: `1 - N` stays below `2³¹`.
type AmountExponentFloor = N2147483647;
/// Inclusive upper bound of the amount exponent: the greatest `i32`.
type AmountExponentCeiling = P2147483647;

impl<L, N> sealed::Sealed for Dimension<L, Z0, N1, Z0, Z0, N, Z0, BaseSemantics>
where
    L: Integer,
    N: Integer
        + Mul<N3, Output = L>
        + IsGreater<AmountExponentFloor, Output = True>
        + IsLessOrEqual<AmountExponentCeiling, Output = True>,
{
}

impl<L, N> RateConstantDimension for Dimension<L, Z0, N1, Z0, Z0, N, Z0, BaseSemantics>
where
    L: Integer,
    N: Integer
        + Mul<N3, Output = L>
        + IsGreater<AmountExponentFloor, Output = True>
        + IsLessOrEqual<AmountExponentCeiling, Output = True>,
{
    const ORDER: i32 = 1 - N::I32;
}

#[cfg(test)]
mod tests {
    use aequitas::Quantity;
    use aequitas::dimension::{BaseSemantics, Dimension};
    use aequitas::systems::si::quantities::ReactionRate;
    use typenum::{N1, N3, N2147483647, P1, P2147483647, Prod, Sum, Z0};

    use super::RateConstantDimension;
    use crate::rate::{FirstOrderConstant, SecondOrderConstant, ThirdOrderConstant};

    /// The amount exponents at both ends of the admitted range, `-(2³¹-2)` and
    /// `2³¹-1`, with the length exponent `-3·N` each fixes.
    type LowestAmount = Sum<N2147483647, P1>;
    type HighestAmount = P2147483647;
    type AtLowest =
        Dimension<Prod<LowestAmount, N3>, Z0, N1, Z0, Z0, LowestAmount, Z0, BaseSemantics>;
    type AtHighest =
        Dimension<Prod<HighestAmount, N3>, Z0, N1, Z0, Z0, HighestAmount, Z0, BaseSemantics>;

    #[test]
    fn order_is_read_from_the_dimension() {
        fn order<D: RateConstantDimension>(_: Quantity<f64, D>) -> i32 {
            D::ORDER
        }
        assert_eq!(order(ReactionRate::from_base(1.0)), 0);
        assert_eq!(order(FirstOrderConstant::from_base(1.0)), 1);
        assert_eq!(order(SecondOrderConstant::from_base(1.0)), 2);
        assert_eq!(order(ThirdOrderConstant::from_base(1.0)), 3);
    }

    #[test]
    fn order_is_exact_at_the_amount_exponent_range_edges() {
        // `1 - N` is `i32::MAX` at the lowest exponent and `i32::MIN + 2` at
        // the highest; `i32` holds both, so no bit of either is dropped.
        assert_eq!(<AtLowest as RateConstantDimension>::ORDER, i32::MAX);
        assert_eq!(<AtHighest as RateConstantDimension>::ORDER, i32::MIN + 2);
    }
}
