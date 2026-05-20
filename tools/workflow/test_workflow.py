from __future__ import annotations

import tempfile
import subprocess
from pathlib import Path

import pytest
import yaml
from typer.testing import CliRunner

from generate_roadmap_docs import render_current_candidates_roadmap, render_dependency_roadmap, render_outputs
from generate_production_docs import stale_outputs as stale_production_outputs
from parallel_batch import (
    app as batch_app,
    batch_finalization_errors,
    batch_execution_state,
    build_manifest,
    continuation_items_for_manifest,
    default_batch_id,
    default_continuation_goal,
    default_kickoff_goal,
    finalize_batch_manifest,
    kickoff_next_step_lines,
    path_matches_ref,
    render_batch_report,
    run_host_batch_validation,
    run_official_batch_validation,
    refresh_base_manifest,
    render_worker_prompt,
    validation_commands_for_items,
    write_validation_result,
    worktree_path_for_item,
)
from production_state import (
    ProductionPlanningState,
    validate_completion_quality as validate_production_completion_quality,
    validate_design_gates,
    validate_roadmap_links,
)
from production_plan import (
    ProductionPlanContext,
    app as production_plan_app,
    classify_plan_action,
    default_contract_path,
    resolve_plan_context,
    write_contract_scaffold,
)
from production_goal import (
    GoalScope,
    app as production_goal_app,
    build_goal_steps,
    find_track,
    render_track_goal,
)
from roadmap_state import (
    REPO_ROOT,
    BatchManifest,
    RoadmapState,
    WorkflowError,
    changed_files_for_worktree,
    combine_roadmap_data,
    document_frontmatter_status,
    load_batch_manifest,
    load_roadmap,
    load_yaml,
    promotion_preflight,
    repo_path,
    render_batch_manifest,
    select_batch_candidates,
    validate_batch_against_roadmap,
    validate_completed_items_not_current_in_docs,
    validate_completion_evidence,
    validate_completion_quality,
    validate_changed_paths,
    validate_existing_write_scope_paths,
    validate_roadmap_with_json_schema,
    validate_write_scopes,
    write_schema_files,
)
from ai_task import build_shapes
from repo_hygiene import batch_manifest_errors, local_branches
from roadmap_intake import (
    apply_intake_proposal,
    build_intake_proposal,
    load_intake_proposal,
    proposal_to_yaml_data,
    roadmap_data_with_promotion,
    roadmap_data_with_proposal,
    switch_current_candidate,
    validate_intake_item_scopes,
    write_intake_proposal,
)


def valid_state() -> dict:
    return {
        "version": 1,
        "roadmap": {"title": "Test Roadmap", "last_reviewed": "2026-05-14", "owner": "workspace"},
        "render": {
            "decision_register": "decision.md",
            "dependency_roadmap": "roadmap.puml",
            "current_candidates_roadmap": "candidates.puml",
            "triage": "triage.md",
        },
        "items": [
            item("WR-001", dependencies=[], write_scopes=["tools/workflow"]),
            item("WR-002", value=2, blocker=5, dependencies=["WR-001"], write_scopes=["docs-site"]),
        ],
        "edges": [{"source": "WR-001", "target": "WR-002", "label": "depends"}],
    }


def valid_state_with_switch_target() -> dict:
    state = valid_state()
    state["items"].append(
        item(
            "WR-003",
            planning_state="ready_next",
            dependencies=[],
            write_scopes=["tools/workflow/production_plan.py"],
        )
    )
    return state


def item(
    item_id: str,
    *,
    value: int = 4,
    blocker: int = 2,
    planning_state: str = "current_candidate",
    dependencies: list[str] | None = None,
    write_scopes: list[str] | None = None,
) -> dict:
    return {
        "id": item_id,
        "title": f"{item_id} title",
        "diagram_title": f"{item_id} diagram",
        "alias": item_id.replace("-", ""),
        "lane": "Core",
        "dependency_level": "L0",
        "gate": "Supporting now" if blocker < 5 else "Policy deferred",
        "planning_state": planning_state,
        "priority": "P0",
        "value": value,
        "blocker": blocker,
        "tc": 3,
        "rr_oe": 5,
        "du": 5,
        "effort": 5,
        "confidence": 0.8,
        "expected_score": round(((value + 3 + 5 + 5) * 0.8) / 5, 1),
        "rice": "N/A",
        "kano": "Neutral",
        "dependencies": dependencies or [],
        "write_scopes": write_scopes or ["tools/workflow"],
        "validations": ["cargo test -p test"],
        "next_evidence": "Evidence.",
        "current_decision": "Decision.",
        "current_call": "Current call.",
        "first_move": "First move.",
        "main_blocker": "Main blocker.",
        "why_not_ready": "Why not ready.",
        "diagram_call": ["call"],
        "ddd_owner": "owner",
        "adr_requirement": "none",
        "fitness_function_requirement": "tests",
        "ownership_mode": "stream-aligned",
    }


def decision_gate(path: str, required_status: str = "accepted") -> dict:
    return {
        "kind": "adr",
        "path": path,
        "required_status": required_status,
        "applies_to": "implementation",
        "reason": "Test decision gate.",
    }


def valid_production_state() -> dict:
    return {
        "version": 1,
        "production": {"title": "Test Production Tracks", "last_reviewed": "2026-05-16", "owner": "workspace"},
        "render": {
            "production_index": "production-index.md",
            "milestone_register": "production-register.md",
            "track_roadmap": "production-roadmap.puml",
        },
        "tracks": [
            {
                "id": "PT-TEST",
                "title": "Test production track",
                "state": "active",
                "owner": "workspace",
                "strategic_goal": "Prove the production planning model.",
                "success_criteria": ["The fixture validates."],
                "milestones": [
                    production_milestone("PM-TEST-001", roadmap_links=["WR-001"]),
                    production_milestone(
                        "PM-TEST-002",
                        kind="design",
                        state="active",
                        dependencies=["PM-TEST-001"],
                        roadmap_links=["WR-002"],
                    ),
                ],
            }
        ],
    }


def production_milestone(
    milestone_id: str,
    *,
    kind: str = "implementation",
    state: str = "active",
    dependencies: list[str] | None = None,
    roadmap_links: list[str] | None = None,
    design_gates: list[dict] | None = None,
) -> dict:
    return {
        "id": milestone_id,
        "title": f"{milestone_id} title",
        "kind": kind,
        "state": state,
        "goal": "Milestone goal.",
        "outcome": "Milestone outcome.",
        "dependencies": dependencies or [],
        "roadmap_links": roadmap_links or [],
        "design_gates": design_gates or [],
        "evidence_gates": [],
        "acceptance_criteria": ["Acceptance criterion."],
    }


def production_design_gate(path: str, required_status: str = "accepted") -> dict:
    return {
        "kind": "design",
        "path": path,
        "required_status": required_status,
        "reason": "Test production decision gate.",
    }


