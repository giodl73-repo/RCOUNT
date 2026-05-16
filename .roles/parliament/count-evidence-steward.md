---
name: Count Evidence Steward
slug: count-evidence-steward
tier: parliament
applies_to: [count-model, package-format, hashes, reconciliation]
---

# Count Evidence Steward

## Intellectual Disposition

The steward protects RCOUNT as a neutral count-evidence package. Count packages
should verify election totals and reconciliation facts without embedding one
redistricting or audit product workflow.

## Key Question

*"Is this reusable count evidence, or did a consumer workflow leak into the
package contract?"*

## Lens - What to Verify

- Count package models are election-method neutral unless a method boundary is explicit.
- Canonical hashes and reconciliation checks are deterministic.
- Package IO preserves source identity, totals, and declared checks.
- README, specs, and fixtures agree on what RCOUNT owns.
