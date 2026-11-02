# Package workbench: reader foundation

Status: reader foundation and browser prototype qualified locally; hosted browser
qualification, release packaging and publication remain open. This is not a
Pages release.

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
- Extracted committed source outside Git into a path containing a space: all
  278 tests, 84 original CLI controls and 811 protected foundation entries pass.
  The rebuild used a distinct `rcount-replay` Cargo profile in the shared target
  cache so fixture paths could not reuse the worktree's compiled test binaries.
- Hosted Windows qualification at candidate
  `003526bde2a6fc54ebcf33aa2308a630a871074b`,
  [run 41514482949](https://github.com/giodl73-repo/RCOUNT/actions/runs/41514482949):
  formatting, native workspace tests, CLI smoke, foundation and original CLI
  control steps pass. This qualifies the reader foundation, not the open browser
  release gates.

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

The local prototype now runs the existing complete verifier and audit replay in
a worker. It opens local folders and byte-preserving bundles, shows every native
verification check and method boundary, and downloads original bytes and exact
Rust transcript strings. Admission bounds are distinct from native validity.
See [`sampler-portability.md`](sampler-portability.md) for the single measured
core correction required to preserve existing 64-bit draws on WASM32.

Local prototype checks: 279 root native tests (271 original, seven reader, one
new frozen sampler vector), twelve adapter admission tests, all 84 frozen CLI
controls, 57 complete native/WASM response strings (42 original packages and
15 generated audit examples), and sixteen actual-browser task/recovery checks.
Those cover privacy/source failures, supported replay and boundary methods,
binary source downloads, exact transcript exports, malformed bundles, cancellation,
worker deadlines/restart, unavailable WASM, keyboard and 390-pixel mobile layout.
Desktop/mobile screenshots were visually reviewed; a file-picker overflow was
fixed. The compiled prototype WASM is 837,927 bytes. It remains a dirty prototype
build until clean-source artifact custody is qualified. Shared browser/native
dependency versions match; native manifest/lock and golden bytes stay preserved.

See [SPEC.md](SPEC.md). Next: hosted adapter/browser gates, complete release
packaging with scoped dependency notices, independent source/WASM/provenance/license
downloads, clean-source rebuild, exact hosted artifact qualification and
default-branch live verification. TRACKER's canonical
RCOUNT gitlink remains at the original preserved source until those gates pass.
