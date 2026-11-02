# Package workbench build

This guide covers the prototype builder and full release qualification pipeline.
The gate contract is in `context/waves/2026-11-02-package-workbench/SPEC.md`;
[deployed build provenance](https://giodl73-repo.github.io/RCOUNT/build.json) and
[default-branch Actions runs](https://github.com/giodl73-repo/RCOUNT/actions)
identify the currently published artifact.

Use Rust 1.95.0 with `wasm32-unknown-unknown`, Python 3.11+, Node.js 22+ and
`wasm-bindgen-cli` exactly 0.2.127. All build commands run from the repo root:

```sh
cargo +1.95.0 test --workspace --locked
cargo +1.95.0 test --manifest-path browser/package-workbench/Cargo.toml --locked
python browser/package-workbench/build-prototype.py --bindgen /path/to/wasm-bindgen
python -m http.server 8775 --directory target/package-prototype --bind 127.0.0.1
```

Open `http://127.0.0.1:8775/RCOUNT/`. Set `CARGO_TARGET_DIR` to reuse a build cache.
The standalone browser workspace retains the root workspace's shared dependency
versions and adds pinned WASM bindings; it does not update the native lockfile.

The builder checks the protected native foundation, builds the native CLI and
both adapter targets, generates synthetic audit packages using the existing native
writer, compares complete reports to the CLI, checks frozen original CLI output,
executes the WASM and compares complete response strings. It then prepares the
project-prefixed site, original-byte examples, source snapshot and build hashes.
`build.json` explicitly records a dirty source tree until a clean committed build
is made. Prototype source ZIPs include the complete tracked workspace and new
unignored source files; generated cache/build output stays excluded.

The JSON request is `{"files":[{"path":"manifest.json","bytes":[123,125]}]}`.
Bytes are original octets, including binary source files. The worker passes that
envelope to the same Rust evaluator used by `rcount-workbench-control` on stdin.
The backend treats canonical relative file names as case-sensitive logical paths.
Filesystem error messages are not claimed to match memory-backend messages.

The native verifier still owns package hash, source hash, privacy and
reconciliation meanings. Supported statistical replay and boundary labels stay
native. Adapter admission limits are an additional browser resource policy, not
RCOUNT package validity rules. Exact transcript strings are exported from Rust so
JavaScript cannot round large computed integers during downloads.

For a release, start with clean committed source and run the following after
the prototype build:

```sh
cd browser/package-workbench
npm ci --ignore-scripts
npx playwright install chromium
npm run test:pages
cd ../..
python browser/package-workbench/package-site.py
python browser/package-workbench/verify-release.py
python browser/package-workbench/verify-source.py --bindgen /path/to/wasm-bindgen
cargo +1.95.0 test --workspace --locked
```

The final site is `target/package-release/RCOUNT/`. Packaging reads every tracked
blob directly from Git into a timestamp-stable ZIP and publishes file SHA-256
records, original dependency/runtime notices, source scope, model and build
provenance. No source or user data is fetched for packaging. Cargo may fetch the
locked native Git dependencies during the full native qualification.

The source replay extracts that ZIP outside Git into a path containing spaces,
verifies every file hash, rebuilds native tests and the adapter, compares native
and executing-WASM reports, and runs the real worker UI suite. A distinct
`rcount-replay` native test profile prevents reusing fixture paths compiled in a
different source directory. Binary identity across source directories is not
claimed. Original source evidence bytes remain exact; Windows checkout newline
conversion of other Git text does not imply byte-identical input envelopes.
