#!/usr/bin/env python3
"""Enforce architecture 02 §2.9 and ADR-0009 on Cargo's resolved graph."""
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parent.parent
ORDER = "verse-syntax verse-hir verse-types verse-own verse-mir verse-reach verse-ffi verse-bc verse-cl verse-llvm verse-wasm verse-rt verse-pkg verse-fmt verse-lsp verse-dbg verse-cli".split()


def violations(metadata, manifest):
    errors = []
    packages = {p["id"]: p for p in metadata["packages"]}
    members = set(metadata["workspace_members"])
    ranks = {name: i for i, name in enumerate(ORDER)}
    for member in members:
        package = packages[member]
        name = package["name"]
        if name not in ranks:
            errors.append(f"{name}: no position in the documented crate layout")
        for dependency in package["dependencies"]:
            other = dependency["name"]
            if other in ranks and ranks.get(name, -1) <= ranks[other]:
                errors.append(f"{name} -> {other}: backward dependency")
    if set(manifest.get("profile", {})) - {"dev", "release", "test", "bench"}:
        errors.append("custom profiles violate ADR-0009")
    return errors


def main():
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version=1", "--locked", "--no-deps"], cwd=ROOT))
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    errors = violations(metadata, manifest)
    for package in metadata["packages"]:
        path = Path(package["manifest_path"])
        package_manifest = tomllib.loads(path.read_text())
        for group in ("dependencies", "dev-dependencies", "build-dependencies"):
            for name, entry in package_manifest.get(group, {}).items():
                if not isinstance(entry, dict) or entry.get("workspace") is not True:
                    errors.append(f"{path}: {name} is not a workspace dependency")
    for error in errors:
        print(error)
    print(f"dependency check: {len(errors)} failed")
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
