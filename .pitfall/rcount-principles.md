# RCOUNT Principles

## RCOUNT-P-01: Evidence Package Neutrality

**Decision rule:** Keep count packages reusable across election methods and
consumers; method-specific behavior must use explicit versioned identifiers.

**Rationale:** RCOUNT should not smuggle BISECT, RPLAN, jurisdiction, or product
workflow assumptions into a reusable count-evidence contract.

**Test:** Package changes preserve the README boundary and family contract, and
role review asks whether consumer workflow leaked into the package contract.

**Evidence:** `README.md`, `docs/family-contract.md`, and
`.roles/parliament/count-evidence-steward.md`.

## RCOUNT-P-02: Replayable Audits Over Narrative Confidence

**Decision rule:** Audit results must be reproducible from package inputs,
method identifiers, samples, risk limits, manifests, transcripts, and
deterministic statistics.

**Rationale:** A transcript is useful only when another tool or reader can
replay the checked boundary and see why it passed, failed, or stopped.

**Test:** CLI verifier behavior matches library audit semantics for accepted
packages, rejected packages, and replay algorithm boundaries.

**Evidence:** `.roles/parliament/audit-replay-auditor.md` and
`crates/rcount-cli/tests/verify_cli.rs`.

## RCOUNT-P-03: Integrity And Privacy Fail Closed

**Decision rule:** Reject hash drift, missing source evidence, stale lineage,
unsupported versions, and choice-bearing proof records explicitly.

**Rationale:** Silent repair or reinterpretation would turn evidence packages
into unreviewable assertions.

**Test:** Negative fixtures for tampered source, missing source hash, bad
lineage, and choice-bearing proof fail verification.

**Evidence:** `docs/family-contract.md` and
`docs/examples/rcount-golden-packages/README.md`.

## RCOUNT-P-04: Bridges Consume Sibling Contracts Thinly

**Decision rule:** Aggregate over RPLAN assignments and refer to RCTX/RHIST
identity without copying their schemas, legal policy, or product ownership.

**Rationale:** Thin bridges let RCOUNT interoperate without becoming a second
RPLAN, RLINE, RHIST, or BISECT.

**Test:** District aggregation depends on declared assignments and hashes, not
hidden product state.

**Evidence:** `.roles/parliament/package-bridge-reviewer.md` and
`docs/specs/rcount-foundation.md`.

## RCOUNT-P-05: Negative Fixtures Are Product Evidence

**Decision rule:** Keep rejected packages in the public validation surface
whenever they encode source custody, privacy, lineage, or audit-boundary
failures.

**Rationale:** Negative packages prove RCOUNT catches the failures downstream
consumers are most likely to miss.

**Test:** Bad packages remain executable through CLI tests or documented
verifier commands.

**Evidence:** `docs/examples/rcount-golden-packages/README.md`, `README.md`,
and `crates/rcount-cli/tests/verify_cli.rs`.
