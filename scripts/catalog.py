"""Generate the release catalog from built package artifacts (CI only)."""
import hashlib
import json
import pathlib
import sys

release, directory = sys.argv[1:]
root = pathlib.Path(directory)
packages = {
    artifact.stem: {"sha256": hashlib.sha256(artifact.read_bytes()).hexdigest()}
    for artifact in sorted(root.glob("*.shadow"))
}
if not packages:
    raise SystemExit("No built packages to publish")
(root / "catalog.json").write_text(
    json.dumps({"format": 1, "release": release, "packages": packages}, indent=2) + "\n"
)