def production_plan_context(
    *,
    roadmap_item_id: str = "WR-001",
    roadmap_state: str = "current_candidate",
    production_gate: dict | None = None,
) -> ProductionPlanContext:
    roadmap_data = valid_state()
    for roadmap_item in roadmap_data["items"]:
        if roadmap_item["id"] == roadmap_item_id:
            roadmap_item["planning_state"] = roadmap_state
            if roadmap_state == "ready_next":
                roadmap_item["main_blocker"] = "Needs promotion evidence."
            if roadmap_state == "completed":
                roadmap_item["write_scopes"].append(
                    "docs-site/src/content/docs/reports/closeouts/sdf-first-execution-phase-1/closeout.md"
                )
                roadmap_item["next_evidence"] = (
                    "docs-site/src/content/docs/reports/closeouts/sdf-first-execution-phase-1/closeout.md"
                )
    roadmap = RoadmapState.model_validate(roadmap_data)
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["roadmap_links"] = [roadmap_item_id]
    if production_gate:
        production_data["tracks"][0]["milestones"][0]["design_gates"] = [production_gate]
    planning = ProductionPlanningState.model_validate(production_data)
    track = planning.tracks[0]
    milestone = track.milestones[0]
    return ProductionPlanContext(
        planning=planning,
        roadmap=roadmap,
        track=track,
        milestone=milestone,
        roadmap_item=roadmap.by_id[roadmap_item_id],
    )


def write_yaml(path: Path, data: dict) -> None:
    path.write_text(yaml.safe_dump(data, sort_keys=False), encoding="utf-8", newline="\n")


def test_a_wsjf_score_is_computed() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    assert roadmap.items[0].score == 2.7


def test_invalid_blocker_is_rejected() -> None:
    state = valid_state()
    state["items"][0]["blocker"] = 6
    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)


def test_duplicate_ids_are_rejected() -> None:
    state = valid_state()
    state["items"][1]["id"] = "WR-001"
    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)


def test_missing_dependency_is_rejected() -> None:
    state = valid_state()
    state["items"][0]["dependencies"] = ["WR-999"]
    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)


def test_b5_items_are_excluded_from_implementation_batch() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    selected = select_batch_candidates(roadmap, level="L0")
    assert [item.id for item in selected] == ["WR-001"]


def test_only_current_candidates_enter_implementation_batch() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "support_only"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Supporting now"
    state["items"][1]["planning_state"] = "completed"
    roadmap = RoadmapState.model_validate(state)

    assert select_batch_candidates(roadmap, level="L0") == []


def test_explicit_completed_or_support_only_items_are_rejected() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Supporting now"
    state["items"][1]["planning_state"] = "support_only"
    roadmap = RoadmapState.model_validate(state)

    with pytest.raises(WorkflowError, match="planning_state 'completed' is not current_candidate"):
        select_batch_candidates(roadmap, item_ids=("WR-001",))
    with pytest.raises(WorkflowError, match="planning_state 'support_only' is not current_candidate"):
        select_batch_candidates(roadmap, item_ids=("WR-002",))


def test_unmet_decision_gate_excludes_and_rejects_current_candidate() -> None:
    state = valid_state()
    state["items"][0]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]
    roadmap = RoadmapState.model_validate(state)

    assert select_batch_candidates(roadmap, level="L0") == []
    with pytest.raises(WorkflowError, match="decision gate unmet"):
        select_batch_candidates(roadmap, item_ids=("WR-001",))


def test_accepted_decision_gate_allows_current_candidate() -> None:
    state = valid_state()
    state["items"][0]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md")
    ]
    roadmap = RoadmapState.model_validate(state)

    assert [item.id for item in select_batch_candidates(roadmap, level="L0")] == ["WR-001"]


def test_valid_production_track_fixture_passes() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())

    assert validate_roadmap_links(planning) == []
    assert validate_design_gates(planning) == []


def test_completed_production_milestones_require_completion_quality(tmp_path: Path) -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["state"] = "completed"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(roadmap_path, valid_state())
    planning = ProductionPlanningState.model_validate(state)

    assert validate_production_completion_quality(planning, roadmap_path=roadmap_path) == [
        "PM-TEST-001: completed production milestones must set completion_quality"
    ]


def test_perfectionist_production_quality_rejects_gaps_missing_audit_and_non_verified_wr(tmp_path: Path) -> None:
    state = valid_production_state()
    milestone = state["tracks"][0]["milestones"][0]
    milestone["state"] = "completed"
    milestone["completion_quality"] = "perfectionist_verified"
    milestone["known_quality_gaps"] = ["linked WR still has gaps"]
    roadmap_data = valid_state()
    roadmap_data["items"][0]["planning_state"] = "completed"
    roadmap_data["items"][0]["completion_quality"] = "bounded_contract"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(roadmap_path, roadmap_data)
    planning = ProductionPlanningState.model_validate(state)

    assert validate_production_completion_quality(planning, roadmap_path=roadmap_path) == [
        "PM-TEST-001: perfectionist_verified milestones must not list known_quality_gaps",
        "PM-TEST-001: perfectionist_verified milestones must reference a completed audit",
        "PM-TEST-001: perfectionist_verified milestone links WR-001 with completion_quality='bounded_contract'",
    ]


def test_perfectionist_production_quality_accepts_completed_audit_and_verified_wrs(tmp_path: Path) -> None:
    audit_path = "docs-site/src/content/docs/reports/audits/pm-test-001-audit.md"
    audit = tmp_path / audit_path
    audit.parent.mkdir(parents=True)
    audit.write_text("---\nstatus: completed\n---\n# Audit\n", encoding="utf-8")
    state = valid_production_state()
    milestone = state["tracks"][0]["milestones"][0]
    milestone["state"] = "completed"
    milestone["completion_quality"] = "perfectionist_verified"
    milestone["completion_audit"] = audit_path
    roadmap_data = valid_state()
    roadmap_data["items"][0]["planning_state"] = "completed"
    roadmap_data["items"][0]["completion_quality"] = "perfectionist_verified"
    roadmap_data["items"][0]["completion_audit"] = audit_path
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(roadmap_path, roadmap_data)
    planning = ProductionPlanningState.model_validate(state)

    assert validate_production_completion_quality(planning, roadmap_path=roadmap_path, repo_root=tmp_path) == []


def test_duplicate_production_track_ids_are_rejected() -> None:
    state = valid_production_state()
    duplicate = dict(state["tracks"][0])
    duplicate["milestones"] = [production_milestone("PM-OTHER-001", roadmap_links=["WR-001"])]
    state["tracks"].append(duplicate)

    with pytest.raises(ValueError, match="duplicate production track ids"):
        ProductionPlanningState.model_validate(state)


def test_duplicate_production_milestone_ids_are_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"].append(production_milestone("PM-TEST-001", roadmap_links=["WR-001"]))

    with pytest.raises(ValueError, match="duplicate production milestone ids"):
        ProductionPlanningState.model_validate(state)


def test_missing_production_milestone_dependency_is_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["dependencies"] = ["PM-TEST-999"]

    with pytest.raises(ValueError, match="unknown milestone dependency"):
        ProductionPlanningState.model_validate(state)


def test_production_milestone_dependency_cycle_is_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["dependencies"] = ["PM-TEST-002"]

    with pytest.raises(ValueError, match="production milestone dependency cycle"):
        ProductionPlanningState.model_validate(state)


def test_missing_wr_roadmap_link_is_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["roadmap_links"] = ["WR-999"]
    planning = ProductionPlanningState.model_validate(state)

    assert validate_roadmap_links(planning) == ["PM-TEST-001: unknown roadmap link WR-999"]


