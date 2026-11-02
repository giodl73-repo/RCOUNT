# Seeded sampler width correction

The actual compiled WASM adapter at `46fa9aec69a7eff11d16d37efe321f5ca05dea9c`
differs from the 64-bit Windows native adapter for 14 of 42 complete golden
package reports. The native adapter independently matches the original native
CLI on every package. The probe summary and failing case names are retained in
`browser/package-workbench/width-probe-before.json`; full local probe output has
the SHA-256 recorded there. This is a measured portability defect, not a proposed
optimization or a change to golden evidence.

Previously `replay_rla_sample` converted an eight-byte SHA-256 prefix to `usize`
before modulus. WASM32 discarded the high 32 bits, changing ballot selection.
The correction performs the modulus in `u64` and then converts the bounded index
to `usize`. For every nonempty population addressable on a 32- or 64-bit host,
the result is less than the population length and therefore representable as an
index. On 64-bit hosts the expression is mathematically identical to the original.
On 32-bit hosts this deliberately corrects the old truncation behavior to match
the existing 64-bit fixture contract.

No schema, hash domain, seed, manifest, sampling method identifier, declared draw,
golden package or source bytes are changed. Old packages generated using a 32-bit
truncating sampler may now fail replay; that incompatibility is explicit. Native
32-bit migration is not claimed to preserve the erroneous historical sequence.

The foundation check allows precisely this one expression replacement, reverses
it in memory and checks the original full core-file hash. All other original
core source and tests remain protected. A new test pins all twelve original
golden draws. Qualification must still prove all original 271 tests and 84 CLI
controls, all 42 complete native/WASM report strings, and browser worker checks.

Role review: Count Evidence Steward retains frozen source evidence and 64-bit
draws. Audit Replay Auditor requires the full 14-case failure set to disappear
after the correction, without masking failing checks. Package Bridge Reviewer
defers BISECT adoption: its separate owner must deliberately reconcile the same
portability defect and qualify its callers before admitting a snapshot update.
