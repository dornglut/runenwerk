"""Fail-closed documentation-only change classifier for exact PR Git revisions."""

from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

DOC_ROOT = "docs-site/"
DOC_FILES = frozenset({"README.md", "ARCHITECTURE.md"})
SHA = re.compile(r"[0-9a-f]{40}\Z")


def is_docs_only(paths: list[str]) -> bool:
    """Only known, normalized documentation paths qualify; mixed/empty do not."""
    return bool(paths) and all(
        path in DOC_FILES
        or (
            path.startswith(DOC_ROOT)
            and all(part not in {"", ".", ".."} for part in path.split("/"))
        )
        for path in paths
    )


def changed_paths(base: str, head: str, *, cwd: Path | None = None) -> list[str]:
    """Compute exact PR changes, preserving both sides of renamed files."""
    if not SHA.fullmatch(base) or not SHA.fullmatch(head):
        raise ValueError("PR base and head must be full lowercase commit SHAs")

    def git(*args: str) -> bytes:
        return subprocess.check_output(["git", *args], cwd=cwd)

    actual_head = git("rev-parse", "HEAD").decode("ascii").strip()
    if actual_head != head:
        raise ValueError("checked-out HEAD differs from the requested PR head")

    merge_base = git("merge-base", base, head).decode("ascii").strip()
    changed = git("diff", "--name-only", "--no-renames", "-z", merge_base, head, "--")
    if not changed:
        return []
    if not changed.endswith(b"\0"):
        raise ValueError("git diff returned malformed NUL-separated filenames")
    return [name.decode("utf-8") for name in changed[:-1].split(b"\0")]


def main() -> None:
    base = os.environ["PR_BASE_SHA"]
    head = os.environ["PR_HEAD_SHA"]
    output = os.environ["GITHUB_OUTPUT"]
    paths = changed_paths(base, head)
    docs_only = is_docs_only(paths)
    print(f"Exact PR changed paths: {len(paths)}")
    print(f"Validation scope: {'docs-only' if docs_only else 'full'}")
    with open(output, "a", encoding="utf-8") as stream:
        stream.write(f"docs_only={'true' if docs_only else 'false'}\n")


if __name__ == "__main__":
    main()
