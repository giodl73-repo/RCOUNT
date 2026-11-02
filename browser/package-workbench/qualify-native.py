"""Capture complete adapter responses and compare independent native CLI output."""
import argparse
import json
from pathlib import Path
import subprocess
from controls import golden_requests

p = argparse.ArgumentParser()
p.add_argument("--binary", required=True, type=Path)
p.add_argument("--original", required=True, type=Path)
p.add_argument("--output", required=True, type=Path)
a = p.parse_args()
root = Path(__file__).resolve().parents[2]
cases = []
for name, request in golden_requests(root):
    raw = json.dumps(request, separators=(",", ":"))
    response = subprocess.check_output([str(a.binary.resolve())], input=raw.encode("utf-8"), timeout=30)
    report = json.loads(response)
    if report["kind"] != "report":
        raise SystemExit(f"Golden package was not admitted: {name}: {report}")
    directory = root / "docs/examples/rcount-golden-packages" / name
    for command, actual in [
        ("verify", report["verification"]),
        ("replay-audit-algorithms", report["replay"].get("transcripts")),
    ]:
        cli = subprocess.run([str(a.original.resolve()), command, str(directory), "--format", "json"], capture_output=True, timeout=30)
        if actual is not None:
            assert json.loads(cli.stdout) == actual, (name, command)
        else:
            assert cli.returncode == 2, (name, command, cli.returncode)
    cases.append(dict(name=name, request=raw, native=response.decode("utf-8")))
a.output.parent.mkdir(parents=True, exist_ok=True)
a.output.write_text(json.dumps(cases, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"{len(cases)} complete native adapter reports match independent original CLI controls")
