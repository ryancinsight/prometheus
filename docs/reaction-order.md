# Full reaction-order contract and provider migration

Target: 0.2.0. The correctness repair is [patch]; the public provider identity
migration is [major] under the documented pre-1.0 minor-version convention.

## Contract and decision

`Order = u32` and the network constructor accept every order. A rate uses the
entire exponent; the analytic Jacobian uses its exact predecessor and the
original order multiplier. Zero order gives one, including at zero concentration.
Differentiation forms products without concentration division.

For n <= i32::MAX, call the existing RealField::powi operation unchanged.
Otherwise n = 2q + r, q = n >> 1 <= i32::MAX, r in {0,1}; evaluate p = c^q,
then p*p, followed by multiplication by c when r=1. All arithmetic stays in T.
The current provider uses binary exponentiation: at most 31 iterations and
62 multiplications for q, then two products. Storage is constant and no
allocation occurs. No loop depends linearly on the order. The integer cast
is exact metadata conversion; no scalar precision changes.

Clamping changes the polynomial and its derivative. Rejecting high orders
would narrow the existing public domain. A second local exponentiation engine
would duplicate the provider. The selected factorization extends its domain
and preserves the ordinary multiplication sequence and its result bits.
Overflow/underflow and NaN propagate through native arithmetic; no new errors
or saturation are introduced. Negative raw solver states remain admissible
polynomial inputs. Validated Concentration separately rejects negatives and
nonfinite values; network evaluation checks shapes, and construction checks
coefficients and species indices. This change does not merge those contracts.

## Numerical verification

For c=1+2^-31 and n=u32::MAX, independent rational/binomial bounds put both
c^n and c^(n-1) strictly between 7 and 8. The first seven binomial terms have
a lower bound above 7; e^2 < 8 is an upper bound. The multiplication-tree
rounding bound gamma_(n+8) < 2^-20 retains both strict margins in f64; all
intermediates are finite normal. The saved independent oracle and the original
five public-network regressions predate this implementation. The old clamp
returns approximately 2.718 rather than a value in (7,8).

Tests retain those assertions and add signed-boundary neighbors, parity,
signed zero, duplicate reactants, zero/one derivative cases, ordinary result
bits, input/error contracts, and allocation counts for actual evaluate/Jacobian
calls. Timing compares old and new kernels with the same current provider graph;
old high-order timings are not a performance-equivalence claim.

## Provider migration and ownership

Use landed Eunomia 0.10, Aequitas 0.3, Leto 0.46 and Horae 0.2; the lockfile
records current landed Mnemosyne 0.10, Melinoe 0.10 and Themis 0.11.2. No
unmerged provider branch or downgrade participates. These nominal public
Quantity/RealField/System identities change, so 0.2.0 has an explicit Breaking
migration entry even though Prometheus item names remain unchanged.

Open PR #49 changes only the older Aequitas minimum from 0.2.0 to 0.2.1.
The 0.3 minimum preserves that intent but overlaps its manifest edit. That PR
and its owner's checkout remain untouched; reconcile the overlap before merge.
This draft does not authorize merging, tagging, publishing, service deployment,
credential changes, or resuming any held work. Exact-head independent review,
CI, supply-chain policy and publication-scope assessment remain lifecycle gates.

## Controlled timing assessment

On this aarch64 macOS host, release builds use identical current manifests and
lockfiles. Nine alternating AB/BA rounds each time 200,000 real network calls
per scalar/order/operation. The selected cold, non-inlined high-order helper
keeps the uncommon extension out of the ordinary provider loop; a shared-call
layout did not improve the measurements. The original ordinary-path arithmetic
and bit results remain identical.

For orders 0,1,2,3,31 and i32::MAX, candidate/baseline median ratios range from
0.974 to 1.113, with absolute increases no greater than 0.75 ns/call in this
probe. This is a measured cost, not a zero-overhead claim or an application
performance budget. High-order ratios range from 0.998 to 1.147; their old
baseline computes the wrong polynomial, so these are timings only. Repeated
public rate/Jacobian calls allocate no memory in either native scalar type.
Review the measured ordinary-path cost against the consuming application
budget before merge. No test, threshold, timeout or gate is relaxed.

`cargo package --locked --allow-dirty` cannot complete because the crates.io
index has no horae-time package. The existing provider publication/authentication
hold is unchanged. No registry publication or authentication retry occurs.

## Independent review and frozen caller contexts

Independent review of d4f40e1 reproduces the original two failures on baseline
and five passes on the repair. It reports no arithmetic defect across its
20,764-case four-build campaign, 12,225 ordinary bit-identical rows, and 3,327
analytical comparisons. Its one P3 finding concerns the new transition test:
the MAX reference split included i32::MAX+1. That reference now uses
[i32::MAX, i32::MAX, 1], with assertions that every factor fits the signed
domain and their u64 metadata sum equals the requested order. The original
five-test witness remains unchanged. Exact-head review renewal is still needed.

Both timing samples are retained as observations. The author's frozen Cargo
release probe reports maximum ordinary median increase +0.74417 ns/call.
The reviewer compiles the byte-identical harness with direct rustc O3 and
explicit overflow checks, linked to independently built O3 libraries from the
same locked provider cohort; it reports +3.99146 ns/call (f32 signed-boundary
Jacobian: 18.70208 -> 22.69354 ns). Its mixed ordinary hot/stream Jacobians
increase by 1.61–3.80 ns; mixed signed-boundary stress increases by up to
5.35708 ns. Neither sample is a universal cap, and neither is discarded.

Bounded disassembly of the four frozen binaries locates the timed caller by
Mach-O function starts and the 200,000-call counters. Author baseline/current
callers have 1,219/1,264 static instructions and 115/120 conditional branches;
reviewer baseline/current have 1,259/1,305 and 123/128. Both current callers
retain the ordinary power loop and a branch to the cold high-order helper.
The reviewer caller contains six umulh instructions versus two in the author
caller, consistent with extra integer-overflow checking; their code layout,
metadata and compilation boundaries also differ. These are whole-caller static
counts, not executed-instruction or cycle estimates. No experiment isolates one
cause for the timing discrepancy, so no sole-cause attribution is made.

The later forced-inlining-only experiment produced byte-identical binaries;
it cannot establish a speedup or explain away the independent baseline/current
cost. Application performance acceptance remains held. Zero measured heap
traffic and unchanged 112-byte network layout do not measure peak stack, total
callee stack, RSS or consuming-solver cost; those remain unmeasured.