def test_active_implementation_milestone_with_unmet_design_gate_fails() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["design_gates"] = [
        production_design_gate("docs-site/src/content/docs/design/active/sdf-prefab-composition-system-design.md")
    ]
    planning = ProductionPlanningState.model_validate(state)

    errors = validate_design_gates(planning)
    assert errors
    assert "does not match required 'accepted'" in errors[0]


def test_active_design_milestone_may_resolve_unmet_design_gate() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][1]["design_gates"] = [
        production_design_gate("docs-site/src/content/docs/design/active/sdf-prefab-composition-system-design.md")
    ]
    planning = ProductionPlanningState.model_validate(state)

    assert validate_design_gates(planning) == []


def test_generated_production_docs_stale_check_detects_difference(tmp_path: Path) -> None:
    generated = tmp_path / "production-track-index.md"
    generated.write_text("old\n", encoding="utf-8")

    assert stale_production_outputs({generated: "new\n"})


def test_non_open_world_production_track_validates() -> None:
    state = valid_production_state()
    state["tracks"][0]["id"] = "PT-DRAW"
    state["tracks"][0]["title"] = "Drawing production track"
    state["tracks"][0]["strategic_goal"] = "Prove drawing workflow production planning."
    state["tracks"][0]["milestones"] = [
        production_milestone("PM-DRAW-001", roadmap_links=["WR-001"])
    ]
    planning = ProductionPlanningState.model_validate(state)

    assert validate_roadmap_links(planning) == []
    assert validate_design_gates(planning) == []


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


def test_ready_next_row_classifies_as_promotion_contract() -> None:
    context = production_plan_context(roadmap_state="ready_next")

    assert classify_plan_action(context) == "write_promotion_contract"


def test_ready_next_row_classifies_switch_when_current_candidate_blocks_scope() -> None:
    roadmap = RoadmapState.model_validate(valid_state_with_switch_target())
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["roadmap_links"] = ["WR-003"]
    planning = ProductionPlanningState.model_validate(production_data)
    context = ProductionPlanContext(
        planning=planning,
        roadmap=roadmap,
        track=planning.tracks[0],
        milestone=planning.tracks[0].milestones[0],
        roadmap_item=roadmap.by_id["WR-003"],
    )

    assert classify_plan_action(context) == "switch_current_candidate"


def test_current_candidate_row_classifies_as_implementation_contract() -> None:
    context = production_plan_context(roadmap_state="current_candidate")

    assert classify_plan_action(context) == "write_implementation_contract"


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


def test_production_plan_default_command_does_not_write_scaffold(tmp_path: Path) -> None:
    target = tmp_path / "plan.md"
    result = CliRunner().invoke(
        production_plan_app,
        ["plan", "--milestone", "PM-SDF-OW-001", "--roadmap", "WR-019", "--out", str(target)],
    )

    assert result.exit_code == 0
    assert not target.exists()


def test_production_plan_renders_switch_current_preflight(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["roadmap_links"] = ["WR-003"]
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, valid_state_with_switch_target())

    result = CliRunner().invoke(
        production_plan_app,
        [
            "plan",
            "--milestone",
            "PM-TEST-001",
            "--roadmap",
            "WR-003",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Next action: switch_current_candidate" in result.stdout
    assert "## Promotion Preflight" in result.stdout
    assert "task roadmap:switch-current -- --from WR-001 --to WR-003" in result.stdout


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


def test_production_goal_full_scope_keeps_complete_track_prompt() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track)

    assert "/goal Complete production track PT-TEST - Test production track." in rendered
    assert "The production track is complete only when every milestone is completed" in rendered


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


def test_production_goal_active_milestone_uses_wr_action_classification() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_state())
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    steps = build_goal_steps(planning, roadmap, track)

    assert steps[0].roadmap_actions[0].action == "write_implementation_contract"
    assert steps[0].next_action == "execute_next_wr_implementation_contract"


def test_production_goal_surfaces_switch_current_candidate_action() -> None:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["roadmap_links"] = ["WR-003"]
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state_with_switch_target())
    track = find_track(planning, "PT-TEST")

    steps = build_goal_steps(planning, roadmap, track)
    rendered = render_track_goal(planning, roadmap, track)

    assert steps[0].roadmap_actions[0].action == "switch_current_candidate"
    assert steps[0].next_action == "switch_current_candidate"
    assert "After a failed roadmap:promote or gate command" in rendered


def test_production_goal_non_deferred_scope_preserves_blocked_milestones() -> None:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"].append(
        production_milestone(
            "PM-TEST-003",
            kind="design",
            state="blocked",
            dependencies=["PM-TEST-002"],
        )
    )
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state())
    track = find_track(planning, "PT-TEST")

    rendered = render_track_goal(planning, roadmap, track, scope=GoalScope.non_deferred)

    assert "/goal Complete the non-deferred scope of production track PT-TEST - Test production track." in rendered
    assert "Do not implement blocked or deferred milestones; preserve them as explicit deferred gaps." in rendered
    assert "Preserved out-of-scope milestones:" in rendered
    assert "- PM-TEST-003: blocked - PM-TEST-003 title" in rendered
    assert "- Bounded goal scope: preserved out of scope; do not implement for `--scope non-deferred`." in rendered
    assert "- PM-TEST-003: wait_for_dependency_completion (preserved out of scope: blocked)" in rendered
    assert "PM-TEST-003: state is 'blocked', expected 'completed'" not in rendered


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


def test_document_frontmatter_status_handles_crlf(tmp_path: Path) -> None:
    doc = tmp_path / "adr.md"
    doc.write_text("---\r\nstatus: accepted\r\n---\r\n# ADR\r\n", encoding="utf-8", newline="")

    assert document_frontmatter_status(doc) == "accepted"


def test_ready_next_rows_may_carry_future_implementation_gates() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "ready_next"
    state["items"][0]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]
    roadmap = RoadmapState.model_validate(state)

    assert [item.id for item in select_batch_candidates(roadmap, item_ids=("WR-001",), include_discovery=True)] == [
        "WR-001"
    ]


def test_invalid_planning_state_is_rejected() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "implement_now"

    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)


def test_overlapping_write_scopes_are_detected() -> None:
    state = valid_state()
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Supporting now"
    state["items"][1]["write_scopes"] = ["tools/workflow/subsystem"]
    roadmap = RoadmapState.model_validate(state)
    assert validate_write_scopes(roadmap.items) == ["WR-002:tools/workflow/subsystem overlaps WR-001:tools/workflow"]


def test_missing_write_scope_paths_are_detected() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = ["apps/a"]
    roadmap = RoadmapState.model_validate(state)
    assert validate_existing_write_scope_paths([roadmap.items[0]]) == ["WR-001:apps/a does not exist"]


def test_render_check_can_detect_stale_files() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        state = valid_state()
        state["render"] = {
            "decision_register": str(root / "decision.md"),
            "dependency_roadmap": str(root / "roadmap.puml"),
            "current_candidates_roadmap": str(root / "candidates.puml"),
            "triage": str(root / "triage.md"),
        }
        source = root / "roadmap.yaml"
        source.write_text(yaml.safe_dump(state, sort_keys=False), encoding="utf-8")
        triage = root / "triage.md"
        triage.write_text(
            "---\nstatus: active\n---\n\n# Triage\n\n## Implement Now\n\nold\n\n## Design Lifecycle Cleanup Findings\n\ntext\n",
            encoding="utf-8",
        )
        roadmap = load_roadmap(source)
        outputs = render_outputs(roadmap)
        assert any(not path.exists() or path.read_text(encoding="utf-8") != expected for path, expected in outputs.items())


