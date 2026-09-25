#!/usr/bin/env python3
import argparse
import json
from html.parser import HTMLParser
from pathlib import Path
import re
import sys
import tomllib
import urllib.parse
from xml.etree import ElementTree

DOCS_ROOT = Path("docs-site/src/content/docs")
REPO_ROOT = Path(".")
IGNORED_DOCS_SUBTREES = {
    DOCS_ROOT / "reports" / "agent-transcripts",
}
DESIGN_LIFECYCLE_DIRS = {
    "active",
    "accepted",
    "implemented",
    "deferred",
    "superseded",
    "rejected",
    "archived",
}
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
PUBLICATION_CLASSES = {
    "primary",
    "reference",
    "repository-current",
    "history",
}
PUBLICATION_ROUTE_CLASSES = {"primary", "reference"}
RETIRED_PLANNING_LEDGER_PATTERNS = {
    "active-work.md": "GitHub issues and the Engineering Portfolio own live work state; the Markdown active-work ledger is retired",
    "deferred-work.md": "GitHub issues and the Engineering Portfolio own deferred work state; the Markdown deferred-work ledger is retired",
    "completed-work.md": "pull requests, reports, accepted documents, and Git history own completed evidence; the Markdown completed-work ledger is retired",
}
STALE_PATTERNS = {
    "engine/docs/": "engine docs moved under docs-site/src/content/docs/engine",
    "engine/README.md": "engine crate docs moved under docs-site/src/content/docs/engine/README.md",
    "engine/src/plugins/README.md": "engine plugin docs moved under docs-site/src/content/docs/engine/plugins/README.md",
    "engine/examples/README.md": "engine examples docs moved under docs-site/src/content/docs/engine/examples/overview.md",
    "engine/tests/README.md": "engine tests docs moved under docs-site/src/content/docs/engine/tests/README.md",
    "plugins/ui/README.md": "engine has no standalone ui plugin docs",
    "foundation/ids": "the current identity crate is foundation/id",
    "docs/design/": "design docs live under docs-site/src/content/docs/design",
    "docs/adr/": "ADRs live under docs-site/src/content/docs/adr",
    "scene_manager_ui": "the scene_manager_ui example is not present",
    "engine_net_quic/src/runtime/helpers.rs": "engine_net_quic runtime helpers.rs was removed",
    "engine_net_quic/src/runtime/utils.rs": "engine_net_quic runtime utils.rs was removed",
    "domain/editor/editor_shell/src/runtime/output/build_ui_frame.rs": "UI frame output moved to domain/ui/ui_runtime/src/output/build_ui_frame.rs",
    "no `impl TextLayouter`": "AtlasTextLayouter now implements TextLayouter",
    **RETIRED_PLANNING_LEDGER_PATTERNS,
}

MARKDOWN_LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
CRATE_INVENTORY_ROW = re.compile(
    r"^\|\s*`[^`]+`\s*\|\s*`((?:foundation|domain|engine|net|apps|adapters)[^`]*)`\s*\|",
    re.MULTILINE,
)
ACTIVE_DESIGN_COMPLETION_PATTERNS = [
    re.compile(r"^Status:\s*(?:complete|implemented)\b", re.IGNORECASE | re.MULTILINE),
    re.compile(r"\bThis design (?:is|was) implemented\b", re.IGNORECASE),
    re.compile(r"\bimplementation is at validated closeout-candidate\b", re.IGNORECASE),
    re.compile(r"^Status after .* closeout:\s*complete\b", re.IGNORECASE | re.MULTILINE),
]
ALLOWED_ACTIVE_LIFECYCLE_EXCEPTIONS = {"active_phase_evidence"}
DESIGN_STATUS_BY_DIR = {
    "active": "active",
    "accepted": "accepted",
    "implemented": "implemented",
    "deferred": "deferred",
    "superseded": "superseded",
    "rejected": "rejected",
    "archived": "archived",
}

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

def extract_frontmatter_value(text: str, key: str) -> str | None:
    if not has_frontmatter(text):
        return None
    frontmatter = text.split("\n---\n", 1)[0]
    match = re.search(rf"^{re.escape(key)}:\s*(.+?)\s*$", frontmatter, re.MULTILINE)
    return match.group(1).strip().strip("\"'") if match else None


