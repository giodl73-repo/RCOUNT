# Pulse 01: Workspace extraction

## Goal

Make RCOUNT a standalone package-family repo.

## Changes

- Copied `rcount-core`, `rcount-stats`, `rcount-io`, `rcount-audit`,
  `rcount-district`, `rcount-cli`, and `rcount-rhist` from BISECT.
- Added root workspace metadata and package dependencies.
- Pointed RPLAN dependencies at the sibling RPLAN repo.
- Pointed RLINE and RPLAN dependencies at GitHub repos, with optional local
  patch config for peer development.
- Added README, foundation spec, wave docs, and repo-local skills.

## Validation

- `cargo fmt`
- `cargo test --workspace`
- CLI help smoke for `rcount`
- `git diff --check`

## Status

Done.

