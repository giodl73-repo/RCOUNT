# Package workbench: reader foundation

Status at the pre-merge gate: implementation, hosted browser/artifact delivery
and extracted-source rebuild qualified. Final merge/default deployment/live
verification are separate gates. See the deployed `build.json` and default-branch
Actions runs for the currently published source identity.

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
or pass/fail/boundary semantics change. The adapter now bounds the RLA `sample_size` allocation/loop and Minerva's sample-driven binomial
work, in addition to file bytes and nested record counts. Those limits are
tested before native verification; worker cancellation and memory caps remain
additional controls.

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
15 generated audit examples), and seventeen actual-browser task/recovery checks (including real folder input).
Those cover privacy/source failures, supported replay and boundary methods,
binary source downloads, exact transcript exports, malformed bundles, cancellation,
worker deadlines/restart, unavailable WASM, keyboard and 390-pixel mobile layout.
Desktop/mobile screenshots were visually reviewed; a file-picker overflow was
fixed. The compiled prototype WASM is 837,927 bytes. A clean committed prototype
build passes the same parity gates; release artifact custody and a full extracted
source rebuild of the adapter remain open. Shared browser/native
dependency versions match; native manifest/lock and golden bytes stay preserved.

The full qualification at `d86273335a732e4a72e902f9b5a7798f5b12e635`
passed [hosted Pages build 41519548361](https://github.com/giodl73-repo/RCOUNT/actions/runs/41519548361)
and [Windows CI 41519548322](https://github.com/giodl73-repo/RCOUNT/actions/runs/41519548322).
Its synthetic checkout `51b05c214fef221f96018ebd9f486b35c691ac9d` has the same
complete Git tree as that candidate. The actual Linux artifact was independently
downloaded and passed all nineteen delivery/task/recovery cases under `/RCOUNT/`.
The delivered source rebuilt outside Git on Windows and hosted Linux: 279 root
tests, twelve adapter tests, 84 frozen original CLI controls, 57 complete
native/WASM reports and seventeen browser cases. All 957 source files in that
qualification archive were hash-verified before rebuilding. The archive's repeated
local packaging produced identical bytes with fixed member timestamps.

The qualified Linux artifact contains 167 served files, 862,038 runtime bytes,
838,154 WASM bytes, 42 dependency notice groups and twelve Rust notice files.
Windows and Linux runtime binaries differ; cross-platform binary identity is not
claimed. Native response strings and raw source evidence remain independently
checked. [`implementation-qualification.json`](implementation-qualification.json)
retains the precise candidate/run identities and counts. Subsequent documentation
commits add evidence without redefining those historical custody measurements.

The [representative local observations](browser-observations.json) use fresh
headless Chromium contexts on Windows, a loopback server and the thirteen-file,
4,357-byte `summary-basic` package, without CPU or network throttling: desktop
startup 70.6 ms and computation/render 14.2 ms; 390-pixel viewport startup 77.0 ms
and computation/render 14.4 ms. These are two observations, not performance or
physical-mobile-device guarantees. Screenshots and no-horizontal-overflow checks
cover both viewports.

Release-stage review: Count Evidence Steward confirms original raw-byte custody
and unchanged native64 outputs; Audit Replay Auditor confirms supported-method
parity, explicit boundaries and the separately documented native32 sampler
correction; Package Bridge Reviewer confirms exact source/dependency notices,
clean-source rebuild and no BISECT snapshot admission. Local own-input packages
remain outside all public source/artifact downloads.

See [SPEC.md](SPEC.md) for the complete gates. Final merge, default deployment and
fresh live browser/file verification remain the publication-stage gate at this
record's pre-merge timestamp. TRACKER's canonical RCOUNT gitlink is updated only
after that publication evidence exists; the active BISECT owner session remains
outside this release's admission scope.
