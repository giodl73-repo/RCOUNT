# RCOUNT

**Reusable election-count package verification, audit replay, and district
aggregation tools.**

RCOUNT is the neutral home for election-count package crates that should be
usable by BISECT and other civic evidence systems without living inside the
BISECT application workspace.

## Workspace

| Crate | Purpose |
|-------|---------|
| `rcount-core` | election-count data model, canonical hashing, and reconciliation checks |
| `rcount-stats` | deterministic statistical primitives for audit replay |
| `rcount-io` | package directory read/write helpers |
| `rcount-audit` | package verification and audit transcripts |
| `rcount-district` | optional aggregation over RPLAN/RCTX assignments |
| `rcount-cli` | `rcount` command-line verifier |
| `rcount-rhist` | optional bridge from RCOUNT lineage to RHIST records |

## Design rule

RCOUNT owns election-count package verification and audit replay boundaries. It
does not own BISECT redistricting, RPLAN plan-package representation, or RLINE
graph/context kernels.

## Dependency note

`rcount-district` and `rcount-rhist` use sibling paths to RLINE for `rctx-core`
and `rhist-core`. RPLAN dependencies point at the sibling RPLAN repo.

## Commands

```powershell
cargo test --workspace
cargo run -p rcount-cli -- --help
```

## Specs

- [`docs\specs\rcount-foundation.md`](docs/specs/rcount-foundation.md) records
  the extraction boundary.
- `context\waves\` tracks implementation waves and pulse history.

## License

MIT. See [`LICENSE`](LICENSE).

