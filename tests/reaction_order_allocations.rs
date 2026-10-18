//! Count heap traffic around actual public solver calls after construction.
use aequitas::systems::si::quantities::Time;
use eunomia::RealField;
use horae::system::{ExplicitSystem, ImplicitSystem};
use horae::time::Instant;
use mnemosyne::counting::{AllocationDelta, CountingAllocator, measure};
use prometheus::{ReactionNetwork, StoichiometricMatrix};
use std::alloc::System;
use std::hint::black_box;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::new(System);

fn allocation_contract<T: RealField>() {
    let time = Instant::new(Time::from_base(T::ZERO)).expect("finite time");
    let m = u32::try_from(i32::MAX).expect("positive maximum");
    for order in [0, 1, 2, 3, 31, m, m + 1, u32::MAX] {
        let matrix =
            StoichiometricMatrix::try_from_entries(2, 1, [(0, 0, -T::ONE), (1, 0, T::ONE)])
                .expect("valid matrix");
        let network =
            ReactionNetwork::new(matrix, vec![(T::ONE, vec![(0, order)])]).expect("accepted order");
        let mut rate = [T::ZERO; 2];
        let mut jacobian = [T::ZERO; 4];
        let ((), delta) = measure(|| {
            for _ in 0..128 {
                network
                    .evaluate(time, black_box(&[T::ONE, T::ZERO]), &mut rate)
                    .expect("valid shape");
                network
                    .jacobian(time, black_box(&[T::ONE, T::ZERO]), &mut jacobian)
                    .expect("valid shape");
                black_box((&rate, &jacobian));
            }
        });
        assert_eq!(delta, AllocationDelta::default());
        assert_eq!(rate, [-T::ONE, T::ONE]);
        let derivative = T::from_integer(i64::from(order));
        assert_eq!(jacobian, [-derivative, T::ZERO, derivative, T::ZERO]);
    }
}
#[test]
fn solver_evaluation_and_jacobian_allocate_nothing() {
    allocation_contract::<f32>();
    allocation_contract::<f64>();
}
