#!/usr/bin/env python3
"""Fail when current UX Lab contracts drift back to Story Lab vocabulary."""

from __future__ import annotations

from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[2]

BANNED_TERMS = (
    "Storybook",
    "Story Lab",
    "EditorUxStory",
    "PrimitiveWidgetStory",
    "story_lab",
    "editor_ux_story_lab",
)

CURRENT_PATHS = (
    "domain/editor/editor_shell/src",
    "domain/ui/ui_widgets/src",
    "apps/runenwerk_editor/src/shell",
    "docs-site/src/content/docs/design/active",
    "docs-site/src/content/docs/workspace/planning/roadmap.md",
    "docs-site/src/content/docs/workspace/planning/active-work.md",
)

TEXT_SUFFIXES = {".md", ".puml", ".rs", ".toml", ".txt", ".yaml", ".yml"}
HISTORICAL_NOTE = "Current name: UX Lab Scenarios; historical name: Story Lab."
HISTORICAL_CLOSEOUTS = (
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-001-governance-truth-audit-and-track-activation/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md",
    "docs-site/src/content/docs/reports/closeouts/pm-editor-ux-009-final-local-native-no-gap-certification/closeout.md",
)

def iter_text_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(
        candidate
        for candidate in path.rglob("*")
        if candidate.is_file() and candidate.suffix in TEXT_SUFFIXES
    )

def check_current_terms() -> list[str]:
    failures: list[str] = []
    for relative in CURRENT_PATHS:
        path = REPO_ROOT / relative
        if not path.exists():
            continue
        for file_path in iter_text_files(path):
            try:
                lines = file_path.read_text(encoding="utf-8").splitlines()
            except UnicodeDecodeError:
                continue
            for line_number, line in enumerate(lines, start=1):
                for term in BANNED_TERMS:
                    if term in line:
                        failures.append(
                            f"{file_path.relative_to(REPO_ROOT)}:{line_number}: "
                            f"banned UX Lab terminology '{term}'"
                        )
    return failures

def check_historical_notes() -> list[str]:
    failures: list[str] = []
    for relative in HISTORICAL_CLOSEOUTS:
        path = REPO_ROOT / relative
        if not path.exists():
            failures.append(f"{relative}: missing PM-EDITOR-UX closeout")
            continue
        text = path.read_text(encoding="utf-8")
        if HISTORICAL_NOTE not in text:
            failures.append(f"{relative}: missing terminology note '{HISTORICAL_NOTE}'")
    return failures

def main() -> int:
    failures = check_current_terms() + check_historical_notes()
    if failures:
        print("UX Lab terminology check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("UX Lab terminology check passed.")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
