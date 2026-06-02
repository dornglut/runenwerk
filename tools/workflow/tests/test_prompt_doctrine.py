from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_production_plan_contract_prompt_includes_quality_doctrine(tmp_path: Path) -> None:
    context = production_plan_context()
    prompt = render_contract_prompt(context, classify_plan_action(context), tmp_path / "plan.md")

    assert QUALITY_DOCTRINE_ID in prompt

def test_ai_task_prompt_includes_quality_doctrine() -> None:
    prompt = render_shape(build_shapes()["implementation"], task="Test task", scope="tools/workflow", roadmap="WR-001")

    assert QUALITY_DOCTRINE_ID in prompt

def test_prompt_doctrine_audit_passes_current_repository() -> None:
    assert audit_prompt_doctrine(repo_root=REPO_ROOT) == []

def test_production_goal_full_scope_keeps_complete_track_prompt() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track)

    assert "/goal Complete production track PT-TEST - Test production track." in rendered
    assert "The production track is complete only when every milestone is completed" in rendered

def test_production_goal_completion_prompt_includes_render_validate_check_gates() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track)

    assert "task production:render" in rendered
    assert "task production:validate" in rendered
    assert "task production:check" in rendered
    assert "task roadmap:render" in rendered
    assert "task roadmap:validate" in rendered
    assert "task roadmap:check" in rendered

def test_ai_task_list_includes_goal_shape() -> None:
    assert "goal" in build_shapes()

def test_batch_manifest_and_worker_prompt() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest("batch-test", "test", [roadmap.items[0]], Path("docs-site/src/content/docs/reports/batches/batch-test"))
    assert manifest.items[0].branch == "codex/batch-test-wr-001"
    assert manifest.items[0].prompt_path.endswith("/wr-001.md")
    absolute_manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        REPO_ROOT / "docs-site/src/content/docs/reports/batches/batch-test",
    )
    assert absolute_manifest.items[0].prompt_path == "docs-site/src/content/docs/reports/batches/batch-test/prompts/wr-001.md"
    prompt = render_worker_prompt(manifest, roadmap.items[0], manifest.items[0])
    assert prompt.startswith("---\ntitle: Worker Prompt WR-001")
    assert QUALITY_DOCTRINE_ID in prompt
    assert "roadmap-items.yaml" in prompt

def test_hygiene_rejects_missing_batch_prompt_path() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    item_with_missing_prompt = manifest.items[0].model_copy(update={"prompt_path": "docs-site/src/content/docs/reports/batches/missing/prompts/wr-001.md"})
    manifest = manifest.model_copy(update={"items": [item_with_missing_prompt]})

    assert batch_manifest_errors(Path("reports/batch-test/batch.toml"), manifest) == [
        "reports/batch-test/batch.toml:WR-001: prompt_path does not exist: "
        "docs-site/src/content/docs/reports/batches/missing/prompts/wr-001.md"
    ]