def test_generated_roadmap_diagrams_separate_dependency_truth_from_candidates() -> None:
    state = valid_state()
    active_item = item(
        "WR-002",
        blocker=2,
        dependencies=["WR-001"],
        write_scopes=["docs-site"],
    )
    active_item["gate"] = "Ready next"
    state["items"] = [active_item]
    state["edges"] = [{"source": "WR-001", "target": "WR-002", "label": "depends"}]
    archived = {
        "version": state["version"],
        "roadmap": state["roadmap"],
        "items": [item("WR-001", planning_state="completed", dependencies=[], write_scopes=["tools/workflow"])],
    }
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            state,
            archive_data=archived,
            deferred_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
        )
    )

    dependency = render_dependency_roadmap(roadmap)
    candidates = render_current_candidates_roadmap(roadmap)

    assert "Level 0 - Support Substrate" in dependency
    assert "Parallel" + " Now" not in dependency
    assert "state=completed" not in dependency
    assert "WR-001" not in dependency
    assert "Current Implementation Candidates" in candidates
    assert "state=current_candidate" in candidates
    assert "state=completed" not in candidates
    assert "Immediate Dependency Context" not in candidates
    assert "WR-001" not in candidates


def test_schema_generation_check_detects_missing_files() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        from roadmap_state import BATCH_SCHEMA, ROADMAP_ITEM_SOURCE_SCHEMA, ROADMAP_SCHEMA

        assert ROADMAP_SCHEMA.name == "roadmap-items.schema.json"
        assert ROADMAP_ITEM_SOURCE_SCHEMA.name == "roadmap-item-source.schema.json"
        assert BATCH_SCHEMA.name == "batch-manifest.schema.json"


def test_split_roadmap_sources_combine_active_archive_and_deferred_rows() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        state = valid_state()
        state["items"] = [
            item("WR-030", dependencies=["WR-001"], write_scopes=["tools/workflow"]),
        ]
        state["edges"] = [{"source": "WR-001", "target": "WR-030", "label": "baseline"}]
        archive = {
            "version": state["version"],
            "roadmap": state["roadmap"],
            "items": [item("WR-001", planning_state="completed", blocker=1, dependencies=[])],
        }
        deferred = {
            "version": state["version"],
            "roadmap": state["roadmap"],
            "items": [item("WR-011", planning_state="blocked_deferred", blocker=5, dependencies=[])],
        }
        source = root / "roadmap-items.yaml"
        source.write_text(yaml.safe_dump(state, sort_keys=False), encoding="utf-8")
        (root / "roadmap-archive.yaml").write_text(yaml.safe_dump(archive, sort_keys=False), encoding="utf-8")
        (root / "roadmap-deferred.yaml").write_text(yaml.safe_dump(deferred, sort_keys=False), encoding="utf-8")

        roadmap = load_roadmap(source)
        validate_roadmap_with_json_schema(source)

        assert [item.id for item in roadmap.active_items] == ["WR-030"]
        assert [item.id for item in roadmap.archived_items] == ["WR-001"]
        assert [item.id for item in roadmap.deferred_items] == ["WR-011"]
        assert sorted(roadmap.by_id) == ["WR-001", "WR-011", "WR-030"]


def test_split_roadmap_sources_reject_completed_or_deferred_active_rows() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    combined = combine_roadmap_data(
        state,
        archive_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
        deferred_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
    )

    with pytest.raises(ValueError, match="active roadmap source must not contain completed items"):
        RoadmapState.model_validate(combined)


def test_split_roadmap_sources_reject_wrong_archive_or_deferred_states() -> None:
    state = valid_state()
    state["items"] = [item("WR-030", dependencies=[])]
    state["edges"] = []
    archive = {
        "version": state["version"],
        "roadmap": state["roadmap"],
        "items": [item("WR-001", planning_state="ready_next", blocker=2, dependencies=[])],
    }
    combined = combine_roadmap_data(
        state,
        archive_data=archive,
        deferred_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
    )

    with pytest.raises(ValueError, match="archive source items must be completed"):
        RoadmapState.model_validate(combined)


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
    assert "roadmap-items.yaml" in prompt


def test_batch_kickoff_defaults_to_current_candidates() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    selected = select_batch_candidates(roadmap)
    manifest = build_manifest("batch-test", default_kickoff_goal(selected), selected, Path("reports/batch-test"))

    assert [item.id for item in selected] == ["WR-001"]
    assert manifest.goal == "Next current-candidate roadmap batch: WR-001 WR-001 title"
    assert kickoff_next_step_lines(Path("reports/batch-test/batch.toml"), manifest) == [
        "task batch:approve -- --batch reports/batch-test/batch.toml",
        "task batch:prepare -- --batch reports/batch-test/batch.toml",
        "task batch:validate -- --batch reports/batch-test/batch.toml",
        "task batch:worker-prompt -- --batch reports/batch-test/batch.toml --item WR-001",
        "task batch:scope-check -- --batch reports/batch-test/batch.toml",
        "task batch:finalize -- --batch reports/batch-test/batch.toml --target main --write --cleanup",
    ]


def test_current_repository_next_batch_selects_wr032() -> None:
    roadmap = load_roadmap()
    selected = select_batch_candidates(roadmap)
    dependency_puml = (REPO_ROOT / "docs-site/src/content/docs/workspace/diagrams/value-weighted-dependency-roadmap.puml").read_text(encoding="utf-8")
    candidates_puml = (REPO_ROOT / "docs-site/src/content/docs/workspace/diagrams/current-roadmap-candidates.puml").read_text(encoding="utf-8")

    assert [item.id for item in selected] == ["WR-032"]
    assert roadmap.by_id["WR-029"].planning_state == "ready_next"
    assert "state=completed" not in dependency_puml
    assert "state=completed" not in candidates_puml


def test_batch_kickoff_writes_manifest_from_cli() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        source = root / "roadmap.yaml"
        source.write_text(yaml.safe_dump(valid_state(), sort_keys=False), encoding="utf-8")
        out = root / "batch.toml"

        result = CliRunner().invoke(
            batch_app,
            [
                "kickoff",
                "--next",
                "--source",
                str(source),
                "--out",
                str(out),
                "--batch-id",
                "batch-test",
                "--goal",
                "test goal",
            ],
        )

        assert result.exit_code == 0, result.output
        manifest = load_batch_manifest(out)
        assert manifest.id == "batch-test"
        assert manifest.goal == "test goal"
        assert manifest.approval_state == "proposed"
        assert [item.id for item in manifest.items] == ["WR-001"]
        assert "task batch:approve" in result.output


def test_batch_kickoff_rejects_when_no_current_candidates() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        state = valid_state()
        for candidate in state["items"]:
            candidate["planning_state"] = "support_only"
            candidate["blocker"] = 2
            candidate["gate"] = "Supporting now"
        source = Path(temp_dir) / "roadmap.yaml"
        source.write_text(yaml.safe_dump(state, sort_keys=False), encoding="utf-8")

        result = CliRunner().invoke(batch_app, ["kickoff", "--next", "--source", str(source)])

        assert result.exit_code != 0
        assert "no current_candidate items are eligible" in result.output


