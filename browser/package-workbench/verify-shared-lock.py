"""Portable bindings may add dependencies; shared native versions stay pinned."""
from pathlib import Path
import tomllib

browser=Path(__file__).resolve().parent
root=browser.parents[1]
native=tomllib.loads((root/'Cargo.lock').read_text(encoding='utf-8'))['package']
portable=tomllib.loads((browser/'Cargo.lock').read_text(encoding='utf-8'))['package']
versions={}
for package in native:
    versions.setdefault(package['name'],set()).add(package['version'])
different=[f"{p['name']} {p['version']}" for p in portable
           if p['name'] in versions and p['version'] not in versions[p['name']]]
if different: raise SystemExit('Shared dependency pins changed: '+', '.join(different))
print('All shared native/browser dependency versions match')
