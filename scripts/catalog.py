"""Generate the release catalog from built package artifacts (CI only)."""
import hashlib
import json
import pathlib
import sys

release, directory = sys.argv[1:]
root = pathlib.Path(directory)
packages = {
    artifact.stem: {
        "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(),
        "bytes": artifact.stat().st_size,
    }
    for artifact in sorted(root.glob("*.shadow"))
}
if not packages:
    raise SystemExit("No built packages to publish")
(root / "catalog-v2.json").write_text(
    json.dumps({"format": 2, "release": release, "packages": packages}, indent=2) + "\n"
)
# Keep the original catalog readable by 0.2.0 clients with strict schemas.
legacy = {name: {"sha256": data["sha256"]} for name, data in packages.items()}
(root / "catalog.json").write_text(
    json.dumps({"format": 1, "release": release, "packages": legacy}, indent=2) + "\n"
)
