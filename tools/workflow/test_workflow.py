from __future__ import annotations

from pathlib import Path


def test_workflow_tests_are_split_by_responsibility() -> None:
    test_dir = Path(__file__).parent / "tests"
    modules = sorted(path.name for path in test_dir.glob("test_*.py"))
    assert len(modules) >= 12
    assert "test_execution_kernel.py" not in modules
    assert "test_roadmap_batch.py" not in modules
    assert "test_truth_conformance.py" in modules
    assert "test_track_control.py" in modules
    assert "test_prompt_doctrine.py" in modules
    assert "test_production_manifest.py" in modules
    assert "test_production_goal.py" in modules
    assert "test_production_state.py" in modules
    assert "test_production_plan.py" in modules
    assert "test_roadmap_batching.py" in modules
    assert "test_roadmap_intake.py" in modules
    assert "test_roadmap_state.py" in modules
    assert "test_workflow_misc.py" not in modules
    for path in test_dir.glob("test_*.py"):
        assert len(path.read_text(encoding="utf-8").splitlines()) <= 1_000, path


def test_workflow_fixtures_are_split_by_responsibility() -> None:
    test_dir = Path(__file__).parent / "tests"
    fixture_dir = test_dir / "fixtures"
    assert (test_dir / "workflow_fixtures.py").read_text(encoding="utf-8").strip() == (
        "from fixtures.source_model import *\nfrom fixtures.execution_kernel import *"
    )
    assert (fixture_dir / "source_model.py").exists()
    assert (fixture_dir / "execution_kernel.py").exists()
    assert (fixture_dir / "roadmap.py").exists()
    assert (fixture_dir / "truth.py").exists()
    for path in fixture_dir.glob("*.py"):
        assert len(path.read_text(encoding="utf-8").splitlines()) <= 1_200, path


def test_workflow_uses_real_suite_command() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    authority_paths = [
        repo_root / "docs-site/src/content/docs/workspace/track-execution-manifests/pt-track-execution-harness.yaml",
        repo_root / "docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program-architecture.yaml",
    ]
    authority_paths.extend(
        (repo_root / "docs-site/src/content/docs/reports/implementation-plans").glob("wr-*/plan.contract.yaml")
    )
    stale_command = "uv run pytest tools/workflow/test_workflow.py -q"
    for path in authority_paths:
        if path.exists():
            assert stale_command not in path.read_text(encoding="utf-8"), path
    assert Path(__file__).stat().st_size < 3_000
