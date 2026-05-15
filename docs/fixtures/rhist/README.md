# RHIST Fixtures

These fixtures exercise the package shape in
`docs/specs/2026-05-13-rhist-implementation.md`.

Current fixtures are design fixtures for the first verifier slice:

- `l0-rename`: positive two-cycle one-to-one rename.
- `l0-missing-unit`: negative lineage event references a missing target unit.
- `l1-split-merge`: positive split/merge with exhaustive rational crosswalks.
- `l1-bad-weights`: negative exhaustive crosswalk weights sum to `6/5`.
- `l2-three-cycle`: locked positive rename plus split/merge across three cycles.
- `real-ri-tract-unchanged`: real-source pressure fixture using preserved
  Rhode Island Census-derived tract rows for GEOID `44001030601`.

The tiny source files are preserved under each fixture's `sources/` directory,
and their SHA-256 values are recorded in `sources/source-index.json`.

Package content hashes are computed with `rhist-io`:

```text
cargo run -p rhist-io --example refresh_fixture_hashes
```

`rhist-core` verifies declared manifest package hashes against the canonical
package projection. RCOUNT references these fixtures by package hash and cycle
ids; consumers should not copy RHIST lineage events into their own package
records.
