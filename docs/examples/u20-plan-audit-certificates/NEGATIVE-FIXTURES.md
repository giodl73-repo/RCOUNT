# U.20 Negative Fixture Catalog

These commands document the failure modes covered by the public U.20 package
and the executable `rplan-cli` tests. Paths are written from the repository
root.

## Missing Context

Command:

```powershell
cargo run -p rplan-cli -- verify-certificate `
  --certificate docs/examples/u20-plan-audit-certificates/grid3x3-valid/audit-certificate.json `
  --plan docs/examples/u20-plan-audit-certificates/grid3x3-valid/plan.rplan
```

Expected failure class:
`certificate verification failed: certificate requires context hash ...`

Meaning:
The certificate was generated for a contextual package. A plan alone is not
enough to verify graph, population, source-hash, or unit-order claims.

## Plan Tamper

Fixture coverage:
`verify_certificate_rejects_tampered_plan_assignment` in
`crates/rplan-cli/tests/audit_cli.rs`.

Expected failure class:
`certificate plan hash mismatch`

Meaning:
The supplied plan assignment does not match the plan hash recorded in the
certificate.

## Context Hash Or Source Change

Fixture coverage:
`verify_certificate_rejects_tampered_context_hash` in
`crates/rplan-cli/tests/audit_cli.rs`.

Expected failure class:
`certificate context hash mismatch`

Meaning:
The supplied context is internally valid but not the context certified by the
certificate.

## Stale RCTX

Fixture coverage:
`verify_certificate_rejects_stale_context_with_same_source_hash` in
`crates/rplan-cli/tests/audit_cli.rs`.

Expected failure class:
`certificate context hash mismatch`

Meaning:
The source label can remain the same while context content changes. The
certificate binds to the full context hash, so stale context is rejected.

## Canonical Unit-Order Mismatch

Fixture coverage:
`verify_certificate_rejects_unit_order_mismatch` in
`crates/rplan-cli/tests/audit_cli.rs`.

Expected failure class:
`certificate context hash mismatch`

Meaning:
RPLAN assignments are positional. Reordering context units changes the context
hash and invalidates the package.

## Profile Mismatch

Fixture coverage:
`state_house_with_congressional_profile_exits_two` in
`crates/rplan-cli/tests/audit_cli.rs`.

Expected failure class:
`legal profile chamber ... does not match plan chamber ...`

Meaning:
Profile applicability is checked at audit-generation time. A certificate is
profile-scoped and should not be presented as evidence for a different chamber
or jurisdiction.

## Missing-Input Constraint

Command:

```powershell
cargo run -p rplan-cli -- audit `
  --plan docs/examples/u20-plan-audit-certificates/grid3x3-valid/plan.rplan `
  --context docs/examples/u20-plan-audit-certificates/grid3x3-valid/context.rctx `
  --constraints splits `
  --fixed-generated-at 2026-05-10T00:00:00Z `
  --format json
```

Expected failure class:
The certificate result is `fail`, and the `splits` check has status
`missing-input`.

Meaning:
Requested checks are explicit. If the context lacks required data, the verifier
reports missing input rather than silently treating the check as passed.

## Broken Lineage Or Reserved Lineage Fields

Fixture coverage:
`verify_rejects_certificate_with_reserved_lineage_extra` in
`crates/rplan-audit/src/lib.rs`.

Expected failure class:
`algorithm lineage extra uses reserved certificate field`

Meaning:
Algorithm-specific lineage metadata cannot override top-level certificate
semantics such as `plan_hash`, `source_hashes`, or `result`.
