"""Compare the native CLI to controls frozen before the package-reader change.

Run after `cargo build -p rcount-cli --locked`; pass the executable's path.
This checks the original platform's JSON outputs, stderr, and exit status.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    fixture = root / "context/waves/2026-11-02-package-workbench/original-cli-controls.json"
    expected_hash = "4747260340da74ed1cc24ccf78849254bbb1fad4c339ff0b650ed037233604e6"
    if hashlib.sha256(fixture.read_bytes()).hexdigest() != expected_hash:
        raise SystemExit("Frozen control custody mismatch")
    controls = json.loads(fixture.read_text(encoding="utf-8"))["controls"]
    executable = str(args.executable.resolve())
    failures = []
    for control in controls:
        result = subprocess.run(
            [executable, control["command"], control["package"], "--format", "json"],
            cwd=root, capture_output=True, timeout=30,
        )
        if (result.returncode, result.stdout, result.stderr) != (
            control["exit"], control["stdout"].encode("utf-8"), control["stderr"].encode("utf-8")
        ):
            failures.append(f'{control["command"]}: {control["package"]}')
    if failures:
        raise SystemExit("Original CLI control mismatch:\n" + "\n".join(failures))
    print(f"{len(controls)} original CLI outputs, stderr, and exit statuses match exactly")


if __name__ == "__main__":
    main()
