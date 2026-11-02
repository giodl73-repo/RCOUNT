# RCOUNT Invariants

## RCOUNT-INV-01: Public Identifiers Preserve Semantics

**Status:** MITIGATED

**Claim:** Version strings, hash domains, method identifiers, equation ids,
status values, and transcript meanings do not acquire incompatible semantics in
place.

**Why it matters:** Downstream repos and published packages depend on those
identifiers as compatibility commitments, not incidental implementation names.

**Enforcement:** Breaking version, hash-input, equation, status, method, or
replay changes require new identifiers or explicit migration paths.

**Evidence:** `docs/family-contract.md`.

## RCOUNT-INV-02: Source And Package Hashes Bind Verification

**Status:** MITIGATED

**Claim:** Verification binds raw source bytes, package content, manifests,
proof records, audit records, and declared source indexes through deterministic
hashes.

**Why it matters:** Election totals can still add up after source evidence has
drifted, so arithmetic checks alone are insufficient.

**Enforcement:** Tampered-source and missing-source-hash fixtures must fail
through `source_hash_match`.

**Evidence:** `docs/examples/rcount-golden-packages/README.md` and
`crates/rcount-cli/tests/verify_cli.rs`.

## RCOUNT-INV-03: Privacy Gates Reject Choice-Bearing Proof

**Status:** MITIGATED

**Claim:** Proof records may establish inclusion or integrity, but they do not
expose voter-linkable choices.

**Why it matters:** Receipt-like proof can undermine voter privacy even when
the rest of the package verifies.

**Enforcement:** The privacy gate accepts receipt-safe inclusion sketches and
rejects choice-bearing proof fixtures.

**Evidence:** `docs/family-contract.md`,
`docs/examples/rcount-golden-packages/README.md`, and
`crates/rcount-cli/tests/verify_cli.rs`.

## RCOUNT-INV-04: Audit Transcripts Match Verifier Semantics

**Status:** MITIGATED

**Claim:** CLI transcripts and library replay results agree on pass, fail,
boundary, continue, stop, escalate, and failure-cause meanings.

**Why it matters:** A transcript that disagrees with verifier behavior cannot be
used as portable evidence.

**Enforcement:** Tests compare CLI outputs, transcript fields, replay statuses,
and exit behavior.

**Evidence:** `.roles/parliament/audit-replay-auditor.md` and
`cargo test --quiet` on 2026-08-24.

## RCOUNT-INV-05: District Aggregation Is Not Plan Approval

**Status:** PARTIAL

**Claim:** District aggregation reports totals by declared assignment and
package, plan, context, and crosswalk hashes; it does not approve district
plans, legal profiles, fairness, or product reports.

**Why it matters:** Aggregated count evidence can look like plan certification
when reused in customer-facing or downstream election workflows.

**Enforcement:** Family contract and bridge role review keep plan approval and
legal-policy workflow outside RCOUNT.

**Evidence:** `docs/family-contract.md`,
`.roles/parliament/package-bridge-reviewer.md`, and `RCOUNT-PF-04`.