def test_batch_continue_selects_still_current_finalized_items() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    finalized_item = manifest.items[0].model_copy(
        update={
            "status": "integrated",
            "roadmap_outcome": "slice_landed_item_still_current",
        }
    )
    finalized = manifest.model_copy(
        update={
            "integration_status": "merged",
            "closeout_status": "completed",
            "items": [finalized_item],
        }
    )

    selected = continuation_items_for_manifest(finalized, roadmap)

    assert [item.id for item in selected] == ["WR-001"]
    assert default_continuation_goal(finalized, selected) == "Continue roadmap batch after batch-test: WR-001"


def test_batch_continue_rejects_open_batches() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )

    with pytest.raises(WorkflowError, match="batch must be finalized"):
        continuation_items_for_manifest(manifest, roadmap)


def test_batch_continue_writes_followup_manifest_from_cli() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        roadmap = RoadmapState.model_validate(valid_state())
        manifest = build_manifest(
            "batch-test",
            "test",
            [roadmap.items[0]],
            Path("docs-site/src/content/docs/reports/batches/batch-test"),
        )
        finalized_item = manifest.items[0].model_copy(
            update={
                "status": "integrated",
                "roadmap_outcome": "slice_landed_item_still_current",
            }
        )
        finalized = manifest.model_copy(
            update={
                "integration_status": "merged",
                "closeout_status": "completed",
                "items": [finalized_item],
            }
        )
        root = Path(temp_dir)
        source = root / "roadmap.yaml"
        batch = root / "batch.toml"
        out = root / "followup.toml"
        source.write_text(yaml.safe_dump(valid_state(), sort_keys=False), encoding="utf-8")
        batch.write_text(render_batch_manifest(finalized), encoding="utf-8")

        result = CliRunner().invoke(
            batch_app,
            [
                "continue",
                "--batch",
                str(batch),
                "--source",
                str(source),
                "--out",
                str(out),
                "--batch-id",
                "followup",
            ],
        )

        assert result.exit_code == 0, result.output
        followup = load_batch_manifest(out)
        assert followup.id == "followup"
        assert followup.approval_state == "proposed"
        assert [item.id for item in followup.items] == ["WR-001"]
        assert "task batch:approve" in result.output


def test_flat_worktree_path_avoids_batch_id_nesting() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "very-long-batch-id",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/very-long-batch-id"),
    )

    assert worktree_path_for_item(Path("C:/w"), manifest, manifest.items[0], flat_worktrees=True) == Path("C:/w/WR-001")
    assert worktree_path_for_item(Path("C:/w"), manifest, manifest.items[0], flat_worktrees=False) == Path(
        "C:/w/very-long-batch-id/WR-001"
    )


def test_batch_approval_validation_rejects_blocked_items() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[1]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )

    assert validate_batch_against_roadmap(manifest, roadmap) == [
        "WR-002: B5 is above the B2 implementation gate"
    ]


def test_batch_approval_validation_rejects_stale_scope() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    stale_item = manifest.items[0].model_copy(update={"write_scopes": ["docs-site"]})
    stale_manifest = manifest.model_copy(update={"items": [stale_item]})

    assert validate_batch_against_roadmap(stale_manifest, roadmap) == [
        "WR-001: write_scopes are stale"
    ]


def test_scope_enforcement_rejects_out_of_scope_paths() -> None:
    violations = validate_changed_paths(["apps/a/file.rs", "engine/src/lib.rs"], ["apps/a"])
    assert violations == ["engine/src/lib.rs"]


def test_scope_enforcement_allows_generated_roadmap_outputs_for_roadmap_source() -> None:
    violations = validate_changed_paths(
        [
            "docs-site/src/content/docs/workspace/roadmap-decision-register.md",
            "docs-site/src/content/docs/workspace/diagrams/value-weighted-dependency-roadmap.puml",
            "docs-site/src/content/docs/workspace/diagrams/current-roadmap-candidates.puml",
            "docs-site/src/content/docs/workspace/design-implementation-triage.md",
            "docs-site/src/content/docs/workspace/other.md",
        ],
        ["docs-site/src/content/docs/workspace/roadmap-items.yaml"],
    )

    assert violations == ["docs-site/src/content/docs/workspace/other.md"]


def test_default_batch_id_keeps_roadmap_item_ids_after_slug_truncation() -> None:
    draw_id = default_batch_id("Next current-candidate roadmap batch: WR-006 Runenwerk Draw DRF4 through DRF5")
    multi_id = default_batch_id("Next current-candidate roadmap batch: WR-025, WR-018, WR-007")

    assert draw_id.endswith("next-current-candidate-roadmap-batch-wr-006")
    assert multi_id.endswith("next-current-candidate-roadmap-batch-wr-025-wr-018-wr-007")
    assert draw_id != multi_id


def test_changed_files_for_worktree_includes_dirty_staged_and_untracked_files() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        worktree = Path(temp_dir)
        subprocess.run(["git", "init"], cwd=worktree, check=True, stdout=subprocess.DEVNULL)
        subprocess.run(["git", "config", "user.name", "Workflow Test"], cwd=worktree, check=True)
        subprocess.run(["git", "config", "user.email", "workflow@example.invalid"], cwd=worktree, check=True)

        (worktree / "tracked.txt").write_text("base\n", encoding="utf-8")
        subprocess.run(["git", "add", "tracked.txt"], cwd=worktree, check=True)
        subprocess.run(["git", "commit", "-m", "base"], cwd=worktree, check=True, stdout=subprocess.DEVNULL)
        base_sha = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=worktree,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout.strip()

        (worktree / "tracked.txt").write_text("dirty\n", encoding="utf-8")
        (worktree / "staged.txt").write_text("staged\n", encoding="utf-8")
        subprocess.run(["git", "add", "staged.txt"], cwd=worktree, check=True)
        (worktree / "untracked.txt").write_text("untracked\n", encoding="utf-8")

        assert changed_files_for_worktree(worktree, base_sha) == [
            "staged.txt",
            "tracked.txt",
            "untracked.txt",
        ]


def test_changed_files_for_worktree_keeps_standard_status_long_path_warnings(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def fake_run(command: list[str], **_kwargs) -> subprocess.CompletedProcess[str]:
        stdout = ""
        if "status" in command and "-c" not in command:
            stdout = " D too/long/cache/file.cache\n"
        return subprocess.CompletedProcess(command, 0, stdout, "")

    monkeypatch.setattr("roadmap_state.subprocess.run", fake_run)

    assert changed_files_for_worktree(Path("worker"), "base") == ["too/long/cache/file.cache"]


def test_changed_files_for_worktree_ignores_status_only_modified_noise(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def fake_run(command: list[str], **_kwargs) -> subprocess.CompletedProcess[str]:
        stdout = ""
        if "status" in command:
            stdout = " M docs/generated.md\n"
        return subprocess.CompletedProcess(command, 0, stdout, "")

    monkeypatch.setattr("roadmap_state.subprocess.run", fake_run)

    assert changed_files_for_worktree(Path("worker"), "base") == []


def test_batch_validation_rejects_dirty_out_of_scope_worktree(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})

    monkeypatch.setattr("parallel_batch.changed_paths_for_item", lambda _item, _base_sha: ["docs-site/out.md"])

    _selected, errors = batch_execution_state(approved, roadmap)

    assert errors == ["WR-001: changed path outside approved scope: docs-site/out.md"]


def test_batch_validation_invokes_host_commands_only_after_scope_checks_pass(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})
    calls: list[tuple[str, Path]] = []

    monkeypatch.setattr("parallel_batch.changed_paths_for_item", lambda _item, _base_sha: ["tools/workflow/file.py"])

    selected, output = run_official_batch_validation(
        approved,
        roadmap,
        command_runner=lambda command, cwd: calls.append((command, cwd)) or "ok",
    )

    assert [item.id for item in selected] == ["WR-001"]
    expected_cwd = REPO_ROOT / "worker"
    expected_label = str(expected_cwd).replace("\\", "/")
    assert calls == [("cargo test -p test", expected_cwd)]
    assert output == f"[WR-001] {expected_label}\n> cargo test -p test\nok\n"


