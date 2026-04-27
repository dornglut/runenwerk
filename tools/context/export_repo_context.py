#!/usr/bin/env python3
"""
Export selected repository files into a single line-numbered context file.

File: tools/context/export_repo_context.py
Function: main
"""

from __future__ import annotations

import argparse
from pathlib import Path

DEFAULT_EXTENSIONS = {
    ".rs",
    ".toml",
    ".md",
    ".mdx",
    ".py",
}

DEFAULT_INCLUDE_FILENAMES = {
    "Cargo.toml",
}

DEFAULT_EXCLUDED_DIRS = {
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    ".astro",
}

DEFAULT_EXCLUDED_FILENAMES = {
    "Cargo.lock",
}


def should_skip_path(path: Path, excluded_dirs: set[str]) -> bool:
    return any(part in excluded_dirs for part in path.parts)


def should_include_file(path: Path, extensions: set[str], include_filenames: set[str]) -> bool:
    if path.name.endswith("-content.txt"):
        return False

    if path.name in DEFAULT_EXCLUDED_FILENAMES:
        return False

    return path.name in include_filenames or path.suffix in extensions


def iter_context_files(
    root: Path,
    extensions: set[str],
    include_filenames: set[str],
    excluded_dirs: set[str],
) -> list[Path]:
    files: list[Path] = []

    for path in root.rglob("*"):
        relative = path.relative_to(root)

        if should_skip_path(relative, excluded_dirs):
            continue

        if not path.is_file():
            continue

        if should_include_file(path, extensions, include_filenames):
            files.append(relative)

    return sorted(files, key=lambda item: str(item))


def write_context_file(root: Path, output: Path, files: list[Path]) -> None:
    with output.open("w", encoding="utf-8") as out:
        for relative in files:
            absolute = root / relative
            out.write(f"\n===== FILE: ./{relative.as_posix()} =====\n")

            with absolute.open("r", encoding="utf-8", errors="replace") as source:
                for line_number, line in enumerate(source, start=1):
                    out.write(f"{line_number:6}\t{line}")

                    if not line.endswith("\n"):
                        out.write("\n")

            out.write(f"\n===== END FILE: ./{relative.as_posix()} =====\n")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Export repository source/docs into a single line-numbered context file."
    )
    parser.add_argument(
        "--root",
        default=".",
        help="Repository root to export from. Defaults to current directory.",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Output file. Defaults to ./<repo-folder-name>-content.txt.",
    )
    parser.add_argument(
        "--exclude-dir",
        action="append",
        default=[],
        help="Additional directory name to exclude. Can be passed multiple times.",
    )

    args = parser.parse_args()

    root = Path(args.root).resolve()
    output = Path(args.output) if args.output else root / f"{root.name}-content.txt"

    excluded_dirs = set(DEFAULT_EXCLUDED_DIRS)
    excluded_dirs.update(args.exclude_dir)

    files = iter_context_files(
        root=root,
        extensions=set(DEFAULT_EXTENSIONS),
        include_filenames=set(DEFAULT_INCLUDE_FILENAMES),
        excluded_dirs=excluded_dirs,
    )

    write_context_file(root=root, output=output, files=files)

    print(f"Wrote {output}")
    print(f"Exported {len(files)} files")


if __name__ == "__main__":
    main()
