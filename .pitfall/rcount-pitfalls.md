# RCOUNT Pitfalls

## RCOUNT-PF-01: Verification Transcript Becomes Certification Claim

**Status:** OPEN

**Pattern:** A passing RCOUNT transcript is treated as legal election
certification, canvassing approval, plan approval, or election-night reporting.

**Domain:** Customer-facing docs, downstream civic systems, and public summaries.

**Detection difficulty:** Pass/fail checks, transcript hashes, audit statuses,
and package evidence look authoritative even when they are only evidence-layer
results.

**Structural solution:** Keep customer-facing language scoped to evidence
verification and route legal, canvassing, plan approval, and reporting claims to
consumer-owned authorities.

**Evidence:** `README.md`, `docs/family-contract.md`, and this PITFALL adoption
pass; open until customer-facing adoption docs receive the same review.

## RCOUNT-PF-02: Audit Method Label Hides Replay Boundary

**Status:** MITIGATED

**Pattern:** RLA, Colorado RLA, California RLA, Minerva, Athena, RAIRE, SOBA, or
similar method labels are read as broad statistical endorsement rather than a
declared replay boundary.

**Domain:** Audit replay, jurisdiction adapters, and verifier transcripts.

**Detection difficulty:** Method names are familiar and can sound conclusive
without the package inputs, samples, risk limits, and boundary statuses.

**Structural solution:** Keep method identifiers versioned and require replay
tests that show pass, boundary, and fail behavior.

**Evidence:** `crates/rcount-cli/tests/verify_cli.rs`,
`docs/examples/rcount-golden-packages/README.md`, and
`cargo run --bin rcount -- verify docs/examples/rcount-golden-packages/bad-rla-statistical`
failing at `rla_stopping_rule` on 2026-08-24.

## RCOUNT-PF-03: Hashes Drift While Arithmetic Still Passes

**Status:** MITIGATED

**Pattern:** A source file, manifest, package content hash, or source index
changes while contest totals and reconciliation arithmetic still pass.

**Domain:** Source custody, manifests, source indexes, and package verification.

**Detection difficulty:** Arithmetic pass results can hide custody drift unless
hash checks remain first-class.

**Structural solution:** Treat source bytes as byte contracts and reject
tampered, missing, or mismatched source evidence.

**Evidence:** `docs/family-contract.md`,
`docs/examples/rcount-golden-packages/README.md`,
`crates/rcount-cli/tests/verify_cli.rs`, and
`cargo run --bin rcount -- verify docs/examples/rcount-golden-packages/summary-basic`
passing on 2026-08-24.

## RCOUNT-PF-04: Thin Bridge Copies Sibling Ownership

**Status:** OPEN

**Pattern:** RPLAN/RCTX/RHIST integration turns into copied schemas, hidden
product state, or BISECT-specific policy inside RCOUNT.

**Domain:** District aggregation, lineage projection, and BISECT adoption.

**Detection difficulty:** The bridge is useful enough that duplicated ownership
can look like convenience rather than contract drift.

**Structural solution:** Keep bridges hash-based and role-reviewed; require
BISECT migration evidence before replacing its embedded RCOUNT snapshot.

**Evidence:** `.roles/parliament/package-bridge-reviewer.md`,
`docs/specs/rcount-foundation.md`, `docs/family-contract.md`, and
`cargo run --example district_aggregation_rplan -p rcount-district` passing on
2026-08-24.

## RCOUNT-PF-05: Negative Fixtures Treated As Optional Examples

**Status:** MITIGATED

**Pattern:** Rejected packages are demoted to prose examples instead of kept as
executable evidence for privacy, custody, lineage, and audit-boundary failures.

**Domain:** Golden package corpus, CLI tests, and downstream verifier adoption.

**Detection difficulty:** Positive verifier demos are easier to present, while
bad packages can look like tutorial clutter.

**Structural solution:** Keep negative fixtures listed, executable, and tied to
expected failure classes.

**Evidence:** `docs/examples/rcount-golden-packages/README.md`,
`crates/rcount-cli/tests/verify_cli.rs`, and `cargo test --quiet` passing on
2026-08-24.
