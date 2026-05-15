# RCOUNT Foundation Spec

## Goal

Extract the reusable election-count package family from BISECT into a neutral
workspace for package verification, audit replay, and optional district
aggregation.

## Initial crates

| Crate | Role |
|-------|------|
| `rcount-core` | count package model, canonical hashes, reconciliation checks |
| `rcount-stats` | deterministic statistics for audit replay |
| `rcount-io` | package directory read/write |
| `rcount-audit` | verification and audit transcript generation |
| `rcount-district` | RPLAN/RCTX district aggregation |
| `rcount-cli` | standalone verifier CLI |
| `rcount-rhist` | optional RHIST lineage bridge |

## Boundary

RCOUNT owns count evidence and audit replay primitives. It does not own RPLAN
plan packages, RLINE graph/context kernels, PROOF/CROP evidence publishing, or
BISECT redistricting workflows.

## Dependency plan

1. Depend on sibling RPLAN for plan package types.
2. Depend on sibling RLINE for `rctx-core` and `rhist-core`.
3. Later switch sibling paths to git dependencies after the consumer migration
   pulse lands.

