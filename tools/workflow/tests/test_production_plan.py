from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_production_plan_valid_link_prints_expected_contract_path() -> None:
    result = CliRunner().invoke(
        production_plan_app,
        ["plan", "--milestone", "PM-SDF-OW-001", "--roadmap", "WR-019"],
    )

    assert result.exit_code == 0
    assert "docs-site/src/content/docs/reports/implementation-plans/wr-019-field-visualizer-product-workflow/plan.md" in result.stdout
    assert "Next action:" in result.stdout

def test_production_plan_unlinked_wr_fails(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    with pytest.raises(WorkflowError, match="not linked"):
        resolve_plan_context("PM-TEST-001", "WR-002", production_source=production_path, roadmap_source=roadmap_path)

def test_production_plan_unknown_milestone_fails(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    with pytest.raises(WorkflowError, match="not present in production tracks"):
        resolve_plan_context("PM-TEST-999", "WR-001", production_source=production_path, roadmap_source=roadmap_path)

def test_production_plan_unknown_wr_fails(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    with pytest.raises(WorkflowError, match="not present in combined roadmap sources"):
        resolve_plan_context("PM-TEST-001", "WR-999", production_source=production_path, roadmap_source=roadmap_path)

def test_production_plan_default_command_does_not_write_scaffold(tmp_path: Path) -> None:
    target = tmp_path / "plan.md"
    result = CliRunner().invoke(
        production_plan_app,
        ["plan", "--milestone", "PM-SDF-OW-001", "--roadmap", "WR-019", "--out", str(target)],
    )

    assert result.exit_code == 0
    assert not target.exists()

def test_invalid_planning_state_is_rejected() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "implement_now"

    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)

def test_ready_next_row_classifies_as_promotion_contract() -> None:
    context = production_plan_context(roadmap_state="ready_next")

    assert classify_plan_action(context) == "write_promotion_contract"

def test_unmet_gate_classifies_as_design_first() -> None:
    context = production_plan_context(
        roadmap_state="current_candidate",
        production_gate=production_design_gate(
            "docs-site/src/content/docs/design/active/sdf-prefab-composition-system-design.md"
        ),
    )

    assert classify_plan_action(context) == "design_first"

def test_completed_row_classifies_as_already_completed() -> None:
    context = production_plan_context(roadmap_state="completed")

    assert classify_plan_action(context) == "already_completed"

def test_write_scaffold_writes_and_refuses_overwrite(tmp_path: Path) -> None:
    context = production_plan_context()
    target = tmp_path / "plan.md"

    write_contract_scaffold(context, "write_implementation_contract", target)

    assert target.exists()
    with pytest.raises(WorkflowError, match="already exists"):
        write_contract_scaffold(context, "write_implementation_contract", target)
    write_contract_scaffold(context, "write_implementation_contract", target, force=True)

def test_default_contract_path_uses_wr_id_and_title_slug() -> None:
    context = production_plan_context()

    assert repo_path(default_contract_path(context.roadmap_item)).endswith(
        "docs-site/src/content/docs/reports/implementation-plans/wr-001-wr-001-title/plan.md"
    )

