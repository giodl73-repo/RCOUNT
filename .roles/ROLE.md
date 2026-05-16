# RCOUNT Role Index

RCOUNT owns reusable election-count package verification, audit replay, district
aggregation, and lineage bridge contracts. Use these roles when changing package
models, reconciliation checks, replay statistics, district aggregation, or
RPLAN/RCTX/RHIST integration surfaces.

## Parliament

| File | Role | Primary tension |
|---|---|---|
| `parliament/count-evidence-steward.md` | Count Evidence Steward | Portable count packages vs. election-workflow leakage |
| `parliament/audit-replay-auditor.md` | Audit Replay Auditor | Reproducible replay evidence vs. statistical handwaving |
| `parliament/package-bridge-reviewer.md` | Package Bridge Reviewer | RPLAN/RCTX/RHIST interoperability vs. duplicated ownership |

## Review order

1. Use Count Evidence Steward for count models, hashes, IO, and reconciliation checks.
2. Use Audit Replay Auditor for replay statistics, transcripts, and verifier behavior.
3. Use Package Bridge Reviewer for RPLAN, RCTX, RHIST, and district aggregation boundaries.