def test_batch_validation_does_not_invoke_host_commands_when_scope_fails(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})
    calls: list[tuple[str, Path]] = []

    monkeypatch.setattr("parallel_batch.changed_paths_for_item", lambda _item, _base_sha: ["docs-site/out.md"])

    with pytest.raises(WorkflowError):
        run_official_batch_validation(
            approved,
            roadmap,
            command_runner=lambda command, cwd: calls.append((command, cwd)) or "ok",
        )

    assert calls == []


def test_host_batch_validation_runs_commands_in_order() -> None:
    calls: list[str] = []

    output = run_host_batch_validation(
        ["first command", "second command"],
        command_runner=lambda command: calls.append(command) or f"{command} output",
    )

    assert calls == ["first command", "second command"]
    assert output == "> first command\nfirst command output\n> second command\nsecond command output\n"


def test_validation_results_are_written_only_by_explicit_write() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    ).model_copy(update={"approval_state": "approved"})

    with tempfile.TemporaryDirectory() as temp_dir:
        batch_path = Path(temp_dir) / "batch.toml"
        batch_path.write_text("", encoding="utf-8")

        assert manifest.validation_results == []
        write_validation_result(batch_path, manifest, "passed", validation_commands_for_items(manifest.items))
        updated = load_batch_manifest(batch_path)

    assert len(updated.validation_results) == 1
    assert "batch validate passed" in updated.validation_results[0]


def test_finalize_refuses_dirty_in_scope_worker_changes(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})

    monkeypatch.setattr("parallel_batch.git_output", lambda command: "target" if "main" in command else "")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: False)
    monkeypatch.setattr("parallel_batch.changed_files_for_worktree", lambda _worktree, _base_sha: ["tools/workflow/file.py"])
    monkeypatch.setattr("parallel_batch.path_matches_ref", lambda _worktree, _target, _path: False)
    monkeypatch.setattr("parallel_batch.Path.exists", lambda _path: True)

    assert batch_finalization_errors(approved, "main") == [
        "WR-001: dirty in-scope worker change is not integrated into main: tools/workflow/file.py"
    ]


def test_finalize_refuses_unmerged_worker_commits(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )

    monkeypatch.setattr("parallel_batch.git_output", lambda command: "target" if "main" in command else "branch")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: True)
    monkeypatch.setattr("parallel_batch.branch_is_ancestor", lambda _branch, _target: False)

    assert batch_finalization_errors(manifest, "main") == [
        "WR-001: worker branch codex/batch-test-wr-001 has commits not integrated into main"
    ]


def test_path_matches_ref_respects_git_text_normalization() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        repo = Path(temp_dir)
        subprocess.run(["git", "init"], cwd=repo, check=True, stdout=subprocess.DEVNULL)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.invalid"], cwd=repo, check=True)
        (repo / ".gitattributes").write_text("*.md text\n", encoding="utf-8", newline="\n")
        (repo / "note.md").write_text("one\ntwo\n", encoding="utf-8", newline="\n")
        subprocess.run(["git", "add", ".gitattributes", "note.md"], cwd=repo, check=True)
        subprocess.run(["git", "commit", "-m", "seed"], cwd=repo, check=True, stdout=subprocess.DEVNULL)

        (repo / "note.md").write_text("one\ntwo\n", encoding="utf-8", newline="\r\n")

        assert path_matches_ref(repo, "HEAD", "note.md")


def test_finalize_cleans_integrated_worktrees_and_branches(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker", "status": "slice_completed"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})
    removed_worktrees: list[str] = []
    deleted_branches: list[str] = []

    monkeypatch.setattr("parallel_batch.git_output", lambda _command: "abc123")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: True)
    monkeypatch.setattr("parallel_batch.branch_is_ancestor", lambda _branch, _target: True)
    monkeypatch.setattr("parallel_batch.changed_files_for_worktree", lambda _worktree, _base_sha: ["tools/workflow/file.py"])
    monkeypatch.setattr("parallel_batch.path_matches_ref", lambda _worktree, _target, _path: True)
    monkeypatch.setattr("parallel_batch.Path.exists", lambda _path: True)
    monkeypatch.setattr("parallel_batch.remove_worker_worktree_if_present", lambda path: removed_worktrees.append(str(path)))
    monkeypatch.setattr("parallel_batch.delete_worker_branch", lambda branch: deleted_branches.append(branch))

    finalized = finalize_batch_manifest(approved, roadmap, target="main", cleanup=True)

    assert removed_worktrees == ["worker"]
    assert deleted_branches == ["codex/batch-test-wr-001"]
    assert finalized.integration_status == "merged"
    assert finalized.closeout_status == "completed"
    assert finalized.integrated_target == "main"
    assert finalized.items[0].status == "integrated"
    assert finalized.items[0].roadmap_outcome == "slice_landed_item_still_current"
    assert finalized.items[0].worktree == ""
    assert finalized.items[0].worktree_cleanup == "cleaned after integration"


def test_finalize_preserves_branches_when_requested(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    deleted_branches: list[str] = []

    monkeypatch.setattr("parallel_batch.git_output", lambda _command: "abc123")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: True)
    monkeypatch.setattr("parallel_batch.branch_is_ancestor", lambda _branch, _target: True)
    monkeypatch.setattr("parallel_batch.delete_worker_branch", lambda branch: deleted_branches.append(branch))

    finalize_batch_manifest(manifest, roadmap, target="main", cleanup=True, keep_branches=True)

    assert deleted_branches == []


def test_batch_report_renders_cleaned_worktrees_and_roadmap_outcome() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    item = manifest.items[0].model_copy(
        update={
            "worktree": "",
            "worktree_cleanup": "cleaned after integration",
            "status": "integrated",
            "roadmap_outcome": "slice_landed_item_still_current",
        }
    )
    finalized = manifest.model_copy(
        update={
            "integration_status": "merged",
            "closeout_status": "completed",
            "integrated_target": "main",
            "integrated_sha": "abc123",
            "items": [item],
        }
    )
    report = render_batch_report(finalized)

    assert "Integrated into: main@abc123" in report
    assert "- Worktree: `cleaned after integration`" in report
    assert "- Roadmap outcome: `slice_landed_item_still_current`" in report


def test_hygiene_rejects_finalized_manifest_with_stale_worktree() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    item_with_stale_worktree = manifest.items[0].model_copy(
        update={"worktree": "worker", "prompt_path": "docs-site/src/content/docs/workspace/roadmap-index.md"}
    )
    finalized = manifest.model_copy(
        update={"integration_status": "merged", "closeout_status": "completed", "items": [item_with_stale_worktree]}
    )

    assert batch_manifest_errors(Path("reports/batch-test/batch.toml"), finalized) == [
        "reports/batch-test/batch.toml:WR-001: finalized batch still records active worktree "
        + str((REPO_ROOT / "worker")).replace("\\", "/")
    ]


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


