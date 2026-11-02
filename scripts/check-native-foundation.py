"""Verify protected native source, fixtures, pins, and original test modules."""

import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
manifest = root / "context/waves/2026-11-02-package-workbench/native-foundation.json"
records = json.loads(manifest.read_text(encoding="utf-8"))["files"]
failures = []
for record in records:
    data = (root / record["path"]).read_bytes()
    # Git text files can use CRLF in Windows working checkouts. Raw source evidence
    # is explicitly pinned to LF by .gitattributes and is never normalized here.
    if "/sources/" not in record["path"]:
        data = data.replace(b"\r\n", b"\n")
    if record["segment"] == "original-test-module":
        data = data[data.index(b"#[cfg(test)]\nmod tests {"):]
    if record["segment"] == "sampler-width-portability":
        # The only approved core source delta: perform modulus in u64 before
        # converting the bounded index to usize. See sampler-portability.md.
        corrected = b"let selected = (u64::from_le_bytes(bytes) % population.len() as u64) as usize;"
        original = b"let selected = u64::from_le_bytes(bytes) as usize % population.len();"
        if data.count(corrected) != 1:
            raise SystemExit("Expected exactly one sampler portability correction")
        data = data.replace(corrected, original)
    if hashlib.sha256(data).hexdigest() != record["sha256"]:
        failures.append(record["path"])
if failures:
    raise SystemExit("Protected foundation changed:\n" + "\n".join(failures))
print(f"{len(records)} protected native source/fixture/test/pin entries match")
