# Wave: RCOUNT Foundation

## Goal

Create a standalone Rust workspace for the reusable RCOUNT package family.

## Pulse table

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Workspace extraction | done | Copied RCOUNT crates from BISECT, added standalone workspace metadata, docs, and validation. |
| 02 | RLINE dependency cleanup | done | Replaced local `rctx-core`/`rhist-core` paths with GitHub RLINE dependencies. |
| 03 | BISECT dependency rewire | pending | Update BISECT to consume RCOUNT from the sibling repo or git dependency. |

## Success criteria

- RCOUNT has its own Rust workspace and git repo.
- Existing RCOUNT crates build and test outside BISECT.
- Docs define product boundaries and GitHub RLINE/RPLAN dependencies.
- `cargo fmt`, `cargo test --workspace`, CLI help smoke, and `git diff --check`
  pass.

