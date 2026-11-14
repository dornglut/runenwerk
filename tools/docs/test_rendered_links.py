"""Focused generated-site hyperlink tests; stdlib only."""

import tempfile
import unittest
from pathlib import Path

from rendered_links import inspect_links


def html(body: str, extra: str = "") -> str:
    return ('<html><body><nav><a href="/broken-nav/">nav</a></nav>'
            '<main id="main-content">' + body + '</main>' + extra + '</body></html>')


def link(href: str) -> str:
    return f'<a href="{href}">link</a>'


class RenderedLinkTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def add(self, path: str, content: str) -> None:
        file = self.root / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(content, encoding="utf-8")

    def kinds(self) -> list[str]:
        return sorted(item.kind for item in inspect_links(self.root)[1])

    def test_root_relative_base_query_and_hash(self) -> None:
        self.add("index.html", html("".join(map(link, [
            "/runenwerk/engine/", "./engine/", "./engine/#heading", "#main-content", "?q=1",
            "https://dornglut.github.io/runenwerk/engine/#heading",
            "https://example.org/", "mailto:team@example.org", "tel:+123"
        ]))))
        self.add("engine/index.html", html('<h2 id="heading">Example</h2>' + link('../')))
        self.assertEqual(self.kinds(), [])

    def test_readme_route_and_asset(self) -> None:
        self.add("index.html", html(link('/runenwerk/engine/readme/#intro') + link('/runenwerk/manual.pdf')))
        self.add("engine/readme/index.html", html('<h2 id="intro">Intro</h2>'))
        self.add("manual.pdf", "fixture")
        self.assertEqual(self.kinds(), [])

    def test_missing_target_anchor_base_and_scheme(self) -> None:
        self.add("index.html", html("".join(map(link, [
            "/runenwerk/engine/README.md", "/runenwerk/missing/", "/runenwerk/engine/#bad",
            "/engine/", "javascript:alert(1)"
        ]))))
        self.add("engine/index.html", html('<h2 id="ok">ok</h2>'))
        self.assertEqual(self.kinds(), sorted([
            "missing-target", "missing-target", "missing-anchor", "outside-base", "unsupported-scheme"
        ]))

    def test_navigation_and_escaped_code_are_not_content_links(self) -> None:
        self.add("index.html", html('<pre>&lt;a href="/broken-code/"&gt;</pre>' + link('#main-content')))
        self.assertEqual(self.kinds(), [])

    def test_html_attribute_entities_and_named_anchors(self) -> None:
        self.add("index.html", html('<a href="/runenwerk/engine/?q=1&amp;x=2#legacy">docs</a>'))
        self.add("engine/index.html", html('<a name="legacy"></a>'))
        self.assertEqual(self.kinds(), [])

    def test_absent_build_and_missing_main_fail(self) -> None:
        self.add("index.html", '<html><body>no main</body></html>')
        self.assertEqual(self.kinds(), ["missing-main"])
        with self.assertRaises(ValueError):
            inspect_links(self.root / "missing")


if __name__ == "__main__":
    unittest.main()
