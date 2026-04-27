#!/usr/bin/env python3
"""
Rename docs-site section landing pages from readme.md to README.md.

Scope:
- Only renames files under docs-site/src/content/docs.
- Updates textual references from readme.md to README.md across repository docs/source files.
- Uses a two-step rename so it works correctly on case-insensitive macOS filesystems.
"""

from __future__ import annotations

import argparse
from pathlib import Path

DOCS_ROOT = Path("docs-site/src/content/docs")

REFERENCE_EXTENSIONS = {
    ".md",
    ".mdx",
    ".astro",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".json",
    ".yaml",
    ".yml",
    ".toml",
}

EXCLUDED_PARTS = {
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
}


def is_excluded(path: Path) -> bool:
    return any(part in EXCLUDED_PARTS for part in path.parts)


def rename_readmes(dry_run: bool) -> list[tuple[Path, Path]]:
    renamed: list[tuple[Path, Path]] = []

    for source in sorted(DOCS_ROOT.rglob("*")):
        if not source.is_file():
            continue

        if source.name != "readme.md":
            continue

        target = source.with_name("README.md")
        temp = source.with_name("__readme_case_migration_tmp__.md")

        if target.exists() and not source.samefile(target):
            raise RuntimeError(f"target already exists beside lowercase source: {target}")

        renamed.append((source, target))

        if dry_run:
            print(f"rename {source} -> {target}")
            continue

        source.rename(temp)
        temp.rename(target)

    return renamed


def update_references(dry_run: bool) -> int:
    changed_count = 0

    for path in sorted(Path(".").rglob("*")):
        if is_excluded(path) or not path.is_file():
            continue

        if path.suffix not in REFERENCE_EXTENSIONS:
            continue

        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue

        updated = text.replace("readme.md", "README.md")

        if updated == text:
            continue

        changed_count += 1

        if dry_run:
            print(f"update references in {path}")
            continue

        path.write_text(updated)

    return changed_count


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Rename docs-site readme.md files to README.md and update references."
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print planned changes without modifying files.",
    )
    args = parser.parse_args()

    if not DOCS_ROOT.exists():
        raise SystemExit(f"missing docs root: {DOCS_ROOT}")

    renamed = rename_readmes(args.dry_run)
    changed_refs = update_references(args.dry_run)

    print(f"readme files renamed: {len(renamed)}")
    print(f"files with updated references: {changed_refs}")

    if args.dry_run:
        print("dry run only; no files changed")


if __name__ == "__main__":
    main()