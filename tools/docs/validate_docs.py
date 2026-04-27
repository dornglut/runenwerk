#!/usr/bin/env python3
from pathlib import Path
import re
import sys

DOCS_ROOT = Path("docs-site/src/content/docs")
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