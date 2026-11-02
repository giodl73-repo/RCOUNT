# Package workbench semantics and limits

The browser uses `rcount-io` to load the complete package, `rcount-core` to verify
its declarations and `rcount-audit`/`rcount-stats` to produce native verification
and statistical replay transcripts. JavaScript handles file selection, byte
transport, worker lifecycle, display and downloads. It does not reimplement
reconciliation, package/source hashes, privacy gates, seeded draws or statistics.

Verification and statistical replay are separate findings. A failed verification
does not hide replay diagnostics. Missing or unreadable package records make
replay unavailable. A boundary transcript is never relabelled pass; the native
replay CLI can exit successfully when all its methods are boundary-only.
Pass means consistency with supplied declarations, not source authentication,
election certification, canvass approval, district-plan approval or a risk
guarantee for a method that is not statistically replayed.

## Admission before native work

- Transport: at most 4 MiB UTF-8 JSON, one through 64 files, 128 KiB per file,
  512 KiB total original bytes. File bytes are octets, including binary sources.
- File names: canonical case-sensitive relative ASCII paths; no aliases,
  traversal, backslashes, drive names, empty components, trailing dots or Windows
  reserved device names. Source-index references must be under `sources/`.
- Parsed data records: at most 2,048 records and 65,536 JSON nodes, arrays of at
  most 512 entries, keys of at most 256 bytes, strings of at most 16 KiB and
  numeric magnitude at most 10^12. The native parser retains malformed-record
  and UTF-8 errors rather than interpreting rejected records as empty data.
- Loop drivers: at most 256 draws per RLA and 1,024 total; at most sixteen audit
  algorithm runs and 256 total sample steps, with at most 64 steps in a Minerva
  run. Minerva's binomial-tail work depends on the cumulative sample count.
- Results: complete report at most 2 MiB. Refusal exports no partial verification
  or partial statistical transcript. Native methods still own their validation,
  exact rational arithmetic, overflow boundaries and reported error meanings.
- Worker: 15-second startup and 10-second computation deadlines, cancellation,
  termination on failure and explicit restart. Inputs survive worker failure.
  The actual compiled WASM memory declares a maximum of 64 MiB, checked during
  packaging; this does not claim a bound on the browser's entire process memory.

These are browser admission policies, not new package validity rules. Oversize
packages, import commands and district aggregation retain their native CLI and
source routes. Local file reading and network example downloads invalidate stale
findings; generation tokens prevent late results from replacing a newer input.

## Seeded sample portability

The original sampler truncated a 64-bit digest prefix to pointer width before
modulus. That produced fourteen measured full-report differences on WASM32.
The single core correction performs modulus in `u64` before converting the
bounded index. It preserves original 64-bit native draws and intentionally
changes the old 32-bit truncating sequence. Existing 32-bit-generated packages
may fail replay after this correction. See the wave's sampler portability record
for before/after evidence and frozen draw vectors; no schema, source fixture,
hash domain or method identifier is rewritten.

The memory reader has case-sensitive logical file identity and backend-specific
missing-file messages. Platform-specific filesystem alias/error equivalence is
not claimed. Exact complete native/WASM response strings and native CLI semantic
comparisons cover the admitted supplied fixtures, with frozen original native
CLI byte controls kept separately.
