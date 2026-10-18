//! Native unsigned-power boundaries and ordinary-path continuity.
use super::reaction_power;
use eunomia::RealField;

fn boundaries<T: RealField>(bits: fn(T) -> u64) {
    let m = u32::try_from(i32::MAX).expect("positive signed maximum");
    for order in [0, 1, 2, 3, m - 1, m, m + 1, m + 2, u32::MAX - 1, u32::MAX] {
        assert_eq!(reaction_power(T::ONE, order), T::ONE);
        assert_eq!(
            reaction_power(-T::ONE, order),
            if order & 1 == 0 { T::ONE } else { -T::ONE }
        );
        let expected = if order == 0 { T::ONE } else { T::ZERO };
        assert_eq!(reaction_power(T::ZERO, order), expected);
        let expected = if order == 0 {
            T::ONE
        } else if order & 1 == 0 {
            T::ZERO
        } else {
            -T::ZERO
        };
        assert_eq!(bits(reaction_power(-T::ZERO, order)), bits(expected));
    }
    for raw in -16..=16 {
        let base = T::from_integer(raw) / T::from_integer(8);
        for order in [0, 1, 2, 3, 4, 7, 31, 63, 127, 1024, m] {
            let signed = i32::try_from(order).expect("ordinary exponent");
            assert_eq!(
                bits(reaction_power(base, order)),
                bits(T::powi(base, signed))
            );
        }
    }
}
#[test]
fn full_order_boundaries_and_ordinary_bits() {
    boundaries::<f32>(|x| u64::from(x.to_bits()));
    boundaries::<f64>(f64::to_bits);
}

fn range_behavior<T: RealField>(nan: T, infinity: T, bits: fn(T) -> u64) {
    let m = u32::try_from(i32::MAX).expect("positive maximum");
    for order in [m + 1, m + 2, u32::MAX - 1, u32::MAX] {
        assert!(reaction_power(nan, order).is_nan());
        assert_eq!(bits(reaction_power(infinity, order)), bits(infinity));
        assert_eq!(
            bits(reaction_power(-infinity, order)),
            bits(if order & 1 == 0 { infinity } else { -infinity })
        );
        assert_eq!(
            bits(reaction_power(T::from_integer(2), order)),
            bits(infinity)
        );
        assert_eq!(
            bits(reaction_power(T::ONE / T::from_integer(2), order)),
            bits(T::ZERO)
        );
    }
    assert_eq!(bits(reaction_power(nan, 0)), bits(T::ONE));
    assert_eq!(bits(reaction_power(infinity, 0)), bits(T::ONE));
}
#[test]
fn full_orders_preserve_native_nonfinite_and_range_behavior() {
    range_behavior(f32::NAN, f32::INFINITY, |x| u64::from(x.to_bits()));
    range_behavior(f64::NAN, f64::INFINITY, f64::to_bits);
}
