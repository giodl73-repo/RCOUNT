# RPLAN Golden Package Corpus

This directory contains tiny public packages that exercise the shared
RPLAN/RCTX/audit-certificate endpoint across the algorithm-family papers.

The packages use a deliberately small 3x3 fixture. They are not performance
benchmarks and do not show that an algorithm is superior. They show that each
family can land on the same verifier-facing package contract with lineage
metadata.

Package conventions are defined in
[`docs/file-formats/rplan-packages.md`](../../file-formats/rplan-packages.md).

## Packages

| Package | Producer | Method | Paper |
|---|---|---|---|
| `T.14+spectral-partitioning` | `bisect-apportion` | `spectral-partitioning` | T.14 |
| `T.15+capacity-constrained-clustering` | `bisect-clustering` | `capacity-constrained-clustering` | T.15 |
| `T.16+hierarchical-regionalization` | `bisect-clustering` | `hierarchical-regionalization` | T.16 |
| `T.17+flow-construction` | `bisect-flow` | `flow-construction` | T.17 |
| `U.16+branch-and-cut` | `bisect-ilp` | `branch-and-cut` | U.16 |
| `U.17+branch-and-price` | `bisect-column` | `branch-and-price` | U.17 |
| `U.18+local-search-improvement` | `bisect-local-search` | `one-move-improvement` | U.18 |
| `U.19+selected-frontier` | `bisect-pareto` | `selected-frontier-export` | U.19 |

The U.20 reference package lives in
[`../u20-plan-audit-certificates/grid3x3-valid`](../u20-plan-audit-certificates/grid3x3-valid).

## Verify A Package

```powershell
cargo run -p rplan-cli -- verify-certificate `
  --certificate docs/examples/rplan-golden-packages/U.18+local-search-improvement/audit-certificate.json `
  --plan docs/examples/rplan-golden-packages/U.18+local-search-improvement/plan.rplan `
  --context docs/examples/rplan-golden-packages/U.18+local-search-improvement/context.rctx
```

The same package can be checked through the BISECT bridge:

```powershell
cargo run -p bisect-cli -- verify `
  --manifest docs/examples/rplan-golden-packages/U.18+local-search-improvement/manifest.json
```

Every package in this directory has:

- `plan.rplan`
- `context.rctx`
- `audit-certificate.json`
- `manifest.json`
- `method-transcript.json`

## Claim Boundary

These packages prove that the final artifact bundle is coherent:

- the certificate content hash verifies
- the certificate binds to the supplied plan and context hashes
- source hashes and context hashes agree
- algorithm lineage is present and cannot override reserved certificate fields

They do not prove large-instance quality, legal sufficiency, fairness,
optimality, or policy preference.
