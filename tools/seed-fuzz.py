#!/usr/bin/env python3
"""Seed fuzz targets with every spec example, including fragments and failures."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    examples = json.loads(subprocess.check_output(["python3", str(ROOT / "tools/extract-examples")]))
    sources = [e["source"] for e in examples]
    sources.extend(path.read_text() for path in (ROOT / "tests/conformance").glob("*.vs"))
    sources.extend(path.read_text() for path in (ROOT / "tests/golden/syntax").glob("*.vs"))
    for target in ["lexer", "parser", "formatter"]:
        directory = ROOT / "tests/fuzz/corpus" / target
        directory.mkdir(parents=True, exist_ok=True)
        for source in sources:
            (directory / hashlib.sha256(source.encode()).hexdigest()).write_text(source)
    print(f"fuzz seeds: {len(sources)} source examples per target")


if __name__ == "__main__":
    main()
