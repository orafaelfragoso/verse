import importlib.machinery
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    loader = importlib.machinery.SourceFileLoader(name, str(path))
    spec = importlib.util.spec_from_loader(name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


class Extraction(unittest.TestCase):
    def setUp(self):
        self.module = load("examples", ROOT / "tools/extract-examples")
        self.directory = tempfile.TemporaryDirectory(prefix="verse-extraction-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "docs/language").mkdir(parents=True)
        self.annotations = self.root / "annotations.json"
        self.annotations.write_text("{}")
        self.module.ROOT = self.root
        self.module.ANNOTATIONS = self.annotations

    def test_locations_fences_and_nested_documentation(self):
        (self.root / "docs/language/01-example.md").write_text("## 1. Example\n\n```verse\n## ```verse\n## x = 1\n## ```\nx = 1\n```\n")
        examples = self.module.corpus()
        self.assertEqual(len(examples), 1)
        self.assertEqual(examples[0]["line"], 4)
        self.assertEqual(examples[0]["section"], "§1")
        self.assertEqual(examples[0]["kind"], "parse")

    def test_stale_annotation_is_an_error(self):
        (self.root / "docs/language/01-example.md").write_text("```verse\n...\n```\n")
        self.annotations.write_text(json.dumps({"01-example/001": {"sha256": "wrong", "kind": "fragment", "reason": "omitted body"}}))
        with self.assertRaisesRegex(ValueError, "stale"):
            self.module.corpus()

    def test_indented_markdown_fences_are_extracted_and_dedented(self):
        (self.root / "docs/language/01-example.md").write_text("## 1. Example\n\n- A rule:\n\n  ```verse\n  fn f()\n      pass\n  ```\n")
        examples = self.module.corpus()
        self.assertEqual(len(examples), 1)
        self.assertEqual(examples[0]["source"], "fn f()\n    pass\n")
        self.assertEqual(examples[0]["line"], 6)
        self.assertEqual(examples[0]["column"], 3)

    def test_missing_reason_and_deleted_examples_fail(self):
        (self.root / "docs/language/01-example.md").write_text("```verse\n...\n```\n")
        example = self.module.extract()[0]
        self.annotations.write_text(json.dumps({example["id"]: {"sha256": example["sha256"], "kind": "fragment"}}))
        with self.assertRaisesRegex(ValueError, "reason"):
            self.module.corpus()
        self.annotations.write_text(json.dumps({"deleted/001": {}}))
        with self.assertRaisesRegex(ValueError, "missing examples"):
            self.module.corpus()

    def test_unterminated_fence_is_an_error(self):
        (self.root / "docs/language/01-example.md").write_text("```verse\nx = 1\n")
        with self.assertRaisesRegex(ValueError, "unterminated"):
            self.module.corpus()


class DependencyDirection(unittest.TestCase):
    def test_frontend_cannot_import_backend(self):
        module = load("dependencies", ROOT / "tools/check-dependencies.py")
        metadata = {"packages": [{"id": "syntax", "name": "verse-syntax", "dependencies": [{"name": "verse-cl"}]}, {"id": "cl", "name": "verse-cl", "dependencies": [{"name": "verse-syntax"}]}], "workspace_members": ["syntax", "cl"]}
        self.assertEqual(module.violations(metadata, {}), ["verse-syntax -> verse-cl: backward dependency"])

    def test_custom_profiles_fail(self):
        module = load("dependencies", ROOT / "tools/check-dependencies.py")
        self.assertEqual(module.violations({"packages": [], "workspace_members": []}, {"profile": {"fast": {}}}), ["custom profiles violate ADR-0009"])


if __name__ == "__main__":
    unittest.main()
