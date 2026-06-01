from __future__ import annotations

from pathlib import Path

from roadmap_state import REPO_ROOT


MANIFEST_REPORT_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/track-execution-manifests"


def manifest_report_path(track_id: str, *, root: Path = MANIFEST_REPORT_ROOT) -> Path:
    return root / track_id.lower() / "manifest.md"
