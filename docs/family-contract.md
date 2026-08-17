# RCOUNT family contract

RCOUNT is the Election Systems owner for portable election-count evidence,
package verification, reconciliation, statistical audit replay, and optional
district aggregation. This contract lets BISECT and future civic systems adopt
those capabilities without inheriting BISECT application policy.

RCOUNT is pre-1.0. Its current package version is `0.1-draft`, but versioned
records, hash domains, method identifiers, and verification meanings are
compatibility commitments rather than incidental implementation details.

## Owned contract

RCOUNT owns:

- the public APIs and error meanings of `rcount-core`, `rcount-stats`,
  `rcount-io`, `rcount-audit`, `rcount-district`, and `rcount-rhist`;
- the RCOUNT package directory, manifest, normalized records, source index,
  reconciliation declarations, proof records, status events, audit records,
  and verification transcripts;
- `RCOUNT_VERSION`, `rcount-audit-transcript-v1`,
  `RCOUNT_DISTRICT_AGGREGATION_VERSION`, and stable audit method identifiers;
- domain-separated `RCOUNT_*_V1` source, record, file, package, event, proof,
  RLA-manifest, and RLA-sample hashes;
- deterministic canonicalization, package-content hashes, source-byte hashes,
  package verification, and reconciliation equation meanings;
- `pass`, `fail`, `boundary`, `continue`, `stop`, and `escalate` meanings where
  exposed by verification or replay records;
- count lifecycle, batch, lineage, CVR, manual-audit, comparison-audit, and RLA
  validation;
- privacy gates that reject choice-bearing or voter-linkable proof records;
- package readers and writers plus statement CSV, NIST CDF, and selected
  jurisdiction adapters;
- RPLAN/RCTX district aggregation transcripts and RHIST lineage projections.

RCOUNT does not certify an election, replace canvassing officials, approve a
district plan, define RPLAN or RLINE schemas, publish product reports, or own
BISECT generation, map, research, or legal-policy workflows.

## Compatibility and deprecation

- Additive APIs and optional record fields may remain in the current `0.y`
  line when old packages and consumers retain their meanings.
- Breaking API, package layout, required-field, default, canonicalization,
  hash-input, equation, status, method, privacy, or replay changes require a
  minor-version bump while pre-1.0.
- Existing version or method identifiers must never acquire incompatible
  semantics. Introduce a new identifier and retain an explicit reader or
  migration path.
- Deprecate public APIs and package fields before removal. Migration notes must
  identify replacement APIs, affected package records, hash consequences, and
  affected consumers.
- Readers reject unsupported versions and integrity drift explicitly; they do
  not silently reinterpret evidence.
- Source files referenced by hashes are byte contracts. Repository attributes
  keep retained source evidence LF-normalized across platforms.
- Downstream repositories should pin commits. Branch updates require the
  foundation and consumer rehearsals below.

## Foundation gate

From RCOUNT:

```powershell
cargo test --workspace
```

The gate covers accepted and rejected package fixtures, deterministic hashes,
source custody, reconciliation, lifecycle transitions, privacy, audit methods,
transcripts, CLI exit behavior, district aggregation, and lineage projection.

## BISECT adoption and migration

BISECT is the first family consumer. It currently retains an embedded RCOUNT
crate snapshot while extraction and dependency admission remain separate
decisions. Its package specification register and R package-family map point to
this document as the upstream compatibility authority.

For an RCOUNT change:

1. Run the RCOUNT foundation gate.
2. Identify every changed public item, package field, hash input, equation,
   status, or method identifier.
3. Compare the affected `rcount-*` crates with BISECT's embedded snapshot.
4. Port the change intentionally or record why BISECT remains on the older
   contract.
5. Run BISECT's affected RCOUNT crate tests and VTRACE package checks before
   admitting the snapshot update.

Moving BISECT to a pinned RCOUNT dependency is compatible only when the same
package fixtures, hashes, verification outcomes, and product boundaries remain
stable. The dependency migration must not move BISECT-owned policy into RCOUNT.