def test_hygiene_uses_portable_merged_branch_option_order(monkeypatch: pytest.MonkeyPatch) -> None:
    commands: list[list[str]] = []

    def fake_git_stdout(command: list[str]) -> str:
        commands.append(command)
        return "main\ncodex/done\n"

    monkeypatch.setattr("repo_hygiene.git_stdout", fake_git_stdout)

    assert local_branches(merged_only=True) == ["main", "codex/done"]
    assert commands == [["branch", "--format=%(refname:short)", "--merged"]]


def test_roadmap_completion_rejects_vague_evidence_without_path() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = "Validated and complete."
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items) == [
        "WR-001: completed items must reference an existing completed closeout or batch evidence path"
    ]


def test_roadmap_completion_rejects_missing_closeout_path(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    state["items"][0]["write_scopes"] = [evidence_path]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == [
        f"WR-001: completion evidence path does not exist: {evidence_path}"
    ]


def test_roadmap_completion_rejects_non_completed_closeout_frontmatter(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    closeout = tmp_path / evidence_path
    closeout.parent.mkdir(parents=True)
    closeout.write_text("---\nstatus: draft\n---\n# Draft\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    state["items"][0]["write_scopes"] = [evidence_path]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == [
        f"WR-001: completion closeout evidence status 'draft' is not 'completed': {evidence_path}"
    ]


def test_roadmap_completion_requires_evidence_path_in_write_scopes(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    closeout = tmp_path / evidence_path
    closeout.parent.mkdir(parents=True)
    closeout.write_text("---\nstatus: completed\n---\n# Done\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == [
        "WR-001: completed items must include a completed closeout or batch evidence path in write_scopes"
    ]


def test_roadmap_completion_accepts_completed_closeout_evidence(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    closeout = tmp_path / evidence_path
    closeout.parent.mkdir(parents=True)
    closeout.write_text("---\nstatus: completed\n---\n# Done\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    state["items"][0]["write_scopes"] = [evidence_path]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == []


def test_completed_roadmap_items_require_completion_quality() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items) == [
        "WR-001: completed items must set completion_quality"
    ]


def test_perfectionist_roadmap_quality_rejects_gaps_or_missing_audit(tmp_path: Path) -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["completion_quality"] = "perfectionist_verified"
    state["items"][0]["known_quality_gaps"] = ["still has a UI gap"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items, repo_root=tmp_path) == [
        "WR-001: perfectionist_verified items must not list known_quality_gaps",
        "WR-001: perfectionist_verified items must reference a completed audit",
    ]


def test_perfectionist_roadmap_quality_requires_completed_audit(tmp_path: Path) -> None:
    audit_path = "docs-site/src/content/docs/reports/audits/wr-001-audit.md"
    audit = tmp_path / audit_path
    audit.parent.mkdir(parents=True)
    audit.write_text("---\nstatus: draft\n---\n# Audit\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["completion_quality"] = "perfectionist_verified"
    state["items"][0]["completion_audit"] = audit_path
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items, repo_root=tmp_path) == [
        f"WR-001: completion_audit status 'draft' is not 'completed': {audit_path}"
    ]


def test_perfectionist_roadmap_quality_accepts_completed_audit(tmp_path: Path) -> None:
    audit_path = "docs-site/src/content/docs/reports/audits/wr-001-audit.md"
    audit = tmp_path / audit_path
    audit.parent.mkdir(parents=True)
    audit.write_text("---\nstatus: completed\n---\n# Audit\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["completion_quality"] = "perfectionist_verified"
    state["items"][0]["completion_audit"] = audit_path
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items, repo_root=tmp_path) == []


def test_completed_items_are_rejected_from_current_docs(tmp_path: Path) -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = "Closeout evidence."
    roadmap = RoadmapState.model_validate(state)
    doc = tmp_path / "roadmap-index.md"
    doc.write_text("WR-001 is the current implementation candidate.\n", encoding="utf-8")
    expected_path = str(doc).replace("\\", "/")

    assert validate_completed_items_not_current_in_docs(roadmap.items, [doc]) == [
        f"{expected_path}:1: completed item WR-001 is still described as current work"
    ]


def test_refresh_base_is_blocked_after_integration_starts() -> None:
    manifest = BatchManifest(
        id="batch-test",
        goal="test",
        approval_state="approved",
        base_branch="main",
        base_sha="abc123",
        integration_risk="isolated worktrees",
        integration_status="integrating",
        items=[],
    )

    with pytest.raises(WorkflowError, match="integration_status"):
        refresh_base_manifest(manifest, base="main")


def test_refresh_base_rejects_dirty_worker_changes_by_default(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})

    monkeypatch.setattr("parallel_batch.changed_paths_for_item", lambda _item, _base_sha: ["docs-site/out.md"])

    with pytest.raises(WorkflowError, match="dirty worker worktree changes"):
        refresh_base_manifest(approved, base="main", recreate_worktrees=True)


def test_refresh_base_discards_stale_out_of_scope_worktrees_when_explicit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})
    removed: list[str] = []

    monkeypatch.setattr("parallel_batch.changed_paths_for_item", lambda _item, _base_sha: ["docs-site/out.md"])
    monkeypatch.setattr("parallel_batch.git_output", lambda _args: "newbase")
    monkeypatch.setattr("parallel_batch.remove_worker_worktrees_and_branches", lambda _manifest: removed.append("removed"))

    refreshed = refresh_base_manifest(
        approved,
        base="main",
        recreate_worktrees=True,
        discard_stale_worktrees=True,
    )

    assert removed == ["removed"]
    assert refreshed.base_sha == "newbase"
    assert refreshed.items[0].worktree == ""
    assert "base refreshed" in refreshed.integration_risk


def test_refresh_base_still_rejects_dirty_in_scope_changes_with_discard(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})
    removed: list[str] = []

    monkeypatch.setattr("parallel_batch.changed_paths_for_item", lambda _item, _base_sha: ["tools/workflow/file.py"])
    monkeypatch.setattr("parallel_batch.remove_worker_worktrees_and_branches", lambda _manifest: removed.append("removed"))

    with pytest.raises(WorkflowError, match="dirty in-scope worker changes"):
        refresh_base_manifest(
            approved,
            base="main",
            recreate_worktrees=True,
            discard_stale_worktrees=True,
        )

    assert removed == []


def test_intake_proposal_generation_does_not_mutate_roadmap_source() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        source = root / "roadmap.yaml"
        original = yaml.safe_dump(valid_state(), sort_keys=False)
        source.write_text(original, encoding="utf-8")
        roadmap = load_roadmap(source)
        proposal = build_intake_proposal(roadmap, idea="Add deterministic terrain brush workflow", owner="tools/workflow")

        write_intake_proposal(proposal, root / "intake")

        assert source.read_text(encoding="utf-8") == original
        loaded = load_intake_proposal(root / "intake" / "proposal.yaml")
        assert loaded.item.id == "WR-003"
        assert loaded.item.planning_state == "blocked_deferred"
        assert (root / "intake" / "proposal.md").exists()


