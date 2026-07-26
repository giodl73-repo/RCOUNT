# RCOUNT

**Reusable election-count package verification, audit replay, and district
aggregation tools.**

**Series:** [Election Systems](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/election-systems.md).

**Review roles:** This repo uses
[ROLES](https://github.com/giodl73-repo/ROLES), the `.roles` convention for
repository-local review panels.

## R package family

RCOUNT is the count-verification layer in a reusable civic-evidence family:

```text
                    ┌→ RPLAN  — district-plan packages, IO, and audits ─┐
RLINE — kernels ────┤                                                   ├→ BISECT
                    └→ RCOUNT — count packages and audit replay ────────┘
```

| Repo | Responsibility |
|------|----------------|
| [RLINE](https://github.com/giodl73-repo/RLINE) | Product-neutral graph, context, statistics, optimization, and history kernels. |
| [RPLAN](https://github.com/giodl73-repo/RPLAN) | Portable district-plan representation, interchange, hashing, and audit certificates. |
| **RCOUNT** | Election-count package verification, reconciliation, aggregation, and audit replay. |
| [BISECT](https://github.com/giodl73-repo/BISECT) | Redistricting application and research workbench that consumes the reusable layers. |

The dependency direction is one-way: verification packages do not inherit
BISECT redistricting or product-reporting workflows.

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

`rcount-district` and `rcount-rhist` use GitHub dependencies on RLINE for
`rctx-core` and `rhist-core`. RPLAN dependencies also resolve from GitHub. For
local peer development, copy `.cargo\config.local.example.toml` to
`.cargo\config.toml`.

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
