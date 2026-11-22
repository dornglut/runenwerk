"""Executable adversarial and positive cases for the CI scope classifier."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from ci_change_scope import changed_paths, is_docs_only


class ScopeRules(unittest.TestCase):
    def test_docs_site_and_safe_root_docs(self) -> None:
        self.assertTrue(
            is_docs_only(
                ["docs-site/src/content/docs/index.mdx", "docs-site/astro.config.mjs"]
            )
        )
        self.assertTrue(is_docs_only(["README.md", "ARCHITECTURE.md"]))

    def test_ambiguous_and_mixed_changes_remain_full(self) -> None:
        for paths in [
            [],
            ["TESTING.md"],
            [".github/workflows/ci.yml"],
            ["Cargo.lock"],
            ["tools/checks/ci_change_scope.py"],
            ["docs-site/readme.md", "engine/src/lib.rs"],
            ["docs-site/../engine/src/lib.rs"],
        ]:
            with self.subTest(paths=paths):
                self.assertFalse(is_docs_only(paths))

    def test_renamed_source_into_docs_does_not_hide_source_change(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def git(*args: str) -> str:
                return subprocess.check_output(
                    ["git", *args], cwd=root, text=True
                ).strip()

            git("init", "-q")
            git("config", "user.name", "CI Test")
            git("config", "user.email", "ci@example.invalid")
            (root / "engine").mkdir()
            (root / "docs-site").mkdir()
            (root / "engine" / "sample.rs").write_text("fn main() {}\n")
            git("add", ".")
            git("commit", "-qm", "initial")
            base = git("rev-parse", "HEAD")
            git("mv", "engine/sample.rs", "docs-site/sample.rs")
            git("commit", "-qm", "move source to documentation")
            head = git("rev-parse", "HEAD")

            paths = changed_paths(base, head, cwd=root)
            self.assertEqual(sorted(paths), ["docs-site/sample.rs", "engine/sample.rs"])
            self.assertFalse(is_docs_only(paths))

            with self.assertRaises(ValueError):
                changed_paths(base, "0" * 40, cwd=root)
            with self.assertRaises(ValueError):
                changed_paths("incorrect-ref", head, cwd=root)
            with self.assertRaises(subprocess.CalledProcessError):
                changed_paths("0" * 40, head, cwd=root)


if __name__ == "__main__":
    unittest.main()
