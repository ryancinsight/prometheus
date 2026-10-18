# Reaction-order audit

Baseline: 569a4bd5b9be68e4135e6a61355483eb48a067b7.

One reproduced P2 correctness defect: mass-action products clamp accepted u32
orders to i32::MAX, changing both rate and analytic Jacobian. Baseline 32 tests
pass; the added five-test witness has three passing controls and two failures in
both debug and release. The fixture is retained byte-for-byte in this repair.

The private repair decomposes high orders as 2q+r and retains the existing
provider operation for every signed order. Additional tests cover native f32/f64,
signed zero and parity, all transition neighbors, zero-concentration derivatives,
duplicate terms, concentration validation, and original shape/coefficient errors.
The runtime domain stays unchanged, including raw signed solver states. Actual
rate/Jacobian calls use zero heap allocations after network construction.

Current landed dependencies compile together; no downgrade or held provider
branch is used. All 42 native tests pass in debug and release. All repository
local correctness gates and the existing cargo-deny policy pass; informational
SemVer comparison passes 223 checks with 30 inapplicable skips. The provider
identity migration still receives an explicit Breaking entry and 0.2.0 bump.

Controlled timing records ordinary median ratios 0.974–1.113 and a maximum
0.75 ns/call increase; application-budget assessment remains open.
Hosted exact-head gates, independent review, manifest reconciliation
with PR #49, and automatic-publication scope remain tracked in backlog.md.
No source changes occur in the owner checkout or the original red-test packet.

Local package verification is blocked: the crates.io index has no horae-time
package. This is publication availability, not a source correctness failure.
No publication or authentication retry is attempted.

Independent d4f40e1 review finds no arithmetic failure and one P3 fixture gap:
the MAX transition reference used one unsigned-only factor. The repaired
reference uses [i32::MAX, i32::MAX, 1] and checks each factor and their sum.
The new structural assertion fails on the old split and passes on the repair.
Production code, providers, original witness, policies and limits stay unchanged.

The review's independent timing reaches +3.99146 ns/call on the identical
single-network harness, and +5.35708 ns on mixed signed-boundary stress. The
older +0.74417 ns sample is retained as one observation. Frozen caller assembly
and flags differ; docs/reaction-order.md records the bounded comparison without
claiming a sole cause or generalizing either result. Performance acceptance,
stack/RSS measurements, exact-head review renewal and all release holds remain.
