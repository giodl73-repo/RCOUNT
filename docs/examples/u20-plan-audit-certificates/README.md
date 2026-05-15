# U.20 Plan Audit Certificate Examples

This directory contains small public examples for the U.20 fixed-point contract.
They are intentionally tiny so the verification behavior is inspectable.

Package conventions are defined in
[`docs/file-formats/rplan-packages.md`](../../file-formats/rplan-packages.md).

## Valid Package

`grid3x3-valid/` contains:

- `plan.rplan`: a three-district assignment over a 3x3 fixture graph.
- `context.rctx`: the graph, population series, canonical unit order, and source hashes.
- `audit-certificate.json`: a profile-scoped certificate generated for the plan/context pair.
- `manifest.json`: file inventory and SHA-256 hashes for the package.

Verify it with:

```powershell
cargo run -p rplan-cli -- verify-certificate `
  --certificate docs/examples/u20-plan-audit-certificates/grid3x3-valid/audit-certificate.json `
  --plan docs/examples/u20-plan-audit-certificates/grid3x3-valid/plan.rplan `
  --context docs/examples/u20-plan-audit-certificates/grid3x3-valid/context.rctx
```

## Negative Fixtures Covered By Tests

The executable negative fixtures live in `crates/rplan-cli/tests/audit_cli.rs`.
The failure catalog is summarized in
[`NEGATIVE-FIXTURES.md`](NEGATIVE-FIXTURES.md).
They cover the U.20 failure catalog:

- missing context for a contextual certificate
- tampered plan assignment
- changed context/source hash
- canonical unit-order mismatch
- stale context with unchanged source label
- profile mismatch at audit-generation time
- missing-input constraint status

Those failures are certificate-verification failures, not broad legal judgments.
The certificate verifies only the declared profile, package hashes, context, and
lineage semantics.
