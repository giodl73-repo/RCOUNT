# RPLAN Benchmark Packages

Benchmark packages sit above tiny golden packages and method-produced fixtures.
They are still small enough to review, but they carry benchmark-tier metadata:
scale notes, timing protocol, hardware notes, method transcript, package
manifest hashes, and RPLAN audit artifacts.

## Claim Boundary

A benchmark package can support:

- verifier behavior on a larger committed package
- replayable command and method transcripts
- data-provenance and package-footprint claims
- timing protocol documentation

It does not prove legal sufficiency, fairness, optimality, real-data quality, or
universal method superiority.

## Package Shape

```text
package-name/
  plan.rplan
  context.rctx
  audit-certificate.json
  manifest.json
  method-transcript.json
  command-transcript.txt
  benchmark-notes.json
  method-specific-report.json
```

The manifest schema is `benchmark-rplan-package-manifest-v1`. It preserves the
same `files` array with `path`, `sha256`, and `role` fields used by the public
RPLAN package verifier bridge.

## Verifier Commands

```powershell
cargo run -p rplan-cli -- verify-certificate `
  --certificate docs/examples/rplan-benchmark-packages/<package>/audit-certificate.json `
  --plan docs/examples/rplan-benchmark-packages/<package>/plan.rplan `
  --context docs/examples/rplan-benchmark-packages/<package>/context.rctx
```

```powershell
cargo run -p bisect-cli -- verify `
  --manifest docs/examples/rplan-benchmark-packages/<package>/manifest.json
```

## Current Packages

| Package | Source workflow | Status | Scope |
|---|---|---|---|
| `T.14+spectral-grid10-benchmark` | `cargo run -p bisect-cli --no-default-features --example spectral_grid10_benchmark_package` | 100-unit synthetic grid partition verifies | Benchmark-tier package contract and verifier scale smoke; no wall-clock or real-data claim |
| `T.15+capacity-path100-benchmark` | `cargo run -p bisect-cli --no-default-features --example capacity_path100_benchmark_package` | 100-unit synthetic path capacity clustering verifies | Capacity-clustering benchmark packaging, lineage, and verifier scale smoke; no wall-clock or real-data claim |
| `T.16+regionalization-path100-benchmark` | `cargo run -p bisect-cli --no-default-features --example regionalization_path100_benchmark_package` | 100-unit synthetic path regionalization verifies | Hierarchical-regionalization benchmark packaging, merge-log artifact capture, lineage, and verifier scale smoke; no wall-clock or real-data claim |
| `T.17+flow-path100-benchmark` | `cargo run -p bisect-cli --no-default-features --example flow_path100_benchmark_package` | 100-unit synthetic path flow construction verifies | Flow-construction benchmark packaging, lineage, and verifier scale smoke; no wall-clock or real-data claim |
| `U.16+branch-and-cut-path8-benchmark` | `cargo run -p bisect-cli --no-default-features --example u16_branch_cut_path8_benchmark_package` | 8-unit synthetic path solved to proven optimality and verifies | Exact-solver artifact packaging, LP model hashing, branch-and-cut lineage, and verifier smoke; no real-data scaling claim |
| `U.18+local-search-grid10-benchmark` | `cargo run -p bisect-cli -- improve` over `T.14+spectral-grid10-benchmark` | one-move search found no improving move; package verifies | Benchmark-tier search packaging, parent-plan lineage, and verifier scale smoke; no improvement or real-data claim |
| `U.20+audit-grid10-benchmark` | `cargo run -p bisect-cli --no-default-features --example u20_audit_grid10_benchmark_package` | 100-unit synthetic audit package verifies | Audit fixed-point packaging and verifier scale smoke; no construction, optimization, or legal sufficiency claim |