def publication_class(path: Path, text: str, errors: list[str]) -> str | None:
    value = extract_frontmatter_value(text, "publication")
    if value is None:
        errors.append(f"missing publication classification: {path}")
        return None
    if value not in PUBLICATION_CLASSES:
        errors.append(f"invalid publication classification '{value}': {path}")
        return None

    draft = extract_frontmatter_value(text, "draft")
    pagefind = extract_frontmatter_value(text, "pagefind")
    if draft not in {None, "true", "false"}:
        errors.append(f"invalid draft value '{draft}': {path}")
    if pagefind not in {None, "true", "false"}:
        errors.append(f"invalid pagefind value '{pagefind}': {path}")

    if value == "primary":
        if draft == "true":
            errors.append(f"primary publication cannot be a draft: {path}")
        if pagefind == "false":
            errors.append(f"primary publication cannot disable Pagefind: {path}")
    elif value == "reference":
        if draft == "true":
            errors.append(f"reference publication must retain a production route: {path}")
        if pagefind != "false":
            errors.append(f"reference publication must disable default Pagefind: {path}")
    else:
        if draft != "true":
            errors.append(f"{value} publication must be a production draft: {path}")
        if pagefind != "false":
            errors.append(f"{value} publication must disable Pagefind: {path}")
        if not re.search(r"^sidebar:\s*$\n^\s+hidden:\s+true\s*$", text, re.MULTILINE):
            errors.append(f"{value} publication must hide its sidebar entry: {path}")
    return value


def docs_source_paths() -> list[Path]:
    return sorted(
        path
        for path in DOCS_ROOT.rglob("*")
        if path.is_file()
        and path.suffix in {".md", ".mdx"}
        and not any(path.is_relative_to(subtree) for subtree in IGNORED_DOCS_SUBTREES)
    )


def route_key(path: Path) -> str:
    relative = path.relative_to(DOCS_ROOT)
    if relative.name == "index.mdx" and relative.parent == Path("."):
        return "/"
    if relative.name in {"README.md", "index.md"}:
        relative = relative.parent / ("readme" if relative.name == "README.md" else "")
    else:
        relative = relative.with_suffix("")
    value = relative.as_posix().strip("/")
    return f"/{value}/" if value else "/"


class SidebarLinkParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self._sidebar_depth = 0
        self.links: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag == "nav" and "sidebar" in attributes.get("class", ""):
            self._sidebar_depth = 1
        elif self._sidebar_depth:
            self._sidebar_depth += 1
        if tag == "a" and self._sidebar_depth:
            href = attributes.get("href")
            if href:
                self.links.add(href)

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag: str) -> None:
        if self._sidebar_depth:
            self._sidebar_depth -= 1


def build_route_key(href: str, base: str = "/runenwerk") -> str | None:
    parsed = urllib.parse.urlparse(href)
    if parsed.scheme or parsed.netloc:
        return None
    path = parsed.path
    if not path.startswith(base):
        return None
    path = path[len(base) :]
    if not path.startswith("/"):
        return None
    return path if path.endswith("/") else f"{path}/"


