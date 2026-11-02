# Package workbench: reader foundation

Status: reader foundation qualified locally; browser adapter, UI, hosted
qualification and publication remain open. This is not a Pages release.

The additive `PackageReader` API lets the existing package parser and source
verifier consume original bytes from another read-only backend. Existing
filesystem APIs delegate to `FileSystemReader`; NDJSON still uses `BufRead::lines`,
JSON still uses `serde_json::from_slice`, and source hashes retain the native
domain prefix and raw file bytes. The audit crate delegates its existing checks
through the reader. Writers and import commands retain their native paths.

## Evidence, 2026-11-02

Environment: Windows, Rust/Cargo 1.95.0, locked original workspace dependencies.

- `cargo +1.95.0 test --workspace --locked`: 278 tests pass (271 original,
  seven new integration tests). New tests compare all 42 golden package
  transcripts across filesystem and memory, and cover CRLF/blank NDJSON,
  raw source-byte changes, missing source/required/optional files, malformed JSON
  and invalid UTF-8. Original test modules remain unchanged.
- `cargo +1.95.0 fmt --all -- --check` and `git diff --check`: pass.
- `cargo +1.95.0 check -p rcount-audit --target wasm32-unknown-unknown --locked`:
  passes before and after the reader change. This is compilation evidence;
  executing-WASM/browser parity remains open.
- `python scripts/check-original-cli.py <built-rcount-executable>`: all 84
  original stdout, stderr and exit-status controls match exactly. The fixture
  records the original native binary hash and source SHA, includes 17 failing
  command invocations, and was captured twice before edits.
- `python scripts/check-native-foundation.py`: checks unchanged native
  algorithms, CLI source, root manifest/lock, examples and original IO/audit tests
  against committed SHA-256 records. Raw source evidence is not newline-normalized.

Strict Clippy is an existing gap rather than this repo's documented release gate.
The original all-targets `-D warnings` invocation fails on boolean assertions in
`rcount-stats`; the original library-only invocation also reports core style
findings (`unnecessary_map_or`, `manual_div_ceil`, `too_many_arguments`). Those
algorithms/tests are preserved. No lint gate is suppressed in this change.

## Scoped role review

Count Evidence Steward: the reader preserves filesystem existence checks and
raw source hashing, including missing-source and hash-mismatch errors. It does
not claim source authenticity. Memory IO messages belong to the backend; only
complete supplied packages are claimed to have full transcript parity here.

Audit Replay Auditor: no statistics, replay method selection, transcript schema,
or pass/fail/boundary semantics change. Before exposing replay, the adapter must
bound the RLA `sample_size` allocation/loop and Minerva's sample-driven binomial
work, in addition to file bytes and nested record counts. Worker cancellation
and memory caps alone do not close native admission requirements.

Package Bridge Reviewer: existing directory entry points and return types stay
available; the new APIs are additive. Root lock and Git dependency pins stay
unchanged. No RPLAN/RLINE adoption, district aggregation change or BISECT snapshot
update is admitted by this foundation.

## BISECT comparison and adoption boundary

Read-only committed-blob comparison at BISECT
`af78b0a708960998b4f0cacfc0ef236e9d1b0590`: its embedded audit source matches
RCOUNT's original `3849b249` source, while its IO source already differs.
The original RCOUNT audit/IO SHA-256 values are respectively
`23471dd1cd460e6c6e23ac6da7688e072d1952535a8bae9340379eec9ddd4252` and
`e2827d6bec0ae69a6a86e5379cdfd539bf30d54f5b80aca23e6c903f90139d07`;
BISECT IO is `d68dc775e372294f0ae2c480a9631e6023c433a13d41974938511f43d97bbcfb`.
The new reader APIs are not ported: BISECT's active owner session controls its
embedded sources and dependency migration. Its existing directory callers do not
need the new reader APIs. A future deliberate adoption must reconcile the IO
differences and run BISECT's affected tests/VTRACE before updating that snapshot.

## Remaining release gates

See [SPEC.md](SPEC.md). Next: bounded native/WASM adapter using raw package files,
complete report parity, worker/browser task and recovery checks, independent
source/WASM/provenance/license downloads, clean-source rebuild, exact hosted
artifact qualification and default-branch live verification. TRACKER's canonical
RCOUNT gitlink remains at the original preserved source until those gates pass.
