from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_production_goal_unknown_track_fails(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-MISSING",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
        ],
    )

    assert result.exit_code == 1
    assert "PT-MISSING: not present in production tracks source" in result.stdout

def test_production_goal_renders_active_track_with_wr_links() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track)

    assert "Production Track /goal Kickoff: PT-TEST" in rendered
    assert "PM-TEST-001 - PM-TEST-001 title" in rendered
    assert "WR-001 WR-001 title: write_implementation_contract" in rendered
    assert "cargo test -p test" in rendered
    assert "tools/workflow" in rendered
    assert "Ready-to-paste /goal Prompt" in rendered

def test_production_goal_renders_manifest_gate_when_source_is_present(tmp_path: Path) -> None:
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    loaded_manifest = load_track_execution_manifest("PT-TEST", root=manifest_root)
    assert loaded_manifest is not None
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track, manifest=loaded_manifest)

    assert QUALITY_DOCTRINE_ID in rendered
    assert "## Track Execution Manifest Gate" in rendered
    assert f"Manifest source: `{repo_path(manifest_root / 'pt-test.yaml')}`" in rendered
    assert "- Current milestone: `PM-TEST-001` - PM-TEST-001 title" in rendered
    assert "- Manifest next legal action: Execute the bounded milestone action." in rendered
    assert "Implementation authorized now: no" in rendered
    assert "Stop after the current legal action and rerun this command" in rendered

def test_production_goal_completed_manifest_track_has_no_stale_unmet_wr_gate(tmp_path: Path) -> None:
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    loaded_manifest = load_track_execution_manifest("PT-TEST", root=manifest_root)
    assert loaded_manifest is not None
    production_data = valid_production_state()
    for milestone in production_data["tracks"][0]["milestones"]:
        milestone["state"] = "completed"
        milestone["completion_quality"] = "bounded_contract"
    roadmap_data = valid_state()
    for roadmap_item in roadmap_data["items"]:
        roadmap_item["planning_state"] = "completed"
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(roadmap_data)
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track, manifest=loaded_manifest)

    assert "Workflow next action: already_completed" in rendered
    assert "Unmet gates: none detected by manifest-aware workflow checks." in rendered
    assert "workflow action is linked_row_completed" not in rendered
    assert "Unmet gates:\n  - WR-" not in rendered

def test_production_goal_manifest_conflict_fails_closed(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["owning_wr"] = "WR-999"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-TEST",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "Track Execution Manifest audit blockers" in result.stdout
    assert "alignment errors:" in result.stdout
    assert "manifest owning_wr WR-999" in result.stdout
    assert "roadmap_links ['WR-001']" in result.stdout
    assert "Ready-to-paste /goal Prompt" not in result.stdout

def test_production_goal_warns_when_long_track_has_no_manifest() -> None:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"] = [
        production_milestone(f"PM-TEST-{index:03}", dependencies=[] if index == 1 else [f"PM-TEST-{index - 1:03}"])
        for index in range(1, 7)
    ]
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track)

    assert "Manifest source: not found" in rendered
    assert "long production track" in rendered

def test_production_goal_cli_keeps_non_manifest_fallback(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"] = [
        production_milestone(f"PM-TEST-{index:03}", dependencies=[] if index == 1 else [f"PM-TEST-{index - 1:03}"])
        for index in range(1, 7)
    ]
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-TEST",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest source: not found" in result.stdout
    assert "long production track" in result.stdout
    assert "Ready-to-paste /goal Prompt" in result.stdout

def test_production_goal_completed_milestone_renders_evidence_verification() -> None:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["state"] = "completed"
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track)

    assert "- Next legal action: verify_completed_evidence" in rendered
    assert "completed production milestones must include evidence gates" in rendered

def test_production_goal_designing_milestone_renders_design_first() -> None:
    production_data = valid_production_state()
    milestone = production_data["tracks"][0]["milestones"][0]
    milestone["kind"] = "design"
    milestone["state"] = "designing"
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    steps = build_goal_steps(planning, roadmap, track)

    assert steps[0].next_action == "write_or_accept_design_before_implementation"

def test_production_goal_active_design_without_wr_routes_to_design_evidence() -> None:
    production_data = valid_production_state()
    milestone = production_data["tracks"][0]["milestones"][0]
    milestone["kind"] = "design"
    milestone["state"] = "active"
    milestone["roadmap_links"] = []
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    steps = build_goal_steps(planning, roadmap, track)

    assert steps[0].next_action == "accept_design_or_record_design_evidence"

def test_production_goal_active_milestone_uses_wr_action_classification() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    steps = build_goal_steps(planning, roadmap, track)

    assert steps[0].roadmap_actions[0].action == "write_implementation_contract"
    assert steps[0].next_action == "execute_next_wr_implementation_contract"

def test_production_goal_invalid_scope_fails(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-TEST",
            "--scope",
            "sideways",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
        ],
    )

    assert result.exit_code != 0
    assert "invalid" in result.output.lower()