def validate_publication_build(build_root: Path, errors: list[str]) -> None:
    if not build_root.is_dir():
        errors.append(f"missing documentation build output: {build_root}")
        return

    classes: dict[str, str] = {}
    for path in docs_source_paths():
        text = path.read_text(encoding="utf-8")
        value = extract_frontmatter_value(text, "publication")
        if value in PUBLICATION_CLASSES:
            classes[route_key(path)] = value

    expected_routes = {
        route for route, value in classes.items() if value in PUBLICATION_ROUTE_CLASSES
    }
    html_routes = {
        route_key_from_build_path(path, build_root)
        for path in build_root.rglob("index.html")
        if path.relative_to(build_root).as_posix() != "404.html"
    }
    html_routes.discard(None)
    if html_routes != expected_routes:
        errors.append(
            "generated route set does not match publication classes: "
            f"expected {len(expected_routes)}, found {len(html_routes)}"
        )

    sitemap_paths = sorted(build_root.glob("sitemap-*.xml"))
    sitemap_routes: set[str] = set()
    sitemap_namespace = {"sm": "http://www.sitemaps.org/schemas/sitemap/0.9"}
    for sitemap in sitemap_paths:
        root = ElementTree.parse(sitemap).getroot()
        for loc in root.findall("sm:url/sm:loc", sitemap_namespace):
            if loc.text:
                route = build_route_key(urllib.parse.urlparse(loc.text).path)
                if route is not None:
                    sitemap_routes.add(route)
    if sitemap_routes != expected_routes:
        errors.append(
            "generated sitemap set does not match publication classes: "
            f"expected {len(expected_routes)}, found {len(sitemap_routes)}"
        )

    pagefind_entry = build_root / "pagefind" / "pagefind-entry.json"
    try:
        pagefind_count = json.loads(pagefind_entry.read_text(encoding="utf-8"))["languages"]["en"][
            "page_count"
        ]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        errors.append(f"could not read generated Pagefind count: {error}")
        pagefind_count = -1
    expected_pagefind = sum(value == "primary" for value in classes.values())
    if pagefind_count != expected_pagefind:
        errors.append(
            "generated Pagefind count does not match primary publication: "
            f"expected {expected_pagefind}, found {pagefind_count}"
        )

    homepage = build_root / "index.html"
    sidebar = SidebarLinkParser()
    try:
        sidebar.feed(homepage.read_text(encoding="utf-8"))
    except OSError as error:
        errors.append(f"could not read generated homepage sidebar: {error}")
    sidebar_routes = {
        route
        for route in (build_route_key(href) for href in sidebar.links)
        if route is not None
    }
    if not expected_routes.issubset(sidebar_routes):
        missing = sorted(expected_routes - sidebar_routes)
        errors.append(f"generated sidebar omits routed publication pages: {missing[:5]}")
    invalid_sidebar_routes = sidebar_routes - expected_routes
    if invalid_sidebar_routes:
        errors.append(
            f"generated sidebar links to non-routed pages: {sorted(invalid_sidebar_routes)[:5]}"
        )

    print(
        "publication build passed: "
        f"source={len(classes)} "
        f"primary={sum(value == 'primary' for value in classes.values())} "
        f"reference={sum(value == 'reference' for value in classes.values())} "
        f"repository-current={sum(value == 'repository-current' for value in classes.values())} "
        f"history={sum(value == 'history' for value in classes.values())} "
        f"routes={len(expected_routes)} "
        f"pagefind={pagefind_count} "
        f"sitemap={len(sitemap_routes)} "
        f"sidebar={len(sidebar_routes)}"
    )


def route_key_from_build_path(path: Path, build_root: Path) -> str | None:
    relative = path.relative_to(build_root)
    if relative.as_posix() == "index.html":
        return "/"
    route = relative.parent.as_posix()
    return f"/{route}/"

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


def validate_publication_links(
    publication_by_path: dict[Path, str | None], errors: list[str]
) -> None:
    for source, publication in publication_by_path.items():
        if publication not in PUBLICATION_ROUTE_CLASSES:
            continue
        text = source.read_text(encoding="utf-8")
        for match in MARKDOWN_LINK.finditer(text):
            raw_target = match.group(1)
            for candidate in link_candidates(source, raw_target):
                target_publication = publication_by_path.get(candidate)
                if target_publication in {"repository-current", "history"}:
                    errors.append(
                        "routed publication links to non-routed publication "
                        f"{target_publication}: {source} -> {candidate}"
                    )
                    break

def repo_path_exists(path_text: str) -> bool:
    if path_text.startswith(("http://", "https://")):
        return True
    return (REPO_ROOT / path_text).exists()

def validate_design_lifecycle_indexes(errors: list[str]) -> None:
    design_root = DOCS_ROOT / "design"
    for directory_name in sorted(DESIGN_LIFECYCLE_DIRS):
        directory = design_root / directory_name
        if not directory.exists():
            continue
        sibling_docs = sorted(
            path for path in directory.glob("*.md") if path.name != "README.md"
        )
        if not sibling_docs:
            continue

        readme = directory / "README.md"
        if not readme.exists():
            continue

        text = readme.read_text(encoding="utf-8")
        linked_targets: set[Path] = set()
        for match in MARKDOWN_LINK.finditer(text):
            for candidate in link_candidates(readme, match.group(1)):
                if candidate.exists():
                    linked_targets.add(candidate)

        for sibling in sibling_docs:
            if sibling.resolve() not in linked_targets:
                errors.append(
                    f"design lifecycle index {readme} does not link sibling document: {sibling.name}"
                )

