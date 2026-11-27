# Prometheus gap audit

## Verified downstream provider migration

A new external Cargo project resolved the consuming package without a pre-existing lock or provider overrides. Its registry graph includes `horae-time 0.4.0`. The committed lock passes 47 executed tests covering the standard-feature integration path and the numerical reaction network (0 tests excluded or skipped in that run).

## Incremental release baseline

Reviewed [prometheus-kinetics 0.1.0](Cargo.toml) at [b98d17547cd3](https://github.com/ryancinsight/prometheus/tree/b98d17547cd33c78673993d37a682679817ba67a). The registry has no entry for this package and version. First publication requires a verified package graph and a clean consumer.

The next [patch] checkpoint uses [rate](tests/rate.rs). [Target version and acceptance criteria](backlog.md#prometheus-stable-qualification).

<a id="whole-sum-order-test-runtime"></a>
## Whole-sum order test runtime margin
- Risk: `order_sum_past_the_largest_total_is_a_mismatch` (`src/rate/tests.rs:161-175`) folds `2^32 + 2` orders; it is the least input that tells saturation from wrapping through `check_order` (derived: `ceil(2^64 / (2^32 - 1)) = 2^32 + 2`), so no smaller witness exists. Slow mark 30 s, termination 60 s, `retries = 0` (`.config/nextest.toml:5-6`).
- Evidence: hosted `PASS [ 20.328s ]` (run 45438701248, a68fd1b) and `PASS [ 20.227s ]` (run 46071701896, 2fa7f44), no SLOW line; local 18.131 s (lane, 2fa7f44). Hosted margin to the slow mark is a factor 1.48 over two samples.
- Re-open trigger: any nextest run marks the test SLOW or terminates it. The remedy is then a reviewed per-test override with a derived bound; never a smaller input and never removal, which reopens the fold-site wrapping hole.

<a id="rate-tests-64-bit-only"></a>
## The lib test target assumes a 64-bit usize
- Risk: `(1 << 32) + 1` is the `usize` repeat count at `src/rate/tests.rs:167`; with a 32-bit `usize` the shift overflows at compile time (rustc 1.97.0 rejects `let n: u32 = (1 << 32) + 1;` with `arithmetic_overflow`, deny by default, observed; the `usize` case is derived), so the `cfg(test)` lib target would not build there.
- Evidence: no target the repository supports is 32-bit (`git grep` of wasm, 32-bit, thumbv, i686, armv7, riscv32, target_pointer_width over README, docs, CI, manifests and boards at 2fa7f44: one unrelated hit, `src/rate/dimension.rs:133`); CI is ubuntu-latest only. A wasm32 test build already fails earlier in dev-dependencies (`getrandom` 0.4.3, `wait-timeout` 0.2.1; observed, 2fa7f44).
- Re-open trigger: a 32-bit target joins CI or the supported-target list; the remedy is `#[cfg(target_pointer_width = "64")]` on that test.
