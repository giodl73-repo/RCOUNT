"""Raw-byte package requests; native controls stay independent of JS rendering."""
from pathlib import Path


def golden_requests(root: Path):
    corpus = root / "docs/examples/rcount-golden-packages"
    for manifest in sorted(corpus.rglob("manifest.json")):
        folder = manifest.parent
        yield folder.relative_to(corpus).as_posix(), {"files": [
            {"path": path.relative_to(folder).as_posix(), "bytes": list(path.read_bytes())}
            for path in sorted(folder.rglob("*")) if path.is_file()
        ]}