def validate_design_lifecycle_status(path: Path, text: str, errors: list[str]) -> None:
    if path.name == "README.md":
        return
    try:
        lifecycle_dir = path.relative_to(DOCS_ROOT / "design").parts[0]
    except ValueError:
        return
    expected = DESIGN_STATUS_BY_DIR.get(lifecycle_dir)
    if expected is None:
        return
    status = extract_status(text)
    if status != expected:
        errors.append(
            f"design lifecycle status mismatch in {path}: folder '{lifecycle_dir}' requires status '{expected}', found '{status}'"
        )

def validate_active_design_lifecycle_claims(path: Path, text: str, errors: list[str]) -> None:
    active_root = DOCS_ROOT / "design" / "active"
    if path.parent != active_root or path.name == "README.md":
        return
    lifecycle_exception = extract_frontmatter_value(text, "lifecycle_exception")
    has_completion_claim = any(pattern.search(text) for pattern in ACTIVE_DESIGN_COMPLETION_PATTERNS)
    if not has_completion_claim:
        return
    if lifecycle_exception in ALLOWED_ACTIVE_LIFECYCLE_EXCEPTIONS:
        return
    errors.append(
        f"active design contains implemented/completed lifecycle claim without allowed lifecycle_exception: {path}"
    )

def load_workspace_members(errors: list[str]) -> list[str]:
    cargo_toml = REPO_ROOT / "Cargo.toml"
    try:
        data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        errors.append(f"could not parse {cargo_toml}: {error}")
        return []
    members = data.get("workspace", {}).get("members", [])
    if not isinstance(members, list):
        errors.append("Cargo.toml workspace.members must be a list")
        return []
    return [member for member in members if isinstance(member, str)]

def validate_crate_inventory_alignment(errors: list[str]) -> None:
    members = load_workspace_members(errors)
    inventory_path = DOCS_ROOT / "workspace" / "crate-inventory.md"
    try:
        inventory_text = inventory_path.read_text(encoding="utf-8")
    except OSError as error:
        errors.append(f"could not read canonical crate inventory: {error}")
        return

    inventory_paths = CRATE_INVENTORY_ROW.findall(inventory_text)
    duplicates = sorted({path for path in inventory_paths if inventory_paths.count(path) > 1})
    for path in duplicates:
        errors.append(f"canonical crate inventory lists workspace member more than once: {path}")

    member_set = set(members)
    inventory_set = set(inventory_paths)
    for member in sorted(member_set - inventory_set):
        errors.append(f"canonical crate inventory missing current workspace member: {member}")
    for path in sorted(inventory_set - member_set):
        errors.append(f"canonical crate inventory lists non-workspace path as active member: {path}")

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--build-output",
        type=Path,
        help="also validate generated routes, Pagefind, sitemap, and sidebar output",
    )
    arguments = parser.parse_args()
    errors: list[str] = []

    if not DOCS_ROOT.exists():
        errors.append(f"missing docs root: {DOCS_ROOT}")
        return report(errors)

    validate_design_lifecycle_indexes(errors)
    validate_crate_inventory_alignment(errors)

    reports_root = DOCS_ROOT / "reports"
    publication_by_path: dict[Path, str | None] = {}

    for path in DOCS_ROOT.rglob("*"):
        if any(path.is_relative_to(subtree) for subtree in IGNORED_DOCS_SUBTREES):
            continue
        if path.is_file() and path.suffix in {".md", ".mdx"}:
            text = path.read_text(encoding="utf-8")
            validate_design_lifecycle_status(path, text, errors)
            validate_active_design_lifecycle_claims(path, text, errors)
            publication_by_path[path.resolve()] = publication_class(path, text, errors)

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
                if path.is_relative_to(reports_root) and stale in RETIRED_PLANNING_LEDGER_PATTERNS:
                    continue
                if contains_stale_pattern(text, stale):
                    errors.append(f"stale docs reference '{stale}' in {path}: {reason}")

            for match in MARKDOWN_LINK.finditer(text):
                raw_target = match.group(1)
                candidates = link_candidates(path, raw_target)
                if candidates and not any(candidate.exists() for candidate in candidates):
                    errors.append(f"broken markdown link in {path}: {raw_target}")

    validate_publication_links(publication_by_path, errors)

    if arguments.build_output is not None:
        validate_publication_build(arguments.build_output, errors)

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
