#!/usr/bin/env python3
"""Report tested spec sections without claiming future semantic coverage."""
import importlib.machinery
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent


def load_examples():
    loader = importlib.machinery.SourceFileLoader("extract_examples", str(ROOT / "tools/extract-examples"))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module.corpus()


def main():
    examples = load_examples()
    sections = set()
    for path in (ROOT / "docs/language").glob("*.md"):
        sections.update("§" + m[1] for m in re.finditer(r"^#{1,6}\s+(\d+(?:\.\d+)*)\.?\s", path.read_text(), re.MULTILINE))
    exercised = {e["section"] for e in examples if e["kind"] != "fragment"}
    fixtures = sorted((ROOT / "tests/conformance").glob("s*.vs"))
    for path in fixtures:
        match = re.match(r"s(\d+(?:\.\d+)*?)-\d+-", path.name)
        if not match:
            raise ValueError(f"invalid rule fixture name: {path}")
        exercised.add("§" + match[1])
    missing = sorted(sections - exercised, key=lambda s: tuple(map(int, s[1:].split('.'))))
    result = {"scope": "M0 syntax examples and rule fixtures; semantic conformance requires later milestones", "examples": len(examples), "parse": sum(e["kind"] == "parse" for e in examples), "reject": sum(e["kind"] == "reject" for e in examples), "fragments": [{"id": e["id"], "reason": e["reason"]} for e in examples if e["kind"] == "fragment"], "rule_fixtures": [str(p.relative_to(ROOT)) for p in fixtures], "sections_exercised": sorted(exercised), "sections_without_direct_tests": missing}
    output = ROOT / "dist/conformance-coverage.json"
    output.parent.mkdir(exist_ok=True)
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(f"spec corpus: {result['examples']} examples; {result['parse']} parse, {result['reject']} reject, {len(result['fragments'])} justified fragments")
    print(f"rule fixtures: {len(fixtures)}; sections without direct tests: {len(missing)} (see {output.relative_to(ROOT)})")


if __name__ == "__main__":
    main()
