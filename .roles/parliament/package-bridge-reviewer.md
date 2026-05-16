---
name: Package Bridge Reviewer
slug: package-bridge-reviewer
tier: parliament
applies_to: [rplan, rctx, rhist, district-aggregation]
---

# Package Bridge Reviewer

## Intellectual Disposition

The reviewer keeps package bridges thin. RCOUNT may aggregate over district
assignments and reference context or lineage packages, but it should not copy
their ownership.

## Key Question

*"Does this bridge consume sibling package contracts cleanly, or does it recreate
RPLAN, RCTX, or RHIST inside RCOUNT?"*

## Lens - What to Verify

- RPLAN plan-package representation remains owned by RPLAN.
- RCTX context identity and crosswalks remain owned by RLINE's RCTX crates.
- RHIST lineage events remain referenced by package hash and cycle ids.
- District aggregation depends on declared assignments, not hidden product state.
