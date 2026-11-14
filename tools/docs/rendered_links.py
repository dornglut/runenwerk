"""Validate user-visible hyperlinks against generated Starlight HTML and assets."""

from collections import Counter
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit

ORIGIN = "https://dornglut.github.io"
BASE = "/runenwerk"


class PageLinks(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.in_main = False
        self.has_main = False
        self.links: list[str] = []
        self.ids: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag == "main":
            self.has_main = True
            self.in_main = True
        if attributes.get("id") is not None:
            self.ids.add(attributes["id"])
        if tag == "a":
            if attributes.get("name") is not None:
                self.ids.add(attributes["name"])
            if self.in_main and attributes.get("href"):
                self.links.append(attributes["href"])

    def handle_endtag(self, tag: str) -> None:
        if tag == "main":
            self.in_main = False


@dataclass(frozen=True)
class Finding:
    kind: str
    page: str
    href: str
    detail: str = ""


def inspect_links(build_root: Path) -> tuple[dict[str, int], list[Finding]]:
    if not build_root.is_dir():
        raise ValueError(f"missing generated documentation: {build_root}")

    root = build_root.resolve()
    html_files = sorted(root.rglob("index.html"))
    if not html_files:
        raise ValueError(f"no generated documentation routes: {root}")

    pages: dict[Path, PageLinks] = {}
    for file in html_files:
        parsed = PageLinks()
        parsed.feed(file.read_text(encoding="utf-8"))
        pages[file] = parsed

    findings: list[Finding] = []
    counts = Counter(pages=len(pages))

    for file, page in pages.items():
        relative_route = file.relative_to(root).parent.as_posix()
        route = f"{BASE}/{relative_route}/" if relative_route != "." else f"{BASE}/"
        route = route.replace("//", "/")
        page_url = ORIGIN + route
        if not page.has_main:
            findings.append(Finding("missing-main", route, ""))
            continue
        for href in page.links:
            counts["links"] += 1
            try:
                uri = urlsplit(urljoin(page_url, href))
            except ValueError:
                findings.append(Finding("invalid-url", route, href))
                continue
            if uri.scheme in {"mailto", "tel"}:
                counts["external"] += 1
                continue
            if uri.scheme not in {"http", "https"}:
                findings.append(Finding("unsupported-scheme", route, href))
                continue
            if uri.hostname != "dornglut.github.io":
                counts["external"] += 1
                continue
            counts["internal"] += 1
            if not (uri.path == BASE or uri.path.startswith(BASE + "/")):
                findings.append(Finding("outside-base", route, href, uri.path))
                continue
            try:
                path = unquote(uri.path[len(BASE):].lstrip("/"), errors="strict")
            except UnicodeDecodeError:
                findings.append(Finding("bad-encoding", route, href))
                continue
            dest = (root / path).resolve()
            if not dest.is_relative_to(root):
                findings.append(Finding("outside-build", route, href))
                continue
            if uri.path.endswith("/") or uri.path == BASE:
                dest /= "index.html"
            if not dest.is_file():
                findings.append(Finding("missing-target", route, href, uri.path))
                continue
            if dest.name != "index.html":
                counts["assets"] += 1
                continue
            if uri.fragment and not uri.fragment.startswith(":~:text="):
                fragment = unquote(uri.fragment)
                if dest not in pages:
                    findings.append(Finding("unlisted-target", route, href))
                elif fragment not in pages[dest].ids:
                    findings.append(Finding("missing-anchor", route, href, fragment))

    findings.sort(key=lambda item: (item.kind, item.page, item.href, item.detail))
    return dict(counts), findings


def append_link_errors(build_root: Path, errors: list[str], limit: int = 80) -> None:
    """Append actionable, bounded diagnostics to the existing publication validator."""
    try:
        counts, findings = inspect_links(build_root)
    except (OSError, ValueError) as error:
        errors.append(str(error))
        return
    kinds = Counter(item.kind for item in findings)
    print("rendered link audit: " + " ".join(f"{k}={v}" for k, v in sorted(counts.items()))
          + f" broken={len(findings)}" + (f" causes={dict(kinds)}" if kinds else ""))
    for item in findings[:limit]:
        errors.append(f"{item.kind}: {item.page} -> {item.href}" + (f" [{item.detail}]" if item.detail else ""))
    if len(findings) > limit:
        errors.append(f"{len(findings) - limit} additional rendered link failures (first {limit} shown)")