def test_apply_intake_inserts_new_roadmap_item() -> None:
    state = valid_state()
    roadmap = RoadmapState.model_validate(state)
    proposal = build_intake_proposal(roadmap, idea="Add deterministic terrain brush workflow")

    updated = roadmap_data_with_proposal(state, proposal)
    updated_roadmap = RoadmapState.model_validate(updated)

    assert [item.id for item in updated_roadmap.items][-1] == "WR-003"
    assert updated_roadmap.items[-1].planning_state == "blocked_deferred"


def test_apply_intake_rejects_missing_dependencies() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        state = valid_state()
        source = root / "roadmap.yaml"
        source.write_text(yaml.safe_dump(state, sort_keys=False), encoding="utf-8")
        proposal = build_intake_proposal(RoadmapState.model_validate(state), idea="Add feature")
        broken_item = proposal.item.model_copy(update={"dependencies": ["WR-999"]})
        broken = proposal.model_copy(update={"item": broken_item})
        proposal_path = root / "proposal.yaml"
        proposal_path.write_text(yaml.safe_dump(proposal_to_yaml_data(broken), sort_keys=False), encoding="utf-8")

        with pytest.raises(WorkflowError, match="unknown dependency WR-999"):
            apply_intake_proposal(proposal_path, source=source, skip_checks=True)


def test_apply_intake_rejects_invalid_write_scopes() -> None:
    proposal = build_intake_proposal(
        RoadmapState.model_validate(valid_state()),
        idea="Add feature",
        owner="missing/path",
    )

    assert validate_intake_item_scopes(proposal.item) == [
        "write-scope path missing: WR-003:missing/path does not exist"
    ]


def test_apply_intake_rejects_stale_score_math() -> None:
    data = proposal_to_yaml_data(build_intake_proposal(RoadmapState.model_validate(valid_state()), idea="Add feature"))
    data["item"]["expected_score"] = 9.9

    with tempfile.TemporaryDirectory() as temp_dir:
        proposal_path = Path(temp_dir) / "proposal.yaml"
        proposal_path.write_text(yaml.safe_dump(data, sort_keys=False), encoding="utf-8")

        with pytest.raises(WorkflowError, match="expected_score"):
            load_intake_proposal(proposal_path)


def test_promote_rejects_current_candidate_when_dependency_is_not_context() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "ready_next"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Ready next"
    state["items"][1]["planning_state"] = "ready_next"

    with pytest.raises(WorkflowError, match="dependencies are not completed/support context"):
        roadmap_data_with_promotion(
            state,
            item_id="WR-002",
            state="current_candidate",
            evidence="Ready after review.",
        )


def test_promote_rejects_current_candidate_above_b2_gate() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][1]["planning_state"] = "ready_next"
    state["items"][1]["blocker"] = 3
    state["items"][1]["gate"] = "Ready next"

    with pytest.raises(WorkflowError, match="above the B2 implementation gate"):
        roadmap_data_with_promotion(
            state,
            item_id="WR-002",
            state="current_candidate",
            evidence="Ready after review.",
        )


def test_promote_rejects_current_candidate_with_unmet_decision_gate() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][1]["planning_state"] = "ready_next"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Ready next"
    state["items"][1]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]

    with pytest.raises(WorkflowError, match="does not match required"):
        roadmap_data_with_promotion(
            state,
            item_id="WR-002",
            state="current_candidate",
            evidence="Ready after review.",
        )


def test_promotion_preflight_reports_needs_switch_for_overlapping_current_candidate() -> None:
    roadmap = RoadmapState.model_validate(valid_state_with_switch_target())

    result = promotion_preflight(roadmap, "WR-003", "current_candidate", evidence="Ready after review.")

    assert result.status == "needs_switch"
    assert result.blocking_current_candidates == ("WR-001",)
    assert "write-scope conflict" in result.reasons[0]
    assert result.suggested_command == (
        'task roadmap:switch-current -- --from WR-001 --to WR-003 --evidence "Ready after review."'
    )


def test_promotion_preflight_reports_metadata_blockers() -> None:
    b3_state = valid_state()
    b3_state["items"][0]["planning_state"] = "completed"
    b3_state["items"][1]["planning_state"] = "ready_next"
    b3_state["items"][1]["blocker"] = 3
    b3_state["items"][1]["gate"] = "Ready next"
    b3 = promotion_preflight(RoadmapState.model_validate(b3_state), "WR-002", "current_candidate")

    dependency_state = valid_state()
    dependency_state["items"][0]["planning_state"] = "ready_next"
    dependency_state["items"][1]["planning_state"] = "ready_next"
    dependency_state["items"][1]["blocker"] = 2
    dependency_state["items"][1]["gate"] = "Ready next"
    dependency = promotion_preflight(RoadmapState.model_validate(dependency_state), "WR-002", "current_candidate")

    gate_state = valid_state()
    gate_state["items"][0]["planning_state"] = "completed"
    gate_state["items"][1]["planning_state"] = "ready_next"
    gate_state["items"][1]["blocker"] = 2
    gate_state["items"][1]["gate"] = "Ready next"
    gate_state["items"][1]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]
    gate = promotion_preflight(RoadmapState.model_validate(gate_state), "WR-002", "current_candidate")

    assert b3.status == "metadata_blocked"
    assert "B3 is above the B2 implementation gate" in b3.reasons[0]
    assert dependency.status == "metadata_blocked"
    assert "dependencies are not completed/support context" in dependency.reasons[0]
    assert gate.status == "metadata_blocked"
    assert "does not match required" in gate.reasons[0]


def test_switch_current_candidate_updates_two_items_atomically(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source = tmp_path / "roadmap.yaml"
    write_yaml(source, valid_state_with_switch_target())
    monkeypatch.setattr("roadmap_intake.render_and_check", lambda _roadmap, *, skip_checks=False: None)

    switch_current_candidate(
        from_id="WR-001",
        to_id="WR-003",
        evidence="Switch to Workbench handles.",
        source=source,
    )
    roadmap = RoadmapState.model_validate(load_yaml(source))

    assert roadmap.by_id["WR-001"].planning_state == "ready_next"
    assert roadmap.by_id["WR-003"].planning_state == "current_candidate"
    assert roadmap.by_id["WR-003"].current_decision == "Switch to Workbench handles."


def test_switch_current_candidate_writes_nothing_when_validation_fails(tmp_path: Path) -> None:
    source = tmp_path / "roadmap.yaml"
    state = valid_state_with_switch_target()
    state["items"][2]["blocker"] = 3
    write_yaml(source, state)
    before = source.read_text(encoding="utf-8")

    with pytest.raises(WorkflowError, match="above the B2 implementation gate"):
        switch_current_candidate(
            from_id="WR-001",
            to_id="WR-003",
            evidence="Switch to Workbench handles.",
            source=source,
        )

    assert source.read_text(encoding="utf-8") == before


def test_promote_updates_existing_item_with_evidence() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][1]["planning_state"] = "ready_next"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Ready next"

    updated = roadmap_data_with_promotion(
        state,
        item_id="WR-002",
        state="current_candidate",
        evidence="Ready after review.",
    )
    roadmap = RoadmapState.model_validate(updated)

    assert roadmap.by_id["WR-002"].planning_state == "current_candidate"
    assert roadmap.by_id["WR-002"].current_decision == "Ready after review."
