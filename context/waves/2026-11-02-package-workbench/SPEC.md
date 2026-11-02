# Package verification and audit replay workbench

Pre-merge snapshot: implementation qualified at `d862733`; publication requires
final default-branch deployment and fresh live verification. The qualification
record keeps implementation and publication evidence separate.

Expose RCOUNT's existing package verification and statistical audit replay in
WebAssembly. Readers choose a complete local package directory or a supplied
synthetic example, inspect the verification checks and replay boundaries, and
download the resulting native transcripts. Source files remain raw bytes: no
newline conversion, reconstructed source JSON, or browser hash substitute.

Add a reader boundary to `rcount-io` and `rcount-audit`; preserve filesystem entry
points and use the same parsing, content hashing, verification, and replay code.
Keep core/statistics algorithms, serialized contracts, errors, CLI commands,
dependency pins, and original tests unchanged. Memory-reader IO errors may differ
from operating-system filesystem error messages; do not claim byte parity there.

Evidence-driven portability exception: the pre-portability adapter at `46fa9aec`
demonstrates 14 native/WASM package-report differences caused by truncating the
sampler hash to pointer width before modulus. Perform that single modulus in
`u64` before converting the bounded index, preserving original 64-bit native
draws. Original 32-bit behavior changes deliberately; see
[`sampler-portability.md`](sampler-portability.md). No other core algorithm edits
are admitted, and original test modules, schema and hash declarations stay fixed.

The browser adapter must admit canonical relative paths and bounded inputs before
computation. Inspect loop-driving numeric fields as well as byte/record limits.
Run computation in a worker with cancellation, deadline, memory bounds, and
recovery. Show malformed input, missing files, failed checks, unsupported replay
methods, and unavailable WASM explicitly. Never infer replay support from CLI
exit status: native boundary-only replay can exit successfully.

Verification establishes consistency against supplied package declarations. It
does not authenticate sources, certify an election, approve canvassing or district
plans, or establish a risk guarantee for methods labelled boundary. District
aggregation and import commands retain their native task and source routes.

## Qualification gates

- Original 271 native tests and formatting pass. Record existing strict Clippy
  findings separately; do not alter protected algorithms to satisfy style lints.
- Frozen original CLI controls: 42 package directories, verify and replay,
  84 outputs repeated byte-for-byte before edits, including 17 nonzero exits.
  Custody SHA-256: `4747260340da74ed1cc24ccf78849254bbb1fad4c339ff0b650ed037233604e6`.
  Compare after refactoring and in an extracted source rebuild.
- Test memory/filesystem equivalence, source corruption, missing files, malformed
  NDJSON, CRLF, invalid UTF-8, and optional-file behavior without weakening tests.
- Compare complete native/WASM reports for synthetic pass, failure, privacy,
  source hash, and replay boundary cases. Exercise actual browser worker recovery,
  keyboard/mobile use, local inputs, exports, links, and unavailable-engine state.
- Publish pinned reproducible builds, complete source download, WASM download,
  provenance and dependency/license notices independently of engine startup.
- Qualify exact hosted artifact and default-branch live bytes before recording
  release or updating TRACKER's gitlink.

RCOUNT starts at `3849b249b6d4f3016d387faef936d82899527141` (preserved remotely
on `pitfall/authority-boundaries-20261102`). BISECT owns its separate active
session. Record affected embedded API differences and defer adoption; this slice
does not update BISECT or claim its snapshot migration has passed.
