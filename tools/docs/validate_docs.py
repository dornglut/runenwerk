#!/usr/bin/env python3
from pathlib import Path
import re
import sys
import urllib.parse

DOCS_ROOT = Path("docs-site/src/content/docs")
REPO_ROOT = Path(".")
ALLOWED_STATUS = {
    "draft",
    "active",
    "accepted",
    "implemented",
    "completed",
    "deferred",
    "superseded",
    "rejected",
    "archived",
}
STALE_PATTERNS = {
    "engine/docs/": "engine docs moved under docs-site/src/content/docs/engine",
    "engine/README.md": "engine crate docs moved under docs-site/src/content/docs/engine/README.md",
    "engine/src/plugins/README.md": "engine plugin docs moved under docs-site/src/content/docs/engine/plugins/README.md",
    "engine/examples/README.md": "engine examples docs moved under docs-site/src/content/docs/engine/examples/overview.md",
    "engine/tests/README.md": "engine tests docs moved under docs-site/src/content/docs/engine/tests/README.md",
    "engine/src/plugins/ui": "engine has no standalone ui plugin; use domain/ui docs and scene/render integration docs",
    "src/plugins/ui/": "engine has no standalone ui plugin; use domain/ui docs and scene/render integration docs",
    "plugins/ui/README.md": "engine has no standalone ui plugin docs",
    "foundation/ids": "the current identity crate is foundation/id",
    "docs/design/": "design docs live under docs-site/src/content/docs/design",
    "docs/adr/": "ADRs live under docs-site/src/content/docs/adr",
    "scene_manager_ui": "the scene_manager_ui example is not present",
    "engine_net_quic/src/runtime/helpers.rs": "engine_net_quic runtime helpers.rs was removed",
    "engine_net_quic/src/runtime/utils.rs": "engine_net_quic runtime utils.rs was removed",
    "domain/editor/editor_shell/src/runtime/output/build_ui_frame.rs": "UI frame output moved to domain/ui/ui_runtime/src/output/build_ui_frame.rs",
    "no `impl TextLayouter`": "AtlasTextLayouter now implements TextLayouter",
}

MARKDOWN_LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")

def contains_stale_pattern(text: str, stale: str) -> bool:
    if stale == "docs/design/":
        text = text.replace("docs-site/src/content/docs/design/", "")
        text = text.replace("content/docs/design/", "")
    elif stale == "docs/adr/":
        text = text.replace("docs-site/src/content/docs/adr/", "")
        text = text.replace("content/docs/adr/", "")
    return stale in text

def has_frontmatter(text: str) -> bool:
    return text.startswith("---\n") and "\n---\n" in text[4:]

def extract_status(text: str) -> str | None:
    if not has_frontmatter(text):
        return None
    frontmatter = text.split("\n---\n", 1)[0]
    match = re.search(r"^status:\s*(\w+)\s*$", frontmatter, re.MULTILINE)
    return match.group(1) if match else None

def is_valid_docs_filename(path: Path) -> bool:
    name = path.name

    if name in {"README.md", "index.mdx"}:
        return True

    if name == "readme.md":
        return False

    return bool(re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*\.(md|mdx)", name))

def link_candidates(source: Path, raw_target: str) -> list[Path]:
    cleaned = raw_target.strip().split()[0].strip("<>")
    if cleaned.startswith(("http://", "https://", "mailto:")):
        return []
    target = urllib.parse.unquote(cleaned.split("#", 1)[0])
    if not target:
        return []

    base = (source.parent / target).resolve()
    if base.suffix:
        return [base]
    return [
        base,
        base.with_suffix(".md"),
        base.with_suffix(".mdx"),
        base / "README.md",
        base / "index.md",
        base / "index.mdx",
    ]

def repo_path_exists(path_text: str) -> bool:
    if path_text.startswith(("http://", "https://")):
        return True
    return (REPO_ROOT / path_text).exists()

def main() -> int:
    errors: list[str] = []

    if not DOCS_ROOT.exists():
        errors.append(f"missing docs root: {DOCS_ROOT}")
        return report(errors)

    for path in DOCS_ROOT.rglob("*"):
        if path.is_file() and path.suffix in {".md", ".mdx"}:
            text = path.read_text(encoding="utf-8")

            if path.name == "readme.md":
                errors.append(f"docs-site landing pages must use README.md, not readme.md: {path}")
            elif not is_valid_docs_filename(path):
                errors.append(f"invalid docs filename: {path}")

            if path.name != "index.mdx":
                if not has_frontmatter(text):
                    errors.append(f"missing frontmatter: {path}")
                else:
                    status = extract_status(text)
                    if status is None:
                        errors.append(f"missing status: {path}")
                    elif status not in ALLOWED_STATUS:
                        errors.append(f"invalid status '{status}': {path}")

            for stale, reason in STALE_PATTERNS.items():
                if contains_stale_pattern(text, stale):
                    errors.append(f"stale docs reference '{stale}' in {path}: {reason}")

            for match in MARKDOWN_LINK.finditer(text):
                raw_target = match.group(1)
                candidates = link_candidates(path, raw_target)
                if candidates and not any(candidate.exists() for candidate in candidates):
                    errors.append(f"broken markdown link in {path}: {raw_target}")

    return report(errors)

def report(errors: list[str]) -> int:
    if errors:
        print("docs validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print("docs validation passed")
    return 0

if __name__ == "__main__":
    sys.exit(main())
