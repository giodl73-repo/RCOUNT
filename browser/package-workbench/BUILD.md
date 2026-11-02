# Package workbench build

This prototype is not yet a qualified Pages release. The open release gates are
in `context/waves/2026-11-02-package-workbench/SPEC.md`.

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
