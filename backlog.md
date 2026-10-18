# Prometheus backlog

Active target: 0.2.0. Checklist entries live in the item blocks, following the
current Atlas board schema; a separate checklist.md would add retired-board debt.

<a id="PROMETHEUS-FULL-ORDER-REVIEW"></a>
## PROMETHEUS-FULL-ORDER-REVIEW — Review complete u32 reaction orders [patch]

- Status: blocked
- Priority: correctness
- Basis: accepted u32::MAX at c=1+2^-31 returns about 2.718 with the old clamp;
  the independent native-rounding interval is (7,8) for rate and Jacobian/order.
- Scope: private power factorization; no accepted orders or error contracts change.
- [x] Preserve the original two red regressions and three passing controls.
- [x] Verify zero/one, signed-boundary neighbors, parity, duplicate terms,
  concentration errors, ordinary result bits and allocation-free public calls.
- [x] Pass debug/release 42-test suites, doctests, feature check, strict Clippy,
  Rustdoc, Rust 1.95 MSRV, patch-class SemVer comparison and existing cargo-deny.
- [x] Assess controlled timing: ordinary median ratios 0.974–1.113, maximum
  increase 0.75 ns/call in the bounded same-graph macOS probe.
- [ ] Assess that cost against the consuming application budget and obtain
  exact-head independent review.
- [ ] Assess hosted gates at the draft head before considering integration.
- Evidence: docs/reaction-order.md and the unchanged regression fixture in
  tests/reaction_order_regression.rs. No merge or publication in this repair pass.

<a id="PROMETHEUS-PROVIDER-RELEASE-SCOPE"></a>
## PROMETHEUS-PROVIDER-RELEASE-SCOPE — Reconcile provider and publication scope [major]

- Status: blocked
- Priority: release-scope
- Basis: current landed nominal provider identities require documented 0.2.0
  migration; existing PR #49 independently edits the old Aequitas minimum.
- [x] Resolve latest landed Eunomia 0.10, Aequitas 0.3, Leto 0.46 and Horae 0.2.
- [x] Record the migration and atomically bump the package to 0.2.0.
- [x] Assess the unchanged supply-chain policy and release workflow triggers.
- [ ] Reconcile the Cargo.toml overlap with #49 before merge; preserve its owner.
- [x] Assess dependency publication availability: local cargo package fails
  because horae-time is absent from the crates.io index.
- [ ] Resolve the existing Horae publication/authentication hold through its
  authorized owner before main advances; version bumps can publish and tag.
- [ ] Obtain exact-head independent review and all applicable lifecycle gates.
- Holds: no merge, tag, registry publication, service deployment, credential
  changes or resumption of held Atlas/member tasks is authorized by this draft.
