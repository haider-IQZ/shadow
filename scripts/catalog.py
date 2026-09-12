"""Generate versioned release catalogs from validated, locally built artifacts."""
import hashlib
import json
import pathlib
import subprocess
import sys

release, directory = sys.argv[1:]
root = pathlib.Path(directory)
artifacts = {}
aliases = {}
legacy = {}
for artifact in sorted(root.glob("*.shadow")):
    manifest = json.loads(subprocess.check_output(
        ["tar", "--zstd", "-xOf", str(artifact), "manifest.json"], timeout=120
    ))
    record = {
        "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(),
        "bytes": artifact.stat().st_size,
        "dependencies": manifest.get("dependencies", []),
    }
    artifacts[artifact.name] = record
    name = manifest["name"]
    if manifest.get("executables", [name]):
        if name in aliases:
            raise SystemExit(f"Ambiguous default app version for {name}; select one before publishing")
        aliases[name] = artifact.name
    # Old clients only support dependency-free format-1 name.shadow artifacts.
    if manifest["format"] == 1 and artifact.name == f"{name}.shadow":
        legacy[name] = {"sha256": record["sha256"], "bytes": record["bytes"]}
if not artifacts:
    raise SystemExit("No built packages to publish")
for filename, record in artifacts.items():
    for dep in record["dependencies"]:
        depfile = f"{dep['name']}-{dep['version']}-r{dep['revision']}.shadow"
        if artifacts.get(depfile, {}).get("sha256") != dep["sha256"]:
            raise SystemExit(f"Missing or mismatched dependency {depfile} required by {filename}")
(root / "catalog-v3.json").write_text(json.dumps({
    "format": 3, "release": release, "packages": aliases, "artifacts": artifacts,
}, indent=2) + "\n")
(root / "catalog-v2.json").write_text(json.dumps({
    "format": 2, "release": release, "packages": legacy,
}, indent=2) + "\n")
(root / "catalog.json").write_text(json.dumps({
    "format": 1, "release": release,
    "packages": {name: {"sha256": data["sha256"]} for name, data in legacy.items()},
}, indent=2) + "\n")
