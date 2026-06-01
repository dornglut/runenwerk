from __future__ import annotations

import copy
import tempfile
import subprocess
from collections.abc import Callable
from hashlib import sha256
from pathlib import Path

import pytest
import yaml
from typer.testing import CliRunner

import track_execution_manifest as track_manifest_module
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
    app as production_state_app,
    load_production_tracks,
    validate_completion_quality as validate_production_completion_quality,
    validate_design_gates,
    validate_manifest_backed_tracks,
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
    render_stack_goal,
    render_track_goal,
)
from track_execution_manifest import (
    apply_auto_safe_track_expansion,
    assert_auto_safe_expansion_allowed,
    audit_manifest,
    build_track_execution_lock_data,
    first_current_manifest_entry,
    implementation_plan_consistency_errors_from_text,
    load_track_execution_lock,
    load_track_execution_manifest,
    new_file_scope_errors,
    product_plan_contract_errors,
    resolve_manifest_command_context,
)
from track_execution_manifest import app as track_manifest_app
from execution.cli import app as execution_app
from execution.compiler import compile_contract_pack, load_contract_pack, write_contract_pack
from execution.contracts import ActionContract, CloseoutContract, ContractPack, EvidenceRequirement, RollbackPolicy
from execution.locks import build_execution_lock, execution_lock_errors, write_execution_lock
from execution.preflight import preflight_pack
from execution.evidence import passed_record, write_evidence_record
from execution.runner import run_action, run_next_action
from execution.writers import AgentResult, run_writer
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
    normalize_repo_path,
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
            "full_track_roadmap": "production-roadmap-full.puml",
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


def valid_production_stack_state() -> dict:
    state = valid_production_state()
    state["tracks"] = [
        {
            "id": "PT-BASE",
            "title": "Base production track",
            "state": "active",
            "owner": "workspace",
            "strategic_goal": "Complete the prerequisite renderer foundation.",
            "success_criteria": ["The base track validates."],
            "milestones": [
                production_milestone("PM-BASE-001", roadmap_links=["WR-001"]),
                production_milestone(
                    "PM-BASE-002",
                    kind="hardening",
                    state="designing",
                    dependencies=["PM-BASE-001"],
                    roadmap_links=["WR-002"],
                ),
            ],
        },
        {
            "id": "PT-END",
            "title": "Final production audit",
            "state": "active",
            "owner": "workspace",
            "strategic_goal": "Finish only after prerequisites complete.",
            "success_criteria": ["The final track validates."],
            "milestones": [
                production_milestone(
                    "PM-END-001",
                    kind="release",
                    state="designing",
                    dependencies=["PM-BASE-002"],
                )
            ],
        },
    ]
    return state


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


def valid_track_manifest_state() -> dict:
    return {
        "version": 1,
        "track_id": "PT-TEST",
        "authority_level": "planning_and_sequencing_only",
        "accepted_design_dependencies": [
            {
                "path": "docs-site/src/content/docs/workspace/track-execution-manifest.md",
                "required_status": "active",
                "reason": "Test manifest dependency.",
            }
        ],
        "global_forbidden_scope": ["no product code from this manifest alone"],
        "global_validation_commands": ["task planning:validate"],
        "global_stop_conditions": ["stop after one legal action"],
        "next_legal_action": "Execute PM-TEST-001.",
        "ai_executable": False,
        "truth_claims": [
            {
                "claim_id": "test-track-proof",
                "claim_kind": "product_behavior",
                "claim_level": "runtime_proven",
                "claim_status": "satisfied",
                "claim_statement": "The test manifest has enough evidence for workflow validation.",
                "required_docs": [
                    {
                        "evidence_kind": "doc_exists",
                        "path": "tools/workflow/test_workflow.py",
                        "reason": "Fixture truth evidence path exists.",
                    }
                ],
                "required_code_contracts": [],
                "required_validations": [],
                "required_closeout_evidence": [],
                "known_gaps": [],
                "supersedes": [],
                "blocks_downstream": [],
            }
        ],
        "milestones": [
            manifest_milestone("PM-TEST-001", owning_wr="WR-001"),
            manifest_milestone(
                "PM-TEST-002",
                milestone_type="docs_only",
                owning_wr="WR-002",
                predecessor_dependencies=["PM-TEST-001"],
                write_scope=["docs-site"],
                may_create_code=False,
                next_legal_action="Wait for PM-TEST-001.",
            ),
        ],
    }


def valid_track_expansion_state() -> tuple[dict, dict, dict]:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["state"] = "completed"
    production_data["tracks"][0]["milestones"][0]["completion_quality"] = "bounded_contract"
    production_data["tracks"][0]["milestones"][1]["state"] = "designing"
    production_data["tracks"][0]["milestones"][1]["roadmap_links"] = []

    manifest_data = valid_track_manifest_state()
    second_milestone = manifest_data["milestones"][1]
    second_milestone.pop("owning_wr")
    second_milestone["future_wr_candidate"] = "WR-TBD-TEST-002"
    second_milestone["milestone_type"] = "design_only"
    second_milestone["write_scope"] = [
        "active design docs",
        "implementation-plan report",
        "roadmap and production metadata",
        "generated planning docs",
        "closeout report",
    ]
    second_milestone["may_create_code"] = False
    second_milestone["may_create_crates"] = False
    second_milestone["may_modify_production_behavior"] = False
    second_milestone["next_legal_action"] = "Create or link the design WR for PM-TEST-002."
    add_test_auto_safe_contract(second_milestone)
    manifest_data["next_legal_action"] = "After PM-TEST-001, create or link the design WR for PM-TEST-002."

    roadmap_data = valid_state()
    return production_data, roadmap_data, manifest_data


def valid_agent_design_state(
    *,
    production_path: Path,
    plan_path: Path,
    design_path: Path,
    manifest_path: Path,
    manifest_report: Path,
    archive_path: Path,
    deferred_path: Path,
    closeout_path: Path,
) -> tuple[dict, dict, dict, dict]:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["state"] = "completed"
    production_data["tracks"][0]["milestones"][0]["completion_quality"] = "bounded_contract"
    production_data["tracks"][0]["milestones"][1]["state"] = "designing"
    production_data["tracks"][0]["milestones"].append(
        production_milestone(
            "PM-TEST-003",
            kind="design",
            state="designing",
            dependencies=["PM-TEST-002"],
            roadmap_links=[],
        )
    )

    active_roadmap = copy.deepcopy(valid_state())
    deferred_item = active_roadmap["items"].pop(1)
    deferred_item["planning_state"] = "blocked_deferred"
    deferred_item["blocker"] = 5
    deferred_item["write_scopes"] = [
        repo_path(production_path),
        repo_path(plan_path),
        repo_path(design_path),
        repo_path(manifest_path),
        repo_path(manifest_report),
        repo_path(archive_path),
        repo_path(deferred_path),
        repo_path(closeout_path),
    ]
    deferred_item["validations"] = ["task docs:validate", "task planning:validate"]
    deferred_roadmap = {
        "version": active_roadmap["version"],
        "roadmap": active_roadmap["roadmap"],
        "items": [deferred_item],
    }

    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["milestone_type"] = "implementation"
    manifest_data["milestones"][0]["may_create_code"] = True
    manifest_data["milestones"][0]["may_modify_production_behavior"] = True
    manifest_data["milestones"][1]["milestone_type"] = "design_only"
    manifest_data["milestones"][1]["write_scope"] = [
        repo_path(production_path),
        repo_path(plan_path),
        repo_path(design_path),
        repo_path(manifest_path),
        repo_path(manifest_report),
        repo_path(archive_path),
        repo_path(deferred_path),
        repo_path(closeout_path),
    ]
    manifest_data["milestones"][1]["expected_closeout_path"] = repo_path(closeout_path)
    manifest_data["milestones"][1]["may_create_code"] = False
    manifest_data["milestones"][1]["may_create_crates"] = False
    manifest_data["milestones"][1]["may_modify_production_behavior"] = False
    manifest_data["milestones"][1]["next_legal_action"] = "Run agent_design for PM-TEST-002."
    manifest_data["milestones"][1]["agent_design"] = {
        "source_documents": ["docs-site/src/content/docs/workspace/track-execution-manifest.md"],
        "required_sections": ["UiProgram graph ownership", "UiSchemaValue contract"],
        "required_decisions": ["UiProgram remains UI-owned."],
        "acceptance_checklist": ["Design plan and architecture section exist."],
    }
    manifest_data["milestones"].append(
        manifest_milestone(
            "PM-TEST-003",
            milestone_type="design_only",
            future_wr_candidate="WR-TBD-TEST-003",
            predecessor_dependencies=["PM-TEST-002"],
            write_scope=["active design docs", "implementation-plan report", "closeout report"],
            may_create_code=False,
            next_legal_action="After PM-TEST-002, create or link the design WR for PM-TEST-003.",
        )
    )
    manifest_data["next_legal_action"] = "Run agent_design for PM-TEST-002."
    return production_data, active_roadmap, deferred_roadmap, manifest_data


def write_agent_design_fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    *,
    mutate: Callable[[dict, dict, dict, dict], None] | None = None,
) -> tuple[Path, Path, Path, Path, Path]:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    archive_path = tmp_path / "roadmap-archive.yaml"
    deferred_path = tmp_path / "roadmap-deferred.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_path = manifest_root / "pt-test.yaml"
    manifest_report = REPO_ROOT / "docs-site/src/content/docs/reports/track-execution-manifests/pt-test/manifest.md"
    plan_path = tmp_path / "plans" / "wr-002-ui-program-contract-design" / "plan.md"
    closeout_path = tmp_path / "closeouts" / "pm-test-002" / "closeout.md"
    design_path = tmp_path / "ui-program-architecture.md"
    design_path.write_text(
        "---\ntitle: UI Program Architecture\nstatus: active\n---\n\n# UI Program Architecture\n\n## 13. Staged Implementation Plan\n\nExisting plan.\n",
        encoding="utf-8",
    )
    production_data, active_roadmap, deferred_roadmap, manifest_data = valid_agent_design_state(
        production_path=production_path,
        plan_path=plan_path,
        design_path=design_path,
        manifest_path=manifest_path,
        manifest_report=manifest_report,
        archive_path=archive_path,
        deferred_path=deferred_path,
        closeout_path=closeout_path,
    )
    if mutate is not None:
        mutate(production_data, active_roadmap, deferred_roadmap, manifest_data)
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, active_roadmap)
    write_yaml(deferred_path, deferred_roadmap)
    write_yaml(manifest_path, manifest_data)
    monkeypatch.setattr("track_execution_manifest.default_contract_path", lambda item: plan_path)
    monkeypatch.setattr("track_execution_manifest.implementation_plan_path", lambda wr_id, milestone: repo_path(plan_path))
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )
    return production_path, roadmap_path, manifest_root, plan_path, design_path


def valid_product_code_state(
    *,
    plan_path: Path,
    closeout_path: Path,
    implementation_path: Path,
    test_path: Path,
) -> tuple[dict, dict, dict]:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["state"] = "completed"
    production_data["tracks"][0]["milestones"][0]["completion_quality"] = "bounded_contract"
    production_data["tracks"][0]["milestones"][1]["kind"] = "implementation"
    production_data["tracks"][0]["milestones"][1]["state"] = "active"
    production_data["tracks"][0]["milestones"][1]["roadmap_links"] = ["WR-002"]

    roadmap_data = valid_state()
    roadmap_data["items"][1]["planning_state"] = "current_candidate"
    roadmap_data["items"][1]["blocker"] = 2
    roadmap_data["items"][1]["gate"] = "Implementation-ready"
    roadmap_data["items"][1]["write_scopes"] = [
        repo_path(implementation_path),
        repo_path(test_path),
        repo_path(closeout_path),
    ]
    roadmap_data["items"][1]["validations"] = ["task docs:validate"]

    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["milestone_type"] = "implementation"
    manifest_data["milestones"][0]["may_create_code"] = True
    manifest_data["milestones"][0]["may_modify_production_behavior"] = True
    manifest_data["milestones"][1]["milestone_type"] = "implementation"
    manifest_data["milestones"][1]["execution_kind"] = "implementation_proof"
    manifest_data["milestones"][1]["closeout_strategy"] = "runtime_proven_closeout"
    manifest_data["milestones"][1]["write_scope"] = [
        repo_path(implementation_path),
        repo_path(test_path),
        repo_path(closeout_path),
    ]
    manifest_data["milestones"][1]["validation_commands"] = ["task docs:validate"]
    manifest_data["milestones"][1]["expected_closeout_path"] = repo_path(closeout_path)
    manifest_data["milestones"][1]["may_create_code"] = True
    manifest_data["milestones"][1]["may_create_crates"] = False
    manifest_data["milestones"][1]["may_modify_production_behavior"] = True
    manifest_data["milestones"][1]["next_legal_action"] = "Run product_code for PM-TEST-002."
    add_test_implementation_contracts(manifest_data["milestones"][1], exact_scope=repo_path(implementation_path))
    manifest_data["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [
        repo_path(implementation_path),
        repo_path(test_path),
        repo_path(closeout_path),
    ]
    manifest_data["milestones"][1]["runtime_closeout_contract"]["files_changed_report"] = [
        repo_path(implementation_path),
        repo_path(test_path),
        repo_path(closeout_path),
    ]
    manifest_data["next_legal_action"] = "Run product_code for PM-TEST-002."
    return production_data, roadmap_data, manifest_data


def product_code_plan_text(implementation_path: Path, test_path: Path, closeout_path: Path) -> str:
    implementation_scope = repo_path(implementation_path)
    test_scope = repo_path(test_path)
    closeout_scope = repo_path(closeout_path)
    return f"""---
title: WR-002 Product Code Contract
description: Synthetic active implementation contract.
status: active
owner: test
layer: test
canonical: false
---

# WR-002 PM-TEST-002 title Product Code Contract

- Production milestone: `PM-TEST-002` - PM-TEST-002 title
- Roadmap item: `WR-002` - WR-002 title
- Proof slice id: `PM-TEST-002`
- Proof slice title: PM-TEST-002 title
- Target control/surface: PM-TEST-002 title
- Implementation proof kind: `PM-TEST-002`

## Implementation Scope

Expected implementation files:

- `{implementation_scope}`
- `{test_scope}`
- `{closeout_scope}`

Expected methods/functions:

- `run_product_code_fixture`

## Forbidden Scope

- Files/modules forbidden: anything outside the exact files/modules above.
- No crate creation.
- No foundation/meta extraction.

## Tests To Add Or Change

- Update focused tests in `{test_scope}`.

## Validation

- `task docs:validate`
- `uv run pytest tools/workflow/test_workflow.py`

## Closeout Requirements

- Closeout evidence: `{closeout_scope}` with runtime/test evidence.

## Compatibility / Rollback Plan

- Compatibility is preserved by limiting changes to the exact fixture files.
- Rollback is the direct revert of those exact files.

## Stop Conditions

- Stop after one implementation WR.
"""


def write_product_code_fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    *,
    write_plan: bool = True,
    mutate: Callable[[dict, dict, dict], None] | None = None,
) -> tuple[Path, Path, Path, Path, Path, Path]:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    plan_path = tmp_path / "plans" / "wr-002-product-code" / "plan.md"
    implementation_path = tmp_path / "product" / "src" / "lib.rs"
    test_path = tmp_path / "product" / "tests" / "product_code.rs"
    closeout_path = tmp_path / "closeouts" / "pm-test-002" / "closeout.md"
    implementation_path.parent.mkdir(parents=True)
    test_path.parent.mkdir(parents=True)
    implementation_path.write_text("// implementation fixture\n", encoding="utf-8")
    test_path.write_text("// test fixture\n", encoding="utf-8")
    if write_plan:
        plan_path.parent.mkdir(parents=True)
        plan_path.write_text(product_code_plan_text(implementation_path, test_path, closeout_path), encoding="utf-8")
    production_data, roadmap_data, manifest_data = valid_product_code_state(
        plan_path=plan_path,
        closeout_path=closeout_path,
        implementation_path=implementation_path,
        test_path=test_path,
    )
    if mutate is not None:
        mutate(production_data, roadmap_data, manifest_data)
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    monkeypatch.setattr("track_execution_manifest.default_contract_path", lambda item: plan_path)
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )
    return production_path, roadmap_path, manifest_root, plan_path, implementation_path, closeout_path


def make_runtime_closeout_ready(
    production_path: Path,
    roadmap_path: Path,
    manifest_root: Path,
    closeout_path: Path,
    *,
    track_id: str = "PT-TEST",
    wr_id: str = "WR-002",
    validation_commands: list[str] | None = None,
) -> None:
    commands = validation_commands or ["cargo test -p test-runtime-proof"]
    manifest_path = manifest_root / f"{track_id.lower()}.yaml"
    archive_path = roadmap_path.with_name("roadmap-archive.yaml")
    deferred_path = roadmap_path.with_name("roadmap-deferred.yaml")
    manifest_report = REPO_ROOT / f"docs-site/src/content/docs/reports/track-execution-manifests/{track_id.lower()}/manifest.md"

    production = load_yaml(production_path)
    roadmap = load_yaml(roadmap_path)
    manifest = load_yaml(manifest_path)

    for item_data in roadmap["items"]:
        if item_data["id"] == wr_id:
            item_data["validations"] = commands
            for scope in [
                repo_path(production_path),
                repo_path(roadmap_path),
                repo_path(archive_path),
                repo_path(deferred_path),
                repo_path(manifest_path),
                repo_path(manifest_report),
                repo_path(closeout_path),
                "generated: production docs from task production:render",
                "generated: roadmap docs from task roadmap:render",
            ]:
                if scope not in item_data["write_scopes"]:
                    item_data["write_scopes"].append(scope)
            break

    manifest["milestones"][1]["validation_commands"] = commands
    manifest["milestones"][1]["product_code_contract"]["validation_commands"] = commands
    manifest["milestones"][1]["runtime_closeout_contract"]["validation_commands"] = commands
    manifest["milestones"][1]["implementation_writer"]["validation_commands"] = commands
    manifest["milestones"][1]["expected_closeout_path"] = repo_path(closeout_path)
    manifest["milestones"][1]["next_legal_action"] = (
        "PM-TEST-002 product_implementation completed; runtime closeout is the next legal action."
    )
    manifest["next_legal_action"] = manifest["milestones"][1]["next_legal_action"]

    write_yaml(production_path, production)
    write_yaml(roadmap_path, roadmap)
    write_yaml(manifest_path, manifest)


def write_implementation_agent_design_fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    *,
    track_id: str = "PT-TEST",
    milestone_id: str = "PM-TEST-002",
    mutate: Callable[[dict, dict, dict], None] | None = None,
) -> tuple[Path, Path, Path, Path, list[Path]]:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    deferred_path = tmp_path / "roadmap-deferred.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_path = manifest_root / f"{track_id.lower()}.yaml"
    plan_path = tmp_path / "plans" / "wr-003-6a-label-structural-uiframe-text-proof" / "plan.md"
    closeout_path = tmp_path / "closeouts" / milestone_id.lower() / "closeout.md"
    scope_paths = [
        tmp_path / "domain" / "ui" / "ui_widgets" / "src" / "label.rs",
        tmp_path / "domain" / "ui" / "ui_text" / "src" / "layout.rs",
        tmp_path / "domain" / "ui" / "ui_render_data" / "src" / "lib.rs",
        tmp_path / "domain" / "ui" / "ui_runtime" / "src" / "output" / "build_ui_frame.rs",
        tmp_path / "domain" / "ui" / "ui_definition" / "src" / "source.rs",
    ]
    for path in scope_paths:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// scoped fixture\n", encoding="utf-8")

    production_data = valid_production_state()
    production_data["tracks"][0]["id"] = track_id
    production_data["tracks"][0]["milestones"][0]["state"] = "completed"
    production_data["tracks"][0]["milestones"][0]["completion_quality"] = "bounded_contract"
    production_data["tracks"][0]["milestones"][1]["id"] = milestone_id
    production_data["tracks"][0]["milestones"][1]["title"] = "6A Label Structural UiFrame Text Proof"
    production_data["tracks"][0]["milestones"][1]["kind"] = "implementation"
    production_data["tracks"][0]["milestones"][1]["state"] = "designing"
    production_data["tracks"][0]["milestones"][1]["dependencies"] = ["PM-TEST-001"]
    production_data["tracks"][0]["milestones"][1]["roadmap_links"] = []

    active_roadmap = valid_state()
    active_roadmap["items"] = [active_roadmap["items"][0]]
    active_roadmap["items"][0]["planning_state"] = "support_only"
    active_roadmap["edges"] = []
    deferred_roadmap = {
        "version": active_roadmap["version"],
        "roadmap": active_roadmap["roadmap"],
        "items": [],
    }

    manifest_data = valid_track_manifest_state()
    manifest_data["track_id"] = track_id
    manifest_data["milestones"][1]["milestone_id"] = milestone_id
    manifest_data["milestones"][1]["title"] = "6A Label Structural UiFrame Text Proof"
    manifest_data["milestones"][1]["milestone_type"] = "implementation"
    manifest_data["milestones"][1].pop("owning_wr", None)
    manifest_data["milestones"][1]["future_wr_candidate"] = "WR-TBD-TEST-002"
    manifest_data["milestones"][1]["write_scope"] = [repo_path(path) for path in scope_paths]
    manifest_data["milestones"][1]["forbidden_scope"] = [
        "no new crates",
        "no placeholder future folders",
        "no broad retained UI rewrite",
        "no Button implementation",
        "no InspectorField implementation",
        "no ColorPicker implementation",
        "no 6B through 6F",
        "no MaterialProgram implementation",
        "no foundation/meta extraction",
    ]
    manifest_data["milestones"][1]["validation_commands"] = ["task docs:validate"]
    manifest_data["milestones"][1]["expected_closeout_path"] = repo_path(closeout_path)
    manifest_data["milestones"][1]["may_create_code"] = True
    manifest_data["milestones"][1]["may_create_crates"] = False
    manifest_data["milestones"][1]["may_modify_production_behavior"] = True
    manifest_data["milestones"][1]["next_legal_action"] = f"Create or link the WR for {milestone_id}."
    manifest_data["milestones"][1]["agent_design"] = {
        "source_documents": ["docs-site/src/content/docs/workspace/track-execution-manifest.md"],
        "required_sections": ["Label text output", "structural UiFrame assertion"],
        "required_decisions": ["6A is limited to Label plus structural UiFrame text proof only."],
        "acceptance_checklist": ["The implementation plan names every exact write_scope path."],
    }
    add_test_auto_safe_contract(manifest_data["milestones"][1])
    add_test_implementation_contracts(
        manifest_data["milestones"][1],
        exact_scope=repo_path(scope_paths[0]),
    )
    manifest_data["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [
        repo_path(path) for path in scope_paths
    ]
    manifest_data["milestones"][1]["runtime_closeout_contract"]["files_changed_report"] = [
        repo_path(path) for path in scope_paths
    ]
    manifest_data["next_legal_action"] = f"Create or link the WR for {milestone_id}."

    if mutate is not None:
        mutate(production_data, active_roadmap, manifest_data)

    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, active_roadmap)
    write_yaml(deferred_path, deferred_roadmap)
    write_yaml(manifest_path, manifest_data)
    monkeypatch.setattr("track_execution_manifest.default_contract_path", lambda item: plan_path)
    monkeypatch.setattr("track_execution_manifest.implementation_plan_path", lambda wr_id, milestone: repo_path(plan_path))
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )
    return production_path, roadmap_path, manifest_root, plan_path, scope_paths


def contract_marker() -> str:
    return "generated_by_production_complete_track_contracts"


def add_test_auto_safe_contract(entry: dict) -> None:
    entry["auto_safe_contract"] = {
        "wr_candidate_policy": "allocate test WR",
        "wr_id_allocation_behavior": "next numeric WR",
        "milestone_to_wr_link_behavior": "link exactly this milestone",
        "manifest_wr_reference_behavior": "replace future WR with owning WR",
        "allowed_metadata_write_scopes": ["tools/workflow"],
        "forbidden_scopes": ["product code"],
        "validation_commands": ["task docs:validate"],
        "stop_conditions": ["stop before implementation"],
        "template_key": "implementation_runtime_proof" if entry["milestone_type"] in {"implementation", "hardening"} else "docs_design",
        "generated_contract_marker": contract_marker(),
        "generated_from_template_version": "v1",
    }


def add_test_implementation_contracts(entry: dict, *, exact_scope: str = "tools/workflow/test_workflow.py") -> None:
    entry.pop("agent_closeout_contract", None)
    entry["agent_design_contract"] = {
        "source_documents": ["docs-site/src/content/docs/workspace/track-execution-manifest.md"],
        "required_sections": ["Test implementation plan"],
        "required_decisions": ["Test implementation remains bounded."],
        "acceptance_checklist": ["Exact write scope is listed."],
        "planning_write_scope": ["tools/workflow"],
        "allowed_write_scopes": ["tools/workflow"],
        "forbidden_scopes": ["crate creation", "foundation/meta extraction"],
        "expected_output_paths": ["tools/workflow/test-plan.md"],
        "validation_commands": ["task docs:validate"],
        "stop_conditions": ["stop before product code"],
        "template_key": "implementation_runtime_proof",
        "generated_contract_marker": contract_marker(),
        "generated_from_template_version": "v1",
    }
    entry["product_code_contract"] = {
        "required_active_wr_state": "current_candidate B2 or lower",
        "required_accepted_implementation_plan": "active accepted test plan",
        "exact_allowed_implementation_write_scopes": [exact_scope],
        "required_function_method_scope": ["test function"],
        "forbidden_implementation_scopes": ["crate creation", "foundation/meta extraction"],
        "tests_to_add_change": ["focused workflow test"],
        "runtime_evidence_required": ["workflow test evidence"],
        "validation_commands": ["uv run pytest tools/workflow/test_workflow.py"],
        "rollback_compatibility_expectations": ["revert exact test scope"],
        "closeout_evidence": [entry["expected_closeout_path"]],
        "stop_conditions": ["stop after one implementation WR"],
        "template_key": "implementation_runtime_proof",
        "generated_contract_marker": contract_marker(),
        "generated_from_template_version": "v1",
    }
    entry["runtime_closeout_contract"] = {
        "runtime_test_evidence_required": ["workflow test evidence"],
        "validation_commands": ["uv run pytest tools/workflow/test_workflow.py"],
        "completion_quality_allowed": ["runtime_proven"],
        "closeout_path": entry["expected_closeout_path"],
        "files_changed_report": [exact_scope],
        "known_gap_reporting": ["report test gaps"],
        "production_roadmap_state_updates": ["complete test milestone"],
        "next_action_update_rules": ["advance next action"],
        "template_key": "implementation_runtime_proof",
        "generated_contract_marker": contract_marker(),
        "generated_from_template_version": "v1",
    }
    entry["implementation_writer"] = {
        "strategy": "template_writer",
        "allowed_files": [exact_scope],
        "required_outputs": ["bounded workflow test implementation evidence"],
        "forbidden_files": ["foundation/meta"],
        "forbidden_patterns": ["foundation/meta"],
        "new_file_policy": "explicit_new_scope_required",
        "validation_commands": ["uv run pytest tools/workflow/test_workflow.py"],
        "stop_conditions": ["stop after one implementation WR"],
        "templates": [
            {
                "file": exact_scope,
                "content": "// changed by product implementation\n",
            }
        ],
    }


def manifest_milestone(
    milestone_id: str,
    *,
    milestone_type: str = "implementation",
    owning_wr: str | None = None,
    future_wr_candidate: str | None = None,
    predecessor_dependencies: list[str] | None = None,
    write_scope: list[str] | None = None,
    may_create_code: bool = True,
    next_legal_action: str = "Execute the bounded milestone action.",
) -> dict:
    marker = contract_marker()
    version = "v1"
    template_key = "implementation_runtime_proof" if milestone_type in {"implementation", "hardening"} else "docs_design"
    execution_kind_by_type = {
        "docs_only": "design_contract",
        "design_only": "design_contract",
        "implementation": "implementation_proof",
        "hardening": "implementation_proof",
        "closeout": "handoff_closeout",
    }
    closeout_strategy_by_execution_kind = {
        "design_contract": "bounded_contract_closeout",
        "implementation_proof": "runtime_proven_closeout",
        "proof_aggregation": "runtime_proven_closeout",
        "handoff_closeout": "handoff_closeout",
        "extraction_gate": "extraction_gate_closeout",
    }
    execution_kind = execution_kind_by_type[milestone_type]
    entry = {
        "milestone_id": milestone_id,
        "title": f"{milestone_id} title",
        "stage": "Stage test",
        "authority_level": "test_authority",
        "milestone_type": milestone_type,
        "execution_kind": execution_kind,
        "closeout_strategy": closeout_strategy_by_execution_kind[execution_kind],
        "predecessor_dependencies": predecessor_dependencies or [],
        "write_scope": write_scope or ["tools/workflow"],
        "forbidden_scope": ["adjacent work"],
        "required_contracts": ["test contract"],
        "validation_commands": ["cargo test -p test"],
        "evidence_gates": ["test closeout"],
        "expected_closeout_path": f"docs-site/src/content/docs/reports/closeouts/{milestone_id.lower()}/closeout.md",
        "stop_conditions": ["stop after one action"],
        "next_legal_action": next_legal_action,
        "may_create_code": may_create_code,
        "may_create_crates": False,
        "may_modify_production_behavior": may_create_code,
    }
    if owning_wr:
        entry["owning_wr"] = owning_wr
    if future_wr_candidate:
        entry["future_wr_candidate"] = future_wr_candidate
        add_test_auto_safe_contract(entry)
    if milestone_type in {"docs_only", "design_only"}:
        entry["agent_design_contract"] = {
            "source_documents": ["docs-site/src/content/docs/workspace/track-execution-manifest.md"],
            "required_sections": ["Test design plan"],
            "required_decisions": ["Test design remains bounded."],
            "acceptance_checklist": ["Test design evidence exists."],
            "planning_write_scope": ["tools/workflow"],
            "allowed_write_scopes": ["tools/workflow"],
            "forbidden_scopes": ["product code"],
            "expected_output_paths": ["tools/workflow/test_workflow.py"],
            "validation_commands": ["task docs:validate"],
            "stop_conditions": ["stop before closeout"],
            "template_key": "docs_design",
            "generated_contract_marker": marker,
            "generated_from_template_version": version,
        }
        entry["agent_closeout_contract"] = {
            "evidence_files": [entry["expected_closeout_path"]],
            "validation_commands": ["task docs:validate"],
            "completion_quality_allowed": ["bounded_contract"],
            "closeout_path": entry["expected_closeout_path"],
            "production_roadmap_state_updates": ["complete test milestone"],
            "known_gap_reporting": ["report test gaps"],
            "next_action_update_rules": ["advance next action"],
            "template_key": "docs_design",
            "generated_contract_marker": marker,
            "generated_from_template_version": version,
        }
    if milestone_type in {"implementation", "hardening"}:
        add_test_implementation_contracts(entry)
    return entry


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


def test_production_goal_rejects_audit_blocked_manifest(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["required_contracts"] = ["blocked: define contract"]
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
    assert "invalid blocked fields:" in result.stdout
    assert "required_contracts remains blocked" in result.stdout
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


def test_track_manifest_plan_track_creates_scaffold(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "plan-track",
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
    assert (manifest_root / "pt-test.yaml").exists()
    assert "No implementation authority is created" in result.stdout


def test_track_manifest_next_prints_single_action(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert "Current milestone: PM-TEST-001 - PM-TEST-001 title" in result.stdout
    assert "Next legal action: Execute the bounded milestone action." in result.stdout
    assert "Implementation authorized now: no - task production:next is read-only" in result.stdout


def test_track_manifest_next_fails_when_manifest_missing(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert "production:next failed" in result.stdout
    assert "no Track Execution Manifest source" in result.stdout
    assert "Current milestone:" not in result.stdout
    assert "Next legal action:" not in result.stdout


def test_track_manifest_next_rejects_malformed_manifest_yaml(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    (manifest_root / "pt-test.yaml").write_text("version: [\n", encoding="utf-8")

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert "production:next failed" in result.stdout
    assert "while parsing" in result.stdout
    assert "Current milestone:" not in result.stdout
    assert "Next legal action:" not in result.stdout


def test_track_manifest_next_rejects_invalid_manifest_model(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", {"version": 1, "track_id": "PT-TEST"})

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert "production:next failed" in result.stdout
    assert "validation errors for" in result.stdout
    assert "Field required" in result.stdout
    assert "Current milestone:" not in result.stdout
    assert "Next legal action:" not in result.stdout


def test_track_manifest_next_rejects_audit_blocked_manifest(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["required_contracts"] = ["blocked: define contract"]
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert "invalid blocked fields:" in result.stdout
    assert "required_contracts remains blocked" in result.stdout
    assert "Current milestone:" not in result.stdout
    assert "Next legal action:" not in result.stdout


def test_track_manifest_next_rejects_invalid_closeout_path(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["expected_closeout_path"] = "docs-site/src/content/docs/reports/closeouts/pm-test-001/closeout.txt"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert "invalid closeout path:" in result.stdout
    assert "expected_closeout_path must point at a Markdown closeout/report" in result.stdout
    assert "Current milestone:" not in result.stdout
    assert "Next legal action:" not in result.stdout


def test_track_manifest_audit_passes_valid_manifest(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "audit-track",
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
    assert "manifest audit passed" in result.stdout


def test_track_manifest_audit_fails_when_manifest_missing(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "audit-track",
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
    assert "production:audit-track failed" in result.stdout
    assert "no Track Execution Manifest source" in result.stdout
    assert "manifest audit passed" not in result.stdout


def test_track_manifest_expand_track_is_read_only(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][1].pop("owning_wr")
    manifest_data["milestones"][1]["future_wr_candidate"] = "WR-TBD-TEST-002"
    add_test_auto_safe_contract(manifest_data["milestones"][1])
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][1]["roadmap_links"] = []
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "expand-track",
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
    assert "WR-TBD-TEST-002" in result.stdout
    assert "read-only" in result.stdout
    assert not (tmp_path / "roadmap-deferred.yaml").exists()


def test_manifest_runner_auto_safe_creates_and_links_deferred_wr(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V1 applied one auto_safe Track Expansion action." in result.stdout
    assert "Created/linked WR: WR-003" in result.stdout
    updated_production = load_yaml(production_path)
    updated_manifest = load_yaml(manifest_root / "pt-test.yaml")
    updated_deferred = load_yaml(tmp_path / "roadmap-deferred.yaml")
    second_production_milestone = updated_production["tracks"][0]["milestones"][1]
    second_manifest_milestone = updated_manifest["milestones"][1]
    deferred_wr = updated_deferred["items"][0]
    assert second_production_milestone["roadmap_links"] == ["WR-003"]
    assert second_manifest_milestone["owning_wr"] == "WR-003"
    assert "future_wr_candidate" not in second_manifest_milestone
    assert deferred_wr["id"] == "WR-003"
    assert deferred_wr["planning_state"] == "blocked_deferred"
    assert deferred_wr["completion_quality"] == "not_applicable"
    assert "docs-site/src/content/docs/reports/implementation-plans/wr-003-pm-test-002-title/plan.md" in deferred_wr["write_scopes"]
    assert "docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md" in deferred_wr["write_scopes"]
    assert "No implementation" not in result.stdout


def test_manifest_runner_crate_creation_permission_alone_does_not_expand_track(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "crate_creation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "Track Expansion must create or link" in result.stdout
    assert "--allow auto_safe" in result.stdout
    assert not (tmp_path / "roadmap-deferred.yaml").exists()


def test_manifest_runner_auto_safe_expands_implementation_milestone_without_code_authority(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    production_data["tracks"][0]["milestones"][1]["kind"] = "implementation"
    manifest_data["milestones"][1]["milestone_type"] = "implementation"
    manifest_data["milestones"][1]["write_scope"] = ["tools/workflow/test_workflow.py"]
    manifest_data["milestones"][1]["may_create_code"] = True
    add_test_auto_safe_contract(manifest_data["milestones"][1])
    add_test_implementation_contracts(manifest_data["milestones"][1])
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V1 applied one auto_safe Track Expansion action." in result.stdout
    assert "Created/linked WR: WR-003" in result.stdout
    assert "product_code" not in result.stdout


def test_manifest_runner_auto_safe_refuses_completed_current_milestone(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    production_data["tracks"][0]["milestones"][1]["state"] = "completed"
    production_data["tracks"][0]["milestones"][1]["completion_quality"] = "bounded_contract"
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    context = resolve_manifest_command_context(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_source_root=manifest_root,
    )
    with pytest.raises(WorkflowError, match="completed milestones must not be mutated"):
        assert_auto_safe_expansion_allowed(
            context.loaded.manifest.milestones[1],
            context.track.milestones[1],
            roadmap=context.roadmap,
            allow={"auto_safe"},
        )
    assert not (tmp_path / "roadmap-deferred.yaml").exists()


def test_manifest_runner_auto_safe_refuses_existing_wr_link(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    production_data["tracks"][0]["milestones"][1]["roadmap_links"] = ["WR-002"]
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "conflicts with" in result.stdout
    assert "production roadmap_links ['WR-002']" in result.stdout
    assert not (tmp_path / "roadmap-deferred.yaml").exists()


def test_manifest_runner_auto_safe_refuses_wr_collision(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    context = resolve_manifest_command_context(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_source_root=manifest_root,
    )
    monkeypatch.setattr("track_execution_manifest.allocate_next_wr_id", lambda roadmap: "WR-001")

    with pytest.raises(WorkflowError, match="WR-001: already present"):
        apply_auto_safe_track_expansion(
            context,
            production_source=production_path,
            roadmap_source=roadmap_path,
            allow={"auto_safe"},
            run_validations=False,
        )

    assert not (tmp_path / "roadmap-deferred.yaml").exists()


def test_manifest_runner_next_after_expansion_points_to_design_plan_not_implementation(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )

    run_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert run_result.exit_code == 0, run_result.output

    next_result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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

    assert next_result.exit_code == 0, next_result.output
    assert "Current milestone: PM-TEST-002 - PM-TEST-002 title" in next_result.stdout
    assert "task production:plan" in next_result.stdout
    assert "PM-TEST-002" in next_result.stdout
    assert "WR-003" in next_result.stdout
    assert "Workflow action: design_first" in next_result.stdout
    assert "PM-TEST-007" not in next_result.stdout


def test_manifest_runner_agent_design_requires_permission(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "workflow action is design_first" in result.stdout
    assert not plan_path.exists()


def test_manifest_runner_agent_design_does_not_require_product_code_denial(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert "Manifest Runner V4" not in result.stdout
    assert plan_path.exists()


def test_manifest_runner_agent_design_ignores_product_code_permission_until_runtime_action(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert "Manifest Runner V4" not in result.stdout
    assert plan_path.exists()


def test_manifest_runner_agent_design_allows_closeout_permission_without_closing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert plan_path.exists()


def test_manifest_runner_agent_design_rejects_missing_source_documents(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _active: dict, _deferred: dict, manifest: dict) -> None:
        manifest["milestones"][1]["agent_design"]["source_documents"] = ["docs-site/src/content/docs/missing-agent-design-source.md"]

    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "agent_design source documents are missing" in result.stdout
    assert "missing-agent-design-source.md" in result.stdout
    assert not plan_path.exists()


def test_manifest_runner_agent_design_rejects_uncovered_write_scope(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _active: dict, _deferred: dict, manifest: dict) -> None:
        manifest["milestones"][1]["write_scope"] = [
            scope for scope in manifest["milestones"][1]["write_scope"] if not scope.endswith("/plan.md")
        ]

    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "agent_design write paths are not covered by manifest write_scope" in result.stdout
    assert not plan_path.exists()


def test_manifest_runner_agent_design_writes_plan_and_stage_contract(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, design_path = write_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert "Plan path:" in result.stdout
    assert "stop for closeout" in result.stdout
    assert "--allow agent_closeout" in result.stdout
    assert plan_path.exists()
    plan_text = plan_path.read_text(encoding="utf-8")
    assert "## Required Design Sections" in plan_text
    assert "## Forbidden Scope" in plan_text
    assert "## PM-TEST-002 Stage test Contract" in plan_text
    assert "does not authorize product/runtime code" in plan_text
    assert "UiSchemaValue contract" in plan_text
    design_text = design_path.read_text(encoding="utf-8")
    assert "## PM-TEST-002 Stage test Contract" not in design_text
    updated_manifest = load_yaml(manifest_root / "pt-test.yaml")
    assert "agent_design completed design/planning writes" in updated_manifest["next_legal_action"]
    updated_deferred = load_yaml(tmp_path / "roadmap-deferred.yaml")
    assert "agent_design wrote the Stage test PM-TEST-002 title plan" in updated_deferred["items"][0]["current_decision"]

    repeat_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert repeat_result.exit_code == 1
    assert "closeout is the next legal action" in repeat_result.stdout
    assert "agent_closeout" in repeat_result.stdout


def run_agent_design_fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> tuple[Path, Path, Path, Path, Path]:
    production_path, roadmap_path, manifest_root, plan_path, design_path = write_agent_design_fixture(tmp_path, monkeypatch)
    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert result.exit_code == 0, result.output
    return production_path, roadmap_path, manifest_root, plan_path, design_path


def test_manifest_runner_agent_closeout_requires_permission(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _design_path = run_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "closeout is the next legal action" in result.stdout
    assert "agent_closeout" in result.stdout


def test_manifest_runner_agent_closeout_closes_design_milestone_as_bounded_contract(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _design_path = run_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V3 applied one agent_closeout action." in result.stdout
    updated_production = load_yaml(production_path)
    milestone = updated_production["tracks"][0]["milestones"][1]
    assert milestone["state"] == "completed"
    assert milestone["completion_quality"] == "bounded_contract"
    assert milestone["completion_audit"].endswith("closeout.md")
    assert milestone["evidence_gates"][0]["required_status"] == "completed"
    updated_deferred = load_yaml(tmp_path / "roadmap-deferred.yaml")
    assert updated_deferred["items"] == []
    updated_archive = load_yaml(tmp_path / "roadmap-archive.yaml")
    archived_wr = updated_archive["items"][0]
    assert archived_wr["id"] == "WR-002"
    assert archived_wr["planning_state"] == "completed"
    assert archived_wr["completion_quality"] == "bounded_contract"
    updated_manifest = load_yaml(manifest_root / "pt-test.yaml")
    assert "create or link the design wr" in updated_manifest["next_legal_action"].lower()
    assert "agent_closeout as bounded_contract" in updated_manifest["milestones"][1]["next_legal_action"]


def test_manifest_runner_agent_closeout_rejects_runtime_proven_design_milestone(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(production: dict, _active: dict, _deferred: dict, _manifest: dict) -> None:
        production["tracks"][0]["milestones"][1]["completion_quality"] = "runtime_proven"

    production_path, roadmap_path, manifest_root, _plan_path, _design_path = write_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    design_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert design_result.exit_code == 0, design_result.output

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "docs or design milestones cannot close as runtime_proven" in result.stdout


def test_manifest_runner_agent_closeout_blocks_missing_evidence(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _design_path = write_agent_design_fixture(tmp_path, monkeypatch)
    manifest_data = load_yaml(manifest_root / "pt-test.yaml")
    manifest_data["next_legal_action"] = "PM-TEST-002 agent_design completed design/planning writes; stop for closeout."
    manifest_data["milestones"][1]["next_legal_action"] = manifest_data["next_legal_action"]
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "required production plan evidence is missing" in result.stdout
    assert not plan_path.exists()


def test_manifest_runner_agent_closeout_blocks_failed_validation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _design_path = run_agent_design_fixture(tmp_path, monkeypatch)
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: (_ for _ in ()).throw(WorkflowError("validation command failed: task docs:validate")),
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "validation command failed: task docs:validate" in result.stdout


def test_manifest_runner_agent_closeout_rejects_product_code_milestone(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _active: dict, _deferred: dict, manifest: dict) -> None:
        manifest["milestones"][1]["may_create_code"] = True

    production_path, roadmap_path, manifest_root, _plan_path, _design_path = write_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    manifest_data = load_yaml(manifest_root / "pt-test.yaml")
    manifest_data["next_legal_action"] = "PM-TEST-002 agent_design completed design/planning writes; stop for closeout."
    manifest_data["milestones"][1]["next_legal_action"] = manifest_data["next_legal_action"]
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "supports docs, design, or governance milestones" in result.stdout
    assert "only" in result.stdout


def test_manifest_runner_agent_closeout_stops_before_next_design_authoring(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _design_path = run_agent_design_fixture(tmp_path, monkeypatch)
    closeout_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_closeout",
            "--deny",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert closeout_result.exit_code == 0, closeout_result.output

    next_result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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

    assert next_result.exit_code == 0, next_result.output
    assert "Current milestone: PM-TEST-003 - PM-TEST-003 title" in next_result.stdout
    assert "Workflow action: track_expansion_required" in next_result.stdout
    assert "PM-TEST-007" not in next_result.stdout


def test_manifest_runner_full_track_continues_across_closeout_and_next_expansion(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _design_path = run_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--deny",
            "crate_creation",
            "--deny",
            "foundation_extraction",
            "--max-actions",
            "2",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V3 applied one agent_closeout action." in result.stdout
    assert "Manifest Runner V1 applied one auto_safe Track Expansion action." in result.stdout
    production = load_yaml(production_path)
    assert production["tracks"][0]["milestones"][1]["state"] == "completed"
    assert production["tracks"][0]["milestones"][2]["roadmap_links"]


def test_manifest_runner_auto_safe_and_agent_design_stop_before_next_milestone(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    deferred_path = tmp_path / "roadmap-deferred.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_path = manifest_root / "pt-test.yaml"
    plan_path = tmp_path / "plans" / "wr-003-ui-program-contract-design" / "plan.md"
    design_path = tmp_path / "ui-program-architecture.md"
    design_path.write_text(
        "---\ntitle: UI Program Architecture\nstatus: active\n---\n\n# UI Program Architecture\n\n## 13. Staged Implementation Plan\n\nExisting plan.\n",
        encoding="utf-8",
    )
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    manifest_data["milestones"][1]["agent_design"] = {
        "source_documents": ["docs-site/src/content/docs/workspace/track-execution-manifest.md"],
        "required_sections": ["UiProgram graph ownership"],
        "required_decisions": ["UiProgram remains UI-owned."],
        "acceptance_checklist": ["Design plan and architecture section exist."],
    }
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_path, manifest_data)
    monkeypatch.setattr("track_execution_manifest.default_contract_path", lambda item: plan_path)
    monkeypatch.setattr("track_execution_manifest.implementation_plan_path", lambda wr_id, milestone: repo_path(plan_path))
    monkeypatch.setattr(
        "track_execution_manifest.run_validation_commands",
        lambda commands: tuple(f"{command}: exit 0" for command in commands),
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V1 applied one auto_safe Track Expansion action." in result.stdout
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert plan_path.exists()
    assert not deferred_path.exists() or load_yaml(deferred_path)["items"][0]["id"] == "WR-003"
    next_result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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
    assert next_result.exit_code == 0, next_result.output
    assert "Current milestone: PM-TEST-002 - PM-TEST-002 title" in next_result.stdout
    assert "stop for closeout" in next_result.stdout
    assert "PM-TEST-007" not in next_result.stdout


def test_manifest_runner_product_code_denied_for_design_only_milestone(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _ = write_agent_design_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "workflow action is design_first" in result.stdout
    assert "no permitted runner action" in result.stdout
    assert not plan_path.exists()


def test_manifest_runner_product_code_denied_without_active_wr(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, roadmap: dict, _manifest: dict) -> None:
        roadmap["items"][1]["planning_state"] = "blocked_deferred"
        roadmap["items"][1]["blocker"] = 5

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "workflow action is design_first" in result.stdout


def test_manifest_runner_product_code_denied_without_production_plan(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        write_plan=False,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "accepted production plan is missing" in result.stdout
    assert plan_path.name in result.stdout


def test_manifest_runner_product_code_denied_without_exact_write_scopes(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, roadmap: dict, manifest: dict) -> None:
        roadmap["items"][1]["write_scopes"] = ["domain"]
        manifest["milestones"][1]["write_scope"] = ["domain"]
        manifest["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"] = ["domain"]

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "product_code_contract exact_allowed_implementation_write_scopes" in result.stdout
    assert "ambiguous or non-path scope: domain" in result.stdout


def test_manifest_runner_product_code_denied_without_validation_commands(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["validation_commands"] = ["blocked: define validation commands"]

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "validation_commands remains blocked" in result.stdout


def test_manifest_runner_product_code_denied_if_crate_creation_needed(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["may_create_crates"] = True

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "crate_creation is required" in result.stdout


def test_manifest_runner_product_code_rejects_crate_creation_without_exact_new_cargo_scope(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["may_create_crates"] = True

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "crate_creation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "crate_creation requires exact 'new:' Cargo.toml scope" in result.stdout


def test_manifest_runner_product_code_allows_crate_creation_with_exact_new_cargo_scope(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    crate_manifest = tmp_path / "domain" / "ui" / "ui_program" / "Cargo.toml"

    def mutate(_production: dict, roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["may_create_crates"] = True
        crate_scope = f"new: {repo_path(crate_manifest)}"
        roadmap["items"][1]["write_scopes"].append(crate_scope)
        manifest["milestones"][1]["write_scope"].append(crate_scope)
        manifest["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"].append(crate_scope)

    production_path, roadmap_path, manifest_root, plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    crate_scope = f"new: {repo_path(crate_manifest)}"
    plan_path.write_text(
        plan_path.read_text(encoding="utf-8")
        + f"\nAdditional exact crate creation scope:\n\n- `{crate_scope}`\n- `{repo_path(crate_manifest)}`\n",
        encoding="utf-8",
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "crate_creation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V4 verified one product_code implementation gate." in result.stdout


def test_manifest_runner_product_code_denied_if_foundation_extraction_requested(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, roadmap: dict, manifest: dict) -> None:
        roadmap["items"][1]["write_scopes"].append("foundation/meta/src/lib.rs")
        manifest["milestones"][1]["write_scope"].append("foundation/meta/src/lib.rs")
        manifest["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"].append(
            "foundation/meta/src/lib.rs"
        )

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "shared foundation/meta extraction" in result.stdout


def test_manifest_runner_product_code_allowed_for_synthetic_implementation_milestone(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V4 verified one product_code implementation gate." in result.stdout
    assert "Validation commands:" in result.stdout
    updated_production = load_yaml(production_path)
    assert updated_production["tracks"][0]["milestones"][1]["state"] == "active"


def test_manifest_runner_product_implementation_requires_product_code(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "product_implementation requires product_code" in result.stdout


def test_manifest_runner_product_code_alone_does_not_write_product_files(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )
    original = implementation_path.read_text(encoding="utf-8")
    monkeypatch.setattr(
        "track_execution_manifest.product_implementation_files",
        lambda _entry: {implementation_path: "// changed by product implementation\n"},
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V4 verified one product_code implementation gate." in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == original


def test_manifest_runner_product_implementation_writes_bounded_existing_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V5 wrote one bounded product_implementation slice." in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// changed by product implementation\n"


def test_manifest_runner_product_implementation_defaults_to_no_writer(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1].pop("implementation_writer")

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    original = implementation_path.read_text(encoding="utf-8")

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "implementation_writer strategy is no_writer" in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == original


def test_manifest_runner_template_writer_rejects_undeclared_output_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["implementation_writer"]["templates"][0]["file"] = "tools/workflow/not-in-scope.rs"

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "implementation_writer.allowed_files" in result.stdout


def test_manifest_runner_patch_writer_updates_declared_existing_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        writer = manifest["milestones"][1]["implementation_writer"]
        writer["strategy"] = "patch_writer"
        writer["templates"] = []
        writer["patches"] = [
            {
                "file": writer["allowed_files"][0],
                "find": "// implementation fixture\n",
                "replace": "// changed by patch writer\n",
            }
        ]

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert implementation_path.read_text(encoding="utf-8").strip() == "// changed by patch writer"


def test_manifest_runner_patch_writer_refuses_undeclared_new_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        writer = manifest["milestones"][1]["implementation_writer"]
        writer["strategy"] = "patch_writer"
        writer["templates"] = []
        writer["patches"] = [
            {
                "file": writer["allowed_files"][0],
                "find": "// implementation fixture\n",
                "replace": "// changed by patch writer\n",
            }
        ]

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    implementation_path.unlink()

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "patch find text not found" in result.stdout
    assert not implementation_path.exists()


def fake_codex_writer_changes_lib(workspace: Path, prompt: str) -> subprocess.CompletedProcess[str]:
    target = next(path for path in workspace.rglob("lib.rs") if "product" in path.parts and "src" in path.parts)
    target.write_text("// changed by agent writer\n", encoding="utf-8")
    return subprocess.CompletedProcess(args=["codex", "exec"], returncode=0, stdout="changed scoped file", stderr="")


def test_manifest_runner_agent_writer_imports_scoped_codex_diff(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        writer = manifest["milestones"][1]["implementation_writer"]
        writer["strategy"] = "agent_writer"
        writer["templates"] = []
        writer["patches"] = []
        writer["agent_prompt"] = "Change only the scoped implementation fixture."
        writer["agent_diff_protocol_version"] = "scoped-diff-v1"
        writer["agent_required_outputs"] = ["implementation file changed"]
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    monkeypatch.setattr("track_execution_manifest.run_codex_agent", fake_codex_writer_changes_lib)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V5 wrote one bounded product_implementation slice." in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// changed by agent writer\n"


def test_manifest_runner_agent_writer_rejects_forbidden_pattern_output(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        writer = manifest["milestones"][1]["implementation_writer"]
        writer["strategy"] = "agent_writer"
        writer["templates"] = []
        writer["patches"] = []
        writer["agent_prompt"] = "Change only the scoped implementation fixture."
        writer["agent_diff_protocol_version"] = "scoped-diff-v1"
        writer["agent_required_outputs"] = ["implementation file changed"]

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    implementation_path.write_text("// product implementation placeholder\n", encoding="utf-8")

    def fake_forbidden_agent(workspace: Path, prompt: str) -> subprocess.CompletedProcess[str]:
        target = next(path for path in workspace.rglob("lib.rs") if "product" in path.parts and "src" in path.parts)
        target.write_text("// foundation/meta is forbidden here\n", encoding="utf-8")
        return subprocess.CompletedProcess(args=["codex", "exec"], returncode=0, stdout="changed scoped file", stderr="")

    monkeypatch.setattr("track_execution_manifest.run_codex_agent", fake_forbidden_agent)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "forbidden pattern" in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// product implementation placeholder\n"


def test_manifest_runner_agent_writer_rejects_out_of_scope_diff(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        writer = manifest["milestones"][1]["implementation_writer"]
        writer["strategy"] = "agent_writer"
        writer["templates"] = []
        writer["patches"] = []
        writer["agent_prompt"] = "Change only the scoped implementation fixture."
        writer["agent_diff_protocol_version"] = "scoped-diff-v1"

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    original = implementation_path.read_text(encoding="utf-8")

    def fake_out_of_scope_agent(workspace: Path, prompt: str) -> subprocess.CompletedProcess[str]:
        target = workspace / "out-of-scope.rs"
        target.write_text("// out of scope\n", encoding="utf-8")
        return subprocess.CompletedProcess(args=["codex", "exec"], returncode=0, stdout="changed undeclared file", stderr="")

    monkeypatch.setattr("track_execution_manifest.run_codex_agent", fake_out_of_scope_agent)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "changed undeclared files" in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == original


def test_manifest_runner_product_implementation_honors_new_file_scope(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, roadmap: dict, manifest: dict) -> None:
        roadmap["items"][1]["write_scopes"][0] = f"new: {roadmap['items'][1]['write_scopes'][0]}"
        manifest["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"][0] = (
            f"new: {manifest['milestones'][1]['product_code_contract']['exact_allowed_implementation_write_scopes'][0]}"
        )

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    implementation_path.unlink()

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert implementation_path.read_text(encoding="utf-8") == "// changed by product implementation\n"


def test_manifest_runner_product_implementation_rejects_unmarked_new_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )
    implementation_path.unlink()

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "must be marked" in result.stdout
    assert "new:" in result.stdout
    assert not implementation_path.exists()


def test_new_file_scope_errors_reject_untracked_existing_repo_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    candidate = tmp_path / "domain/ui/ui_widgets/src/untracked_existing.rs"
    candidate.parent.mkdir(parents=True)
    candidate.write_text("// existing but untracked\n", encoding="utf-8")
    monkeypatch.setattr(track_manifest_module, "REPO_ROOT", tmp_path)
    monkeypatch.setattr(track_manifest_module, "git_tracks_path", lambda _path: False)

    errors = new_file_scope_errors(
        "PM-TEST-NEW",
        ["domain/ui/ui_widgets/src/untracked_existing.rs"],
        label="product_code_contract",
    )

    assert errors == [
        "PM-TEST-NEW: product_code_contract new file scope must be marked with 'new:': "
        "domain/ui/ui_widgets/src/untracked_existing.rs"
    ]


def test_manifest_runner_product_implementation_rejects_forbidden_scope(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        implementation_scope = manifest["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"][0]
        manifest["milestones"][1]["forbidden_scope"].append(implementation_scope)

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "would touch forbidden scope" in result.stdout


def write_proof_aggregation_fixture(
    tmp_path: Path,
    *,
    mutate: Callable[[dict, dict, dict], None] | None = None,
) -> tuple[Path, Path, Path]:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    closeout_path = tmp_path / "closeouts" / "pm-test-001" / "closeout.md"
    closeout_path.parent.mkdir(parents=True)
    closeout_path.write_text(
        "---\ntitle: PM-TEST-001 Closeout\nstatus: completed\n---\n\nRuntime proven prior evidence.\n",
        encoding="utf-8",
    )

    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["state"] = "completed"
    production_data["tracks"][0]["milestones"][0]["completion_quality"] = "runtime_proven"
    production_data["tracks"][0]["milestones"][0]["completion_audit"] = repo_path(closeout_path)
    production_data["tracks"][0]["milestones"][1]["kind"] = "hardening"
    production_data["tracks"][0]["milestones"][1]["state"] = "active"
    production_data["tracks"][0]["milestones"][1]["dependencies"] = ["PM-TEST-001"]
    production_data["tracks"][0]["milestones"][1]["roadmap_links"] = ["WR-002"]

    roadmap_data = valid_state()
    roadmap_data["items"][0]["planning_state"] = "completed"
    roadmap_data["items"][0]["completion_quality"] = "runtime_proven"
    roadmap_data["items"][0]["completion_audit"] = repo_path(closeout_path)
    roadmap_data["items"][0]["write_scopes"] = ["tools/workflow/prior-proof.rs"]
    roadmap_data["items"][1]["planning_state"] = "current_candidate"
    roadmap_data["items"][1]["blocker"] = 2
    roadmap_data["items"][1]["gate"] = "Implementation-ready"
    roadmap_data["items"][1]["write_scopes"] = [
        "tools/workflow/test_workflow.py",
        "docs-site/src/content/docs/reports/implementation-plans/wr-002-product-code/plan.md",
    ]

    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["write_scope"] = ["tools/workflow/prior-proof.rs"]
    manifest_data["milestones"][0]["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [
        "tools/workflow/prior-proof.rs"
    ]
    manifest_data["milestones"][0]["runtime_closeout_contract"]["files_changed_report"] = [
        "tools/workflow/prior-proof.rs"
    ]
    manifest_data["milestones"][0]["implementation_writer"]["allowed_files"] = ["tools/workflow/prior-proof.rs"]
    manifest_data["milestones"][0]["implementation_writer"]["templates"][0]["file"] = "tools/workflow/prior-proof.rs"
    manifest_data["milestones"][1]["milestone_type"] = "hardening"
    manifest_data["milestones"][1]["execution_kind"] = "proof_aggregation"
    manifest_data["milestones"][1]["predecessor_dependencies"] = ["PM-TEST-001"]
    manifest_data["milestones"][1]["write_scope"] = ["tools/workflow/test_workflow.py"]
    manifest_data["milestones"][1]["may_create_code"] = True
    manifest_data["milestones"][1]["may_modify_production_behavior"] = True
    add_test_implementation_contracts(
        manifest_data["milestones"][1],
        exact_scope="tools/workflow/test_workflow.py",
    )
    manifest_data["milestones"][1]["implementation_writer"] = {
        "strategy": "proof_aggregation_writer",
        "aggregation_only": True,
        "required_prior_milestones": ["PM-TEST-001"],
        "required_prior_completion_quality": "runtime_proven",
        "required_evidence_categories": [
            "headless fixture",
            "diagnostics",
            "source-map proof",
            "runtime artifact evidence",
            "reproducibility evidence",
        ],
        "allowed_write_scopes": [
            "docs-site/src/content/docs/reports/implementation-plans/wr-002-product-code/plan.md"
        ],
        "required_outputs": ["aggregate prior runtime_proven closeout evidence"],
        "forbidden_scopes": ["tools/workflow/prior-proof.rs", "foundation/meta", "MaterialProgram"],
        "forbidden_patterns": ["foundation/meta", "MaterialProgram"],
        "new_file_policy": "existing_files_only",
        "validation_commands": ["uv run pytest tools/workflow/test_workflow.py"],
        "closeout_path": manifest_data["milestones"][1]["expected_closeout_path"],
        "stop_conditions": ["stop if prior evidence is missing"],
    }
    if mutate is not None:
        mutate(production_data, roadmap_data, manifest_data)
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    return production_path, roadmap_path, manifest_root


def proof_aggregation_audit_errors(
    production_path: Path,
    roadmap_path: Path,
    manifest_root: Path,
) -> list[str]:
    planning = load_production_tracks(production_path)
    track = planning.tracks[0]
    roadmap = load_roadmap(roadmap_path)
    loaded = load_track_execution_manifest("PT-TEST", root=manifest_root)
    assert loaded is not None
    return audit_manifest(loaded, track=track, roadmap=roadmap)


def test_proof_aggregation_writer_requires_prior_milestones(tmp_path: Path) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["implementation_writer"]["required_prior_milestones"] = []

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("requires required_prior_milestones" in error for error in errors)


def test_proof_aggregation_writer_requires_prior_runtime_proven_completion(tmp_path: Path) -> None:
    def mutate(production: dict, _roadmap: dict, _manifest: dict) -> None:
        production["tracks"][0]["milestones"][0]["completion_quality"] = "bounded_contract"

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("expected runtime_proven" in error for error in errors)


def test_proof_aggregation_writer_blocks_missing_prior_closeout(tmp_path: Path) -> None:
    def mutate(production: dict, _roadmap: dict, _manifest: dict) -> None:
        production["tracks"][0]["milestones"][0]["completion_audit"] = str(tmp_path / "missing-closeout.md")

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("closeout is missing" in error for error in errors)


def test_proof_aggregation_writer_blocks_missing_evidence_category(tmp_path: Path) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["implementation_writer"]["required_evidence_categories"].remove(
            "reproducibility evidence"
        )

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("missing required evidence categories" in error for error in errors)


def test_proof_aggregation_writer_requires_machine_readable_prior_evidence_for_full_automation(
    tmp_path: Path,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["full_automation_target"] = True

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("closeout is missing closeout_evidence metadata" in error for error in errors)


def test_proof_aggregation_writer_reads_machine_readable_prior_evidence_for_full_automation(
    tmp_path: Path,
) -> None:
    def mutate(production: dict, _roadmap: dict, manifest: dict) -> None:
        closeout_ref = production["tracks"][0]["milestones"][0]["completion_audit"]
        closeout = Path(closeout_ref)
        if not closeout.is_absolute():
            closeout = tmp_path / closeout_ref
        closeout.write_text(
            "---\n"
            "title: PM-TEST-001 Closeout\n"
            "status: completed\n"
            "closeout_evidence:\n"
            "  milestone_id: PM-TEST-001\n"
            "  wr_id: WR-001\n"
            "  completion_quality: runtime_proven\n"
            "  evidence_categories:\n"
            "    - headless fixture\n"
            "    - diagnostics\n"
            "    - source-map proof\n"
            "    - runtime artifact evidence\n"
            "    - reproducibility evidence\n"
            "  validation_commands:\n"
            "    - uv run pytest tools/workflow/test_workflow.py\n"
            "  validation_results:\n"
            "    - 'uv run pytest tools/workflow/test_workflow.py: exit 0'\n"
            "  files_changed:\n"
            "    - tools/workflow/prior-proof.rs\n"
            "  runtime_artifacts:\n"
            "    - prior runtime artifact\n"
            "  diagnostics:\n"
            "    - prior diagnostics\n"
            "  source_maps:\n"
            "    - prior source map\n"
            "  known_gaps:\n"
            "    - bounded fixture only\n"
            "  closeout_path: " + closeout_ref + "\n"
            "---\n\nRuntime proven prior evidence.\n",
            encoding="utf-8",
        )
        manifest["full_automation_target"] = True

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert errors == []


def test_proof_aggregation_writer_refuses_prior_product_file_outputs(tmp_path: Path) -> None:
    def mutate(_production: dict, roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["implementation_writer"]["allowed_write_scopes"] = [
            "tools/workflow/prior-proof.rs"
        ]
        roadmap["items"][1]["write_scopes"].append("tools/workflow/prior-proof.rs")

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("must not modify prior proof-slice product file" in error for error in errors)


def test_proof_aggregation_writer_requires_wr_scoped_outputs(tmp_path: Path) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["implementation_writer"]["allowed_write_scopes"] = [
            "docs-site/src/content/docs/reports/outside-wr-scope.md"
        ]

    errors = proof_aggregation_audit_errors(*write_proof_aggregation_fixture(tmp_path, mutate=mutate))

    assert any("implementation_writer allowed scope" in error and "not covered" in error for error in errors)


def prepare_full_automation_product_fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    *,
    mutate: Callable[[dict, dict, dict, Path, Path, Path], None] | None = None,
) -> tuple[Path, Path, Path, Path, Path, Path]:
    production_path, roadmap_path, manifest_root, plan_path, implementation_path, closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )
    production = load_yaml(production_path)
    roadmap = load_yaml(roadmap_path)
    manifest = load_yaml(manifest_root / "pt-test.yaml")
    manifest["ai_executable"] = True
    entry = manifest["milestones"][1]
    entry["milestone_kind"] = "implementation_proof"
    entry["execution_kind"] = "implementation_proof"
    entry["permission_classes_required"] = [
        "agent_design",
        "product_code",
        "product_implementation",
        "runtime_closeout",
    ]
    entry["required_evidence_categories"] = ["runtime_test"]
    entry["agent_design_contract"]["allowed_write_scopes"] = [repo_path(plan_path)]
    entry["agent_design_contract"]["planning_write_scope"] = [repo_path(plan_path)]
    entry["agent_design_contract"]["expected_output_paths"] = [repo_path(plan_path)]
    entry["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [repo_path(implementation_path)]
    entry["runtime_closeout_contract"]["files_changed_report"] = [repo_path(implementation_path)]
    entry["implementation_writer"]["allowed_files"] = [repo_path(implementation_path)]
    entry["implementation_writer"]["templates"][0]["file"] = repo_path(implementation_path)
    if mutate is not None:
        mutate(production, roadmap, manifest, plan_path, implementation_path, closeout_path)
    write_yaml(
        plan_path.with_suffix(".execution.yaml"),
        {
            "version": 1,
            "track_id": "PT-TEST",
            "milestone_id": entry["milestone_id"],
            "wr_id": entry["owning_wr"],
            "executor_kind": "proof_aggregation" if entry.get("execution_kind") == "proof_aggregation" else "product_implementation",
            "allowed_outputs": [repo_path(implementation_path)],
            "new_outputs": [],
            "forbidden_outputs": ["foundation/meta"],
            "validation_commands": list(entry["implementation_writer"]["validation_commands"]),
            "evidence_required": ["runtime_test"],
            "closeout_path": entry["expected_closeout_path"],
        },
    )
    write_yaml(production_path, production)
    write_yaml(roadmap_path, roadmap)
    write_yaml(manifest_root / "pt-test.yaml", manifest)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
    return production_path, roadmap_path, manifest_root, plan_path, implementation_path, closeout_path


def write_track_execution_lock_fixture(
    track_id: str,
    production_path: Path,
    roadmap_path: Path,
    manifest_root: Path,
    lock_root: Path,
    *,
    allow: list[str] | None = None,
    deny: list[str] | None = None,
    mutate: Callable[[dict], None] | None = None,
) -> Path:
    loaded = load_track_execution_manifest(track_id, root=manifest_root)
    assert loaded is not None
    data = build_track_execution_lock_data(
        loaded,
        production_source=production_path,
        roadmap_source=roadmap_path,
        locked_by="test",
        granted_permissions=allow or sorted(track_manifest_module.FULL_TRACK_PERMISSION_SET),
        denied_permissions=deny or ["crate_creation", "foundation_extraction"],
    )
    if mutate is not None:
        mutate(data)
    data = track_manifest_module.TrackExecutionLock.model_validate(data).model_dump(mode="json")
    lock_root.mkdir(parents=True, exist_ok=True)
    path = lock_root / f"{track_id.lower()}.yaml"
    write_yaml(path, data)
    return path


def write_execution_pack_and_lock_fixture(
    track_id: str,
    production_path: Path,
    roadmap_path: Path,
    manifest_root: Path,
    lock_root: Path,
    *,
    allow: list[str] | None = None,
    deny: list[str] | None = None,
    mutate_lock: Callable[[dict], None] | None = None,
) -> tuple[Path, Path]:
    pack_root = production_path.parent / "contract-packs"
    pack = compile_contract_pack(
        track_id,
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )
    pack_path = write_contract_pack(pack, root=pack_root)
    lock = build_execution_lock(
        track_id,
        locked_by="test",
        contract_pack_root=pack_root,
        granted_permissions=allow or sorted(track_manifest_module.FULL_TRACK_PERMISSION_SET),
        denied_permissions=deny or ["crate_creation", "foundation_extraction"],
    )
    lock_data = lock.model_dump(mode="json")
    if mutate_lock is not None:
        mutate_lock(lock_data)
    lock_root.mkdir(parents=True, exist_ok=True)
    lock_path = lock_root / f"{track_id.lower()}.yaml"
    write_yaml(lock_path, lock_data)
    return pack_path, lock_path


def invoke_full_automation_audit(
    production_path: Path,
    roadmap_path: Path,
    manifest_root: Path,
    lock_root: Path | None = None,
    *,
    require_lock: bool = False,
) -> object:
    contract_pack_root = production_path.parent / "contract-packs"
    args = [
        "audit-track",
        "--track",
        "PT-TEST",
        "--full-automation",
        "--production-source",
        str(production_path),
        "--roadmap-source",
        str(roadmap_path),
        "--manifest-source-root",
        str(manifest_root),
        "--contract-pack-root",
        str(contract_pack_root),
    ]
    if require_lock:
        args.append("--require-lock")
    if lock_root is not None:
        args.extend(["--lock-source-root", str(lock_root)])
    return CliRunner().invoke(
        track_manifest_app,
        args,
    )


def test_full_automation_preflight_fails_when_future_milestone_has_no_writer(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1]["implementation_writer"]["strategy"] = "no_writer"

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "execution:preflight failed" in result.stdout
    assert "implementation/proof action cannot use no_writer" in result.stdout


def test_goal_reports_full_automation_blockers_for_manifest_target(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["full_automation_target"] = True
        manifest["milestones"][1]["implementation_writer"]["strategy"] = "no_writer"

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

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
    assert "Full automation readiness: blocked" in result.stdout
    assert "implementation_writer.strategy must not be no_writer" in result.stdout
    assert "Unmet gates: none detected" not in result.stdout


def test_full_automation_preflight_requires_execution_kind(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1].pop("execution_kind", None)

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "unsupported execution_kind None" in result.stdout


def test_full_automation_preflight_rejects_legacy_execution_kind(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1]["execution_kind"] = "hardening"

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "Input should be" in result.stdout


def test_full_automation_preflight_fails_when_proof_aggregation_lacks_writer(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        entry = manifest["milestones"][1]
        entry["execution_kind"] = "proof_aggregation"
        entry["implementation_writer"]["strategy"] = "template_writer"

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "proof_aggregation" in result.stdout
    assert "proof_aggregation_writer" in result.stdout


def test_full_automation_preflight_fails_when_validation_command_is_prose(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        entry = manifest["milestones"][1]
        entry["validation_commands"] = ["focused tests named by the owning production plan"]
        entry["product_code_contract"]["validation_commands"] = ["focused tests named by the owning production plan"]

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "validation command is prose/non-executable" in result.stdout


def test_full_automation_preflight_fails_when_closeout_contract_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1].pop("runtime_closeout_contract")

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "runtime_closeout_contract" in result.stdout


def test_full_track_runner_preflight_catches_future_no_writer_before_mutation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    lock_root = tmp_path / "locks"
    production = load_yaml(production_path)
    roadmap = load_yaml(roadmap_path)
    manifest = load_yaml(manifest_root / "pt-test.yaml")

    production["tracks"][0]["milestones"][1]["state"] = "designing"
    production["tracks"][0]["milestones"][1]["roadmap_links"] = []
    manifest["milestones"][1].pop("owning_wr")
    manifest["milestones"][1]["future_wr_candidate"] = "WR-TBD-TEST-002"
    add_test_auto_safe_contract(manifest["milestones"][1])
    manifest["milestones"][1]["implementation_writer"]["strategy"] = "no_writer"
    write_yaml(production_path, production)
    write_yaml(roadmap_path, roadmap)
    write_yaml(manifest_root / "pt-test.yaml", manifest)
    pack_root = production_path.parent / "contract-packs"
    write_execution_pack_and_lock_fixture("PT-TEST", production_path, roadmap_path, manifest_root, lock_root)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--mode",
            "full-track",
            "--max-actions",
            "999",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
            "--contract-pack-root",
            str(pack_root),
        ],
    )

    assert result.exit_code == 1
    assert "planning_expansion executor" in result.stdout
    assert not (tmp_path / "roadmap-deferred.yaml").exists()


def test_full_track_runner_fails_without_execution_lock(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    pack_root = production_path.parent / "contract-packs"
    write_contract_pack(
        compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
        ),
        root=pack_root,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--deny",
            "crate_creation",
            "--deny",
            "foundation_extraction",
            "--mode",
            "full-track",
            "--max-actions",
            "999",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(tmp_path / "locks"),
            "--contract-pack-root",
            str(pack_root),
        ],
    )

    assert result.exit_code == 1
    assert "requires current Execution Lock" in result.stdout


def test_full_track_runner_fails_with_stale_manifest_digest(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    lock_root = tmp_path / "locks"
    pack_root = production_path.parent / "contract-packs"
    write_execution_pack_and_lock_fixture(
        "PT-TEST",
        production_path,
        roadmap_path,
        manifest_root,
        lock_root,
        mutate_lock=lambda data: data.update({"contract_pack_digest": "0" * 64}),
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--mode",
            "full-track",
            "--max-actions",
            "999",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
            "--contract-pack-root",
            str(pack_root),
        ],
    )

    assert result.exit_code == 1
    assert "execution lock contract_pack_digest is stale" in result.stdout


def test_full_track_runner_fails_when_requested_permission_exceeds_lock(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    lock_root = tmp_path / "locks"
    pack_root = production_path.parent / "contract-packs"
    write_execution_pack_and_lock_fixture(
        "PT-TEST",
        production_path,
        roadmap_path,
        manifest_root,
        lock_root,
        allow=["auto_safe", "agent_design", "agent_closeout", "product_code"],
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--mode",
            "full-track",
            "--max-actions",
            "999",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
            "--contract-pack-root",
            str(pack_root),
        ],
    )

    assert result.exit_code == 1
    assert "requested permissions exceed execution lock grants" in result.stdout
    assert "product_implementation" in result.stdout


def test_full_track_preflight_reports_strategic_human_gate(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1]["permission_classes_required"].append("foundation_extraction")

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    lock_root = tmp_path / "locks"
    pack_root = production_path.parent / "contract-packs"
    write_execution_pack_and_lock_fixture("PT-TEST", production_path, roadmap_path, manifest_root, lock_root)

    result = invoke_full_automation_audit(
        production_path,
        roadmap_path,
        manifest_root,
        lock_root,
        require_lock=True,
    )

    assert result.exit_code == 1
    assert "foundation_extraction" in result.stdout
    assert "foundation_extraction is a strategic human gate" in result.stdout


def test_full_track_runner_requires_explicit_full_track_mode(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--max-actions",
            "999",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "requires explicit --mode" in result.stdout
    assert "full-track" in result.stdout


def test_full_automation_preflight_passes_for_fully_specified_contract(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 0, result.output
    assert "execution preflight passed" in result.stdout


def test_full_automation_audit_requires_current_execution_lock_when_requested(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    lock_root = tmp_path / "locks"
    write_execution_pack_and_lock_fixture("PT-TEST", production_path, roadmap_path, manifest_root, lock_root)

    result = invoke_full_automation_audit(
        production_path,
        roadmap_path,
        manifest_root,
        lock_root,
        require_lock=True,
    )

    assert result.exit_code == 0, result.output
    assert "Execution Harness lock passed" in result.stdout


def test_full_track_runner_writes_run_ledger_after_successful_action(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout = (
        prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    )
    lock_root = tmp_path / "locks"
    run_root = tmp_path / "runs"
    pack_root = production_path.parent / "contract-packs"
    write_execution_pack_and_lock_fixture("PT-TEST", production_path, roadmap_path, manifest_root, lock_root)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--deny",
            "crate_creation",
            "--deny",
            "foundation_extraction",
            "--mode",
            "full-track",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
            "--contract-pack-root",
            str(pack_root),
            "--run-ledger-root",
            str(run_root),
        ],
    )

    assert result.exit_code == 1, result.output
    assert "output parent directory does not exist" in result.stdout
    assert "Run ledger:" in result.stdout
    ledger_paths = list((run_root / "pt-test").glob("*.yaml"))
    assert len(ledger_paths) == 1
    ledger = load_yaml(ledger_paths[0])
    assert ledger["track_id"] == "PT-TEST"
    assert ledger["actions"][0]["status"] == "failed"
    assert ledger["actions"][0]["executor_kind"] == "product_implementation"


def test_agent_track_allows_full_permission_set_and_refreshes_lock_after_preflight(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout = (
        prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    )
    lock_root = tmp_path / "locks"
    run_root = tmp_path / "runs"

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--deny",
            "crate_creation",
            "--deny",
            "foundation_extraction",
            "--mode",
            "agent-track",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
            "--run-ledger-root",
            str(run_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V5 wrote one bounded product_implementation slice." in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// changed by product implementation\n"
    lock = load_yaml(lock_root / "pt-test.yaml")
    assert lock["ai_executable"] is True
    assert "product_implementation" in lock["granted_permissions"]


def test_agent_track_creates_missing_wr_and_plan_without_product_code(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
    )
    lock_root = tmp_path / "locks"

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--mode",
            "agent-track",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V1 applied one auto_safe Track Expansion action." in result.stdout
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert plan_path.exists()
    assert not (lock_root / "pt-test.yaml").exists()


def test_agent_track_creates_lock_before_agent_writer_product_import(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        writer = manifest["milestones"][1]["implementation_writer"]
        writer["strategy"] = "agent_writer"
        writer["templates"] = []
        writer["patches"] = []
        writer["agent_prompt"] = "Change only the scoped implementation fixture."
        writer["agent_diff_protocol_version"] = "scoped-diff-v1"
        writer["agent_required_outputs"] = ["implementation file changed"]

    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout_path = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    lock_root = tmp_path / "locks"
    monkeypatch.setattr("track_execution_manifest.run_codex_agent", fake_codex_writer_changes_lib)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--deny",
            "crate_creation",
            "--deny",
            "foundation_extraction",
            "--mode",
            "agent-track",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--lock-source-root",
            str(lock_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V5 wrote one bounded product_implementation slice." in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// changed by agent writer\n"
    assert (lock_root / "pt-test.yaml").exists()


def test_execution_contract_compiler_rejects_missing_plan(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        write_plan=False,
    )
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    with pytest.raises(WorkflowError, match="implementation/design plan is missing"):
        compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
        )


def test_execution_contract_pack_preflight_passes_for_runtime_action(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    assert pack.actions
    assert preflight_pack(pack) == []
    assert pack.actions[0].writer_strategy == "template_writer"


def test_execution_preflight_rejects_prose_validation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1]["product_code_contract"]["validation_commands"] = ["run relevant tests"]
        manifest["milestones"][1]["implementation_writer"]["validation_commands"] = ["run relevant tests"]

    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    errors = preflight_pack(pack)
    assert any("validation command is prose/non-executable" in error for error in errors)


def test_execution_lock_detects_stale_sources(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    pack_root = tmp_path / "contract-packs"
    lock_root = tmp_path / "locks"
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )
    write_contract_pack(pack, root=pack_root)
    lock = build_execution_lock(
        "PT-TEST",
        locked_by="test",
        contract_pack_root=pack_root,
        granted_permissions=["agent_design", "product_code", "product_implementation", "runtime_closeout"],
        denied_permissions=[],
    )
    write_execution_lock(lock, root=lock_root)
    production_path.write_text(production_path.read_text(encoding="utf-8") + "\n", encoding="utf-8")

    errors = execution_lock_errors(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=lock_root,
        requested_permissions={"product_code"},
    )

    assert any("source digest is stale" in error for error in errors)


def execution_test_action(
    *,
    writer_strategy: str = "agent_writer",
    allowed_outputs: list[str] | None = None,
    new_outputs: list[str] | None = None,
    forbidden_outputs: list[str] | None = None,
    forbidden_patterns: list[str] | None = None,
) -> ActionContract:
    return ActionContract(
        action_id="PT-TEST:PM-TEST-002:WR-002",
        track_id="PT-TEST",
        milestone_id="PM-TEST-002",
        wr_id="WR-002",
        execution_kind="implementation_proof",
        executor_kind="product_implementation",
        authority_level="implementation_runtime_proof",
        permissions_required=["product_code", "product_implementation"],
        allowed_outputs=allowed_outputs or ["src/lib.rs"],
        new_outputs=new_outputs or [],
        forbidden_outputs=forbidden_outputs or ["foundation/meta"],
        forbidden_patterns=forbidden_patterns or [],
        writer_strategy=writer_strategy,
        validation_commands=["python3 --version"],
        evidence_required=[EvidenceRequirement(kind="runtime_test", name="runtime test")],
        closeout_contract=CloseoutContract(
            path="docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            completion_quality="runtime_proven",
            evidence_required=[EvidenceRequirement(kind="runtime_test", name="runtime test")],
        ),
        rollback_policy=RollbackPolicy(policy="reject import on scope, digest, or validation failure"),
        stop_conditions=["stop after one implementation action"],
    )


class FakeExecutionAgent:
    def __init__(self, edits: dict[str, str]) -> None:
        self.edits = edits

    def run(self, *, workspace: Path, prompt: str) -> AgentResult:
        for relative, content in self.edits.items():
            target = workspace / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        return AgentResult(returncode=0, stdout="ok", stderr="")


def test_execution_agent_writer_requires_lock_and_imports_scoped_diff(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action()

    with pytest.raises(WorkflowError, match="requires a current execution lock"):
        run_writer(
            action,
            backend=FakeExecutionAgent({"src/lib.rs": "// after\n"}),
            lock_validated=False,
            repo_root=tmp_path,
        )

    written = run_writer(
        action,
        backend=FakeExecutionAgent({"src/lib.rs": "// after\n"}),
        lock_validated=True,
        repo_root=tmp_path,
    )

    assert written == [source]
    assert source.read_text(encoding="utf-8") == "// after\n"


def test_execution_agent_writer_rejects_out_of_scope_diff(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action()

    with pytest.raises(WorkflowError, match="changed undeclared file other.rs"):
        run_writer(
            action,
            backend=FakeExecutionAgent({"src/lib.rs": "// after\n", "other.rs": "// no\n"}),
            lock_validated=True,
            repo_root=tmp_path,
        )

    assert source.read_text(encoding="utf-8") == "// before\n"


def test_execution_agent_writer_rejects_undeclared_new_file(tmp_path: Path) -> None:
    action = execution_test_action(allowed_outputs=["src/new.rs"])

    with pytest.raises(WorkflowError, match="created undeclared new file src/new.rs"):
        run_writer(
            action,
            backend=FakeExecutionAgent({"src/new.rs": "// new\n"}),
            lock_validated=True,
            repo_root=tmp_path,
        )

    assert not (tmp_path / "src" / "new.rs").exists()


def test_execution_agent_writer_accepts_declared_new_file(tmp_path: Path) -> None:
    (tmp_path / "src").mkdir(parents=True)
    action = execution_test_action(allowed_outputs=[], new_outputs=["src/new.rs"])

    written = run_writer(
        action,
        backend=FakeExecutionAgent({"src/new.rs": "// new\n"}),
        lock_validated=True,
        repo_root=tmp_path,
    )

    assert written == [tmp_path / "src" / "new.rs"]
    assert (tmp_path / "src" / "new.rs").read_text(encoding="utf-8") == "// new\n"


def test_execution_agent_writer_rejects_forbidden_pattern(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action(forbidden_patterns=[r"src/lib\.rs"])

    with pytest.raises(WorkflowError, match="matches forbidden pattern"):
        run_writer(
            action,
            backend=FakeExecutionAgent({"src/lib.rs": "// after\n"}),
            lock_validated=True,
            repo_root=tmp_path,
        )

    assert source.read_text(encoding="utf-8") == "// before\n"


def test_execution_runner_writes_machine_readable_evidence(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action(writer_strategy="template_writer")
    action.template_outputs["src/lib.rs"] = "// generated\n"
    action.validation_commands = ["python3 --version"]

    result = run_next_action(
        ContractPack(
            track_id="PT-TEST",
            generated_at="2026-06-01T00:00:00Z",
            source_digests={"source.yaml": "digest"},
            actions=[action],
        ),
        lock_validated=True,
        repo_root=tmp_path,
        evidence_root=tmp_path / "evidence",
    )

    assert source.read_text(encoding="utf-8") == "// generated\n"
    assert len(result.evidence_paths) == 1
    evidence = load_yaml(result.evidence_paths[0])
    assert evidence["track_id"] == "PT-TEST"
    assert evidence["milestone_id"] == "PM-TEST-002"
    assert evidence["evidence_kind"] == "runtime_test"
    assert evidence["status"] == "passed"


def test_execution_cli_writes_run_ledger_after_successful_action(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action(writer_strategy="template_writer")
    action.template_outputs["src/lib.rs"] = "// generated\n"
    action.validation_commands = ["python3 --version"]
    pack_root = tmp_path / "packs"
    lock_root = tmp_path / "locks"
    ledger_root = tmp_path / "ledgers"
    write_contract_pack(
        ContractPack(
            track_id="PT-TEST",
            generated_at="2026-06-01T00:00:00Z",
            source_digests={"tools/workflow/test_workflow.py": sha256(Path(__file__).read_bytes()).hexdigest()},
            actions=[action],
        ),
        root=pack_root,
    )
    write_execution_lock(
        build_execution_lock(
            "PT-TEST",
            locked_by="test",
            contract_pack_root=pack_root,
            granted_permissions=["product_code", "product_implementation"],
            denied_permissions=[],
        ),
        root=lock_root,
    )

    result = CliRunner().invoke(
        execution_app,
        [
            "run",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--contract-pack-root",
            str(pack_root),
            "--lock-root",
            str(lock_root),
            "--run-ledger-root",
            str(ledger_root),
            "--repo-root",
            str(tmp_path),
        ],
    )

    assert result.exit_code == 0, result.output
    ledgers = list((ledger_root / "pt-test").glob("*.yaml"))
    assert len(ledgers) == 1
    ledger = load_yaml(ledgers[0])
    assert ledger["actions"][0]["action_id"] == action.action_id
    assert ledger["actions"][0]["files_changed"] == [repo_path(source)]


def test_execution_validation_command_registry_rejects_unsafe_forms() -> None:
    action = execution_test_action()
    action.validation_commands = ["task docs:validate && task planning:validate"]
    assert any("shell metacharacters" in error for error in preflight_pack(ContractPack(
        track_id="PT-TEST",
        generated_at="2026-06-01T00:00:00Z",
        source_digests={"source.yaml": "digest"},
        actions=[action],
    )))

    action.validation_commands = ["git reset --hard"]
    assert any("safe command registry" in error for error in preflight_pack(ContractPack(
        track_id="PT-TEST",
        generated_at="2026-06-01T00:00:00Z",
        source_digests={"source.yaml": "digest"},
        actions=[action],
    )))

    action.validation_commands = [
        {
            "command_id": "python3:version",
            "argv": ["python3", "--version"],
            "cwd": "/tmp",
        }
    ]
    assert any("cwd must be repo-relative" in error for error in preflight_pack(ContractPack(
        track_id="PT-TEST",
        generated_at="2026-06-01T00:00:00Z",
        source_digests={"source.yaml": "digest"},
        actions=[action],
    )))

    with pytest.raises(ValueError, match="command_id must match argv-derived id"):
        action.validation_commands = [
            {
                "command_id": "not-the-registry-id",
                "argv": ["python3", "--version"],
            }
        ]


def test_execution_transactional_writer_leaves_main_workspace_unchanged_on_validation_failure(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action(writer_strategy="template_writer")
    action.template_outputs["src/lib.rs"] = "// generated before failing validation\n"
    action.validation_commands = ["python3 -m pytest missing-test-file.py"]

    with pytest.raises(WorkflowError, match="validation failed"):
        run_action(action, lock_validated=True, repo_root=tmp_path, evidence_root=tmp_path / "evidence")

    assert source.read_text(encoding="utf-8") == "// before\n"
    assert not (tmp_path / "evidence").exists()


def test_execution_proof_aggregation_requires_prior_machine_evidence(tmp_path: Path) -> None:
    action = ActionContract(
        action_id="PT-TEST:PM-TEST-003:WR-003",
        track_id="PT-TEST",
        milestone_id="PM-TEST-003",
        wr_id="WR-003",
        execution_kind="proof_aggregation",
        executor_kind="proof_aggregation",
        authority_level="runtime_proof_aggregation",
        permissions_required=["product_code", "product_implementation"],
        allowed_outputs=[],
        new_outputs=[],
        forbidden_outputs=["product/src/lib.rs"],
        writer_strategy="proof_aggregation_writer",
        validation_commands=["python3 --version"],
        evidence_required=[EvidenceRequirement(kind="runtime_test", name="runtime test")],
        closeout_contract=CloseoutContract(
            path="docs-site/src/content/docs/reports/closeouts/pm-test-003/closeout.md",
            completion_quality="runtime_proven",
            evidence_required=[EvidenceRequirement(kind="runtime_test", name="runtime test")],
        ),
        rollback_policy=RollbackPolicy(policy="aggregation-only; do not patch prior proof files"),
        stop_conditions=["stop if prior evidence is missing"],
        required_prior_milestones=["PM-TEST-002"],
        required_prior_completion_quality="runtime_proven",
    )

    with pytest.raises(WorkflowError, match="prior milestone PM-TEST-002 is missing runtime_test evidence"):
        run_action(action, lock_validated=True, repo_root=tmp_path, evidence_root=tmp_path / "evidence")

    write_evidence_record(
        passed_record(
            track_id="PT-TEST",
            milestone_id="PM-TEST-002",
            action_id="PT-TEST:PM-TEST-002:WR-002",
            evidence_kind="runtime_test",
            name="runtime test",
            paths=[],
            validation_commands=["python3:version"],
        ),
        root=tmp_path / "evidence",
    )

    result = run_action(action, lock_validated=True, repo_root=tmp_path, evidence_root=tmp_path / "evidence")

    assert result.written_paths == ()
    assert len(result.evidence_paths) == 1


def test_production_validation_requires_contract_pack_for_full_automation_target(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    manifest = load_yaml(manifest_root / "pt-test.yaml")
    manifest["full_automation_target"] = True
    write_yaml(manifest_root / "pt-test.yaml", manifest)
    monkeypatch.setattr("execution.compiler.CONTRACT_PACK_ROOT", tmp_path / "missing-contract-packs")
    state = ProductionPlanningState.model_validate(load_yaml(production_path))

    errors = validate_manifest_backed_tracks(state, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("full automation target requires Execution Contract Pack" in error for error in errors)


def test_production_validation_accepts_valid_contract_pack_for_full_automation_target(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    manifest = load_yaml(manifest_root / "pt-test.yaml")
    manifest["full_automation_target"] = True
    write_yaml(manifest_root / "pt-test.yaml", manifest)
    pack_root = tmp_path / "contract-packs"
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
    write_contract_pack(
        compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
        ),
        root=pack_root,
    )
    monkeypatch.setattr("execution.compiler.CONTRACT_PACK_ROOT", pack_root)
    state = ProductionPlanningState.model_validate(load_yaml(production_path))

    errors = validate_manifest_backed_tracks(state, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert not any("Execution Contract Pack" in error for error in errors)

def test_single_step_run_can_operate_on_current_milestone_without_full_preflight(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        manifest["milestones"][1]["milestone_kind"] = "legacy-generic-kind"
        manifest["milestones"][1].pop("execution_kind", None)

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V4 verified one product_code implementation gate." in result.stdout


def test_current_pm012_proof_aggregation_writer_contract_validates() -> None:
    planning = load_production_tracks()
    track = next(candidate for candidate in planning.tracks if candidate.id == "PT-UI-PROGRAM")
    roadmap = load_roadmap()
    loaded = load_track_execution_manifest("PT-UI-PROGRAM")
    assert loaded is not None

    errors = audit_manifest(loaded, track=track, roadmap=roadmap)

    assert errors == []


def test_current_pm012_does_not_start_materialprogram_and_pm013_is_handoff_only() -> None:
    loaded = load_track_execution_manifest("PT-UI-PROGRAM")
    assert loaded is not None
    pm012 = loaded.manifest.by_milestone_id["PM-UI-PROGRAM-012"]
    pm013 = loaded.manifest.by_milestone_id["PM-UI-PROGRAM-013"]

    assert loaded.manifest.full_automation_target
    assert pm012.execution_kind == "proof_aggregation"
    assert pm012.implementation_writer is not None
    assert pm012.implementation_writer.strategy == "proof_aggregation_writer"
    assert "product_implementation" in pm012.permission_classes_required
    assert "MaterialProgram implementation" in pm012.implementation_writer.forbidden_scopes
    assert pm013.execution_kind == "handoff_closeout"
    assert pm013.milestone_type == "closeout"
    assert not pm013.may_create_code
    assert pm013.product_code_contract is None


def test_manifest_runner_runtime_closeout_closes_after_product_code_evidence(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )
    make_runtime_closeout_ready(production_path, roadmap_path, manifest_root, closeout_path)
    runtime_artifact_path = "tools/workflow/runtime-proof.yaml"
    manifest_path = manifest_root / "pt-test.yaml"
    manifest_data = load_yaml(manifest_path)
    manifest_data["milestones"][1]["runtime_closeout_contract"]["files_changed_report"] = [
        runtime_artifact_path,
    ]
    write_yaml(manifest_path, manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "agent_closeout",
            "--max-actions",
            "2",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner runtime closeout completed one implementation milestone." in result.stdout
    assert closeout_path.exists()
    closeout_text = closeout_path.read_text(encoding="utf-8")
    assert runtime_artifact_path in closeout_text
    production = load_yaml(production_path)
    assert production["tracks"][0]["state"] == "completed"
    milestone = production["tracks"][0]["milestones"][1]
    assert milestone["state"] == "completed"
    assert milestone["completion_quality"] == "runtime_proven"
    archive = load_yaml(roadmap_path.with_name("roadmap-archive.yaml"))
    archived = next(item for item in archive["items"] if item["id"] == "WR-002")
    assert archived["completion_quality"] == "runtime_proven"


def test_manifest_runner_runtime_closeout_stops_at_missing_runtime_evidence(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )
    make_runtime_closeout_ready(
        production_path,
        roadmap_path,
        manifest_root,
        closeout_path,
        validation_commands=["task docs:validate"],
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "agent_closeout",
            "--max-actions",
            "2",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "runtime closeout requires at least one runtime/test validation" in result.stdout
    assert "command" in result.stdout
    assert not closeout_path.exists()


def test_manifest_runner_runtime_closeout_stops_at_failed_validation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )
    make_runtime_closeout_ready(production_path, roadmap_path, manifest_root, closeout_path)

    def fail_validation(commands: list[str]) -> tuple[str, ...]:
        raise WorkflowError(f"validation command failed: {commands[0]}")

    monkeypatch.setattr("track_execution_manifest.run_validation_commands", fail_validation)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "agent_closeout",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "validation command failed:" in result.stdout
    assert not closeout_path.exists()


def test_manifest_runner_product_code_stops_after_one_implementation_wr_by_default(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert result.stdout.count("Manifest Runner V4 verified one product_code implementation gate.") == 1
    assert "Must stop after this action: yes" in result.stdout


def test_manifest_runner_product_code_rejects_runtime_proven_claim_without_evidence(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, roadmap: dict, _manifest: dict) -> None:
        roadmap["items"][1]["completion_quality"] = "runtime_proven"

    production_path, roadmap_path, manifest_root, _plan_path, _implementation_path, _closeout_path = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "cannot claim runtime_proven before closeout" in result.stdout
    assert "runtime/test evidence exists" in result.stdout


def test_manifest_runner_agent_design_creates_implementation_plan_without_product_code(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V1 applied one auto_safe Track Expansion action." in result.stdout
    assert "Manifest Runner V2 applied one agent_design action." in result.stdout
    assert "Manifest Runner V4" not in result.stdout
    assert plan_path.exists()
    plan_text = plan_path.read_text(encoding="utf-8")
    assert "## Exact Files/Modules Expected To Change" in plan_text
    assert "## Expected Methods/Functions" in plan_text
    assert "## Tests To Add/Change" in plan_text
    assert "## Compatibility / Rollback Plan" in plan_text
    assert "Post-plan transition: Manifest Runner V2 records the milestone as `active`" in plan_text
    for path in scope_paths:
        assert repo_path(path) in plan_text
    production = load_yaml(production_path)
    assert production["tracks"][0]["milestones"][1]["state"] == "active"
    active_roadmap = load_yaml(roadmap_path)
    wr_item = next(item for item in active_roadmap["items"] if item["id"] != "WR-001")
    assert wr_item["planning_state"] == "current_candidate"
    assert wr_item["blocker"] == 2
    for path in scope_paths:
        assert normalize_repo_path(repo_path(path)) in wr_item["write_scopes"]
    assert normalize_repo_path(repo_path(plan_path)) in wr_item["write_scopes"]

    rerun = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert rerun.exit_code == 0, rerun.output
    assert "Manifest Runner stopped before product_code." in rerun.stdout
    assert "Manifest Runner V4" not in rerun.stdout


def mutate_manifest_to_runtime_slice(
    *,
    milestone_id: str,
    title: str,
    stage: str,
    proof_kind: str,
    target: str,
    validation_commands: list[str] | None = None,
) -> Callable[[dict, dict, dict], None]:
    commands = validation_commands or ["uv run pytest tools/workflow/test_workflow.py"]

    def mutate(production: dict, _roadmap: dict, manifest: dict) -> None:
        production["tracks"][0]["milestones"][1]["id"] = milestone_id
        production["tracks"][0]["milestones"][1]["title"] = title
        entry = manifest["milestones"][1]
        exact_scopes = ["tools/workflow/test_workflow.py"]
        entry["milestone_id"] = milestone_id
        entry["title"] = title
        entry["stage"] = stage
        entry["implementation_proof_kind"] = proof_kind
        entry["write_scope"] = list(exact_scopes)
        entry["validation_commands"] = list(commands)
        entry["expected_closeout_path"] = (
            f"docs-site/src/content/docs/reports/closeouts/{milestone_id.lower()}-{proof_kind}/closeout.md"
        )
        entry["contract_parameters"] = {
            "proof_slice_id": stage.removeprefix("Stage "),
            "proof_slice_title": title,
            "target_control_surface": target,
            "exact_allowed_implementation_write_scopes": list(exact_scopes),
            "required_function_method_scope": [f"{target} bounded functions only."],
            "tests_to_add_change": [f"Focused {target} proof tests in scoped modules only."],
            "runtime_evidence_required": [f"{title} runtime/test evidence."],
            "validation_commands": list(commands),
            "source_documents": ["docs-site/src/content/docs/workspace/track-execution-manifest.md"],
            "required_sections": [f"{title} implementation plan.", f"{target} proof scope."],
            "required_decisions": [f"{target} remains bounded to {stage}."],
            "acceptance_checklist": [f"Plan names {target} proof scope."],
            "rollback_compatibility_expectations": [f"Rollback is limited to the exact {target} proof files."],
        }
        entry.pop("agent_design", None)
        entry["agent_design_contract"]["source_documents"] = entry["contract_parameters"]["source_documents"]
        entry["agent_design_contract"]["required_sections"] = entry["contract_parameters"]["required_sections"]
        entry["agent_design_contract"]["required_decisions"] = entry["contract_parameters"]["required_decisions"]
        entry["agent_design_contract"]["acceptance_checklist"] = entry["contract_parameters"]["acceptance_checklist"]
        entry["agent_design_contract"]["validation_commands"] = list(commands)
        entry["product_code_contract"]["required_function_method_scope"] = entry["contract_parameters"]["required_function_method_scope"]
        entry["product_code_contract"]["exact_allowed_implementation_write_scopes"] = list(exact_scopes)
        entry["product_code_contract"]["tests_to_add_change"] = entry["contract_parameters"]["tests_to_add_change"]
        entry["product_code_contract"]["runtime_evidence_required"] = entry["contract_parameters"]["runtime_evidence_required"]
        entry["product_code_contract"]["validation_commands"] = list(commands)
        entry["product_code_contract"]["rollback_compatibility_expectations"] = entry["contract_parameters"]["rollback_compatibility_expectations"]
        entry["runtime_closeout_contract"]["runtime_test_evidence_required"] = entry["contract_parameters"]["runtime_evidence_required"]
        entry["runtime_closeout_contract"]["validation_commands"] = list(commands)
        entry["runtime_closeout_contract"]["files_changed_report"] = list(exact_scopes)
        entry["implementation_writer"]["allowed_files"] = list(exact_scopes)
        entry["implementation_writer"]["validation_commands"] = list(commands)
        if entry["implementation_writer"].get("templates"):
            entry["implementation_writer"]["templates"][0]["file"] = exact_scopes[0]

    return mutate


def run_agent_design_for_fixture(
    production_path: Path,
    roadmap_path: Path,
    manifest_root: Path,
) -> CliRunner:
    return CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )


def test_generated_6b_plan_does_not_contain_6a_label_stale_text(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate_manifest_to_runtime_slice(
            milestone_id="PM-TEST-008",
            title="6B Button Route Event Host Command Proof",
            stage="Stage 6B",
            proof_kind="6b-button-route-event-host-command-proof",
            target="Button",
        ),
    )

    result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 0, result.output
    plan_text = plan_path.read_text(encoding="utf-8")
    assert "Button Route Event Host Command Proof" in plan_text
    assert "UiEventPacket" not in plan_text or "Label text output" not in plan_text
    assert "Label text output" not in plan_text
    assert "6A implementation" not in plan_text
    assert "Stop before PM-TEST-008" not in plan_text


def test_generated_plan_consistency_rejects_wrong_milestone_reference(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate_manifest_to_runtime_slice(
            milestone_id="PM-TEST-008",
            title="6B Button Route Event Host Command Proof",
            stage="Stage 6B",
            proof_kind="6b-button-route-event-host-command-proof",
            target="Button",
        ),
    )
    result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)
    assert result.exit_code == 0, result.output
    context = resolve_manifest_command_context(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_source_root=manifest_root,
    )
    entry, _milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id[entry.owning_wr]
    broken_text = plan_path.read_text(encoding="utf-8").replace(
        "6B Button Route Event Host Command Proof",
        "6B Wrong Proof",
    )

    errors = implementation_plan_consistency_errors_from_text(
        entry,
        roadmap_item=roadmap_item,
        text=broken_text,
        plan_path=plan_path,
    )

    assert any("missing current proof term" in error and "Button Route Event Host" in error for error in errors)


def test_generated_plan_consistency_rejects_prose_validation_command(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate_manifest_to_runtime_slice(
            milestone_id="PM-TEST-008",
            title="6B Button Route Event Host Command Proof",
            stage="Stage 6B",
            proof_kind="6b-button-route-event-host-command-proof",
            target="Button",
            validation_commands=["focused 6B tests named by the owning production plan"],
        ),
    )

    result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "non-executable placeholder" in result.stdout


def test_generated_plan_consistency_rejects_stale_closeout_evidence_terms(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate_manifest_to_runtime_slice(
            milestone_id="PM-TEST-008",
            title="6B Button Route Event Host Command Proof",
            stage="Stage 6B",
            proof_kind="6b-button-route-event-host-command-proof",
            target="Button",
        ),
    )
    result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)
    assert result.exit_code == 0, result.output
    context = resolve_manifest_command_context(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_source_root=manifest_root,
    )
    entry, _milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id[entry.owning_wr]
    broken_text = plan_path.read_text(encoding="utf-8") + "\n- Closeout evidence must include Label text output.\n"

    errors = implementation_plan_consistency_errors_from_text(
        entry,
        roadmap_item=roadmap_item,
        text=broken_text,
        plan_path=plan_path,
    )

    assert any("stale slice term 'label text output'" in error for error in errors)


def test_product_code_gate_refuses_unsafe_generated_plan(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate_manifest_to_runtime_slice(
            milestone_id="PM-TEST-008",
            title="6B Button Route Event Host Command Proof",
            stage="Stage 6B",
            proof_kind="6b-button-route-event-host-command-proof",
            target="Button",
        ),
    )
    result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)
    assert result.exit_code == 0, result.output
    plan_path.write_text(plan_path.read_text(encoding="utf-8") + "\n- Label text output stale evidence.\n", encoding="utf-8")

    product_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert product_result.exit_code == 1
    assert "stale slice term" in product_result.stdout
    assert "label" in product_result.stdout


def test_product_code_gate_permits_corrected_pm008_plan_after_validation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate_manifest_to_runtime_slice(
            milestone_id="PM-TEST-008",
            title="6B Button Route Event Host Command Proof",
            stage="Stage 6B",
            proof_kind="6b-button-route-event-host-command-proof",
            target="Button",
        ),
    )
    result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)
    assert result.exit_code == 0, result.output

    product_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert product_result.exit_code == 0, product_result.output
    assert "Manifest Runner V4 verified one product_code implementation gate." in product_result.stdout


def test_generated_plan_for_6c_and_6d_uses_stage_specific_terms(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    for milestone_id, title, stage, proof_kind, target in [
        ("PM-TEST-009", "6C InspectorField Binding State Proof", "Stage 6C", "6c-inspectorfield-binding-state-proof", "InspectorField"),
        ("PM-TEST-010", "6D ColorPicker ControlPackage Proof", "Stage 6D", "6d-colorpicker-controlpackage-proof", "ColorPicker"),
    ]:
        case_root = tmp_path / milestone_id.lower()
        case_root.mkdir()
        production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
            case_root,
            monkeypatch,
            mutate=mutate_manifest_to_runtime_slice(
                milestone_id=milestone_id,
                title=title,
                stage=stage,
                proof_kind=proof_kind,
                target=target,
            ),
        )
        result = run_agent_design_for_fixture(production_path, roadmap_path, manifest_root)
        assert result.exit_code == 0, result.output
        plan_text = plan_path.read_text(encoding="utf-8")
        assert title in plan_text
        assert target in plan_text
        assert "Label text output" not in plan_text


def test_manifest_runner_product_code_remains_blocked_until_implementation_plan_exists(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
    )
    expansion = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert expansion.exit_code == 0, expansion.output

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "workflow action is design_first" in result.stdout


def test_manifest_runner_stops_cleanly_when_agent_design_contract_is_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
    )
    expansion = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert expansion.exit_code == 0, expansion.output
    manifest_path = manifest_root / "pt-test.yaml"
    manifest_data = load_yaml(manifest_path)
    manifest_data["milestones"][1].pop("agent_design")
    manifest_data["milestones"][1].pop("agent_design_contract", None)
    write_yaml(manifest_path, manifest_data)

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "agent_design",
            "--allow",
            "product_code",
            "--deny",
            "crate_creation",
            "--deny",
            "foundation_extraction",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "needs agent_design_contract" in result.stdout
    assert "Manifest Runner V4" not in result.stdout


def test_complete_track_contracts_fills_missing_action_contracts(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def remove_action_contracts(_production: dict, _roadmap: dict, manifest: dict) -> None:
        milestone = manifest["milestones"][1]
        milestone.pop("agent_design", None)
        milestone.pop("agent_design_contract", None)
        milestone.pop("product_code_contract", None)
        milestone.pop("runtime_closeout_contract", None)
        milestone.pop("auto_safe_contract", None)

    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=remove_action_contracts,
    )
    monkeypatch.setattr(
        "track_execution_manifest.manifest_report_path",
        lambda track_id: str(tmp_path / "reports" / "track-execution-manifests" / track_id.lower() / "manifest.md"),
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "complete-track-contracts",
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
    manifest = load_yaml(manifest_root / "pt-test.yaml")
    milestone = manifest["milestones"][1]
    assert milestone["auto_safe_contract"]["generated_from_template_version"] == "v1"
    assert milestone["agent_design_contract"]["generated_contract_marker"] == "generated_by_production_complete_track_contracts"
    assert milestone["product_code_contract"]["exact_allowed_implementation_write_scopes"]
    assert milestone["runtime_closeout_contract"]["completion_quality_allowed"] == ["runtime_proven"]


def test_complete_track_contracts_fails_when_template_is_missing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def use_missing_template(_production: dict, _roadmap: dict, manifest: dict) -> None:
        manifest["milestones"][1]["template_key"] = "missing-template"

    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=use_missing_template,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "complete-track-contracts",
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
    assert "missing contract template" in result.stdout


def test_full_track_runner_refuses_missing_required_contracts(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def remove_action_contracts(_production: dict, _roadmap: dict, manifest: dict) -> None:
        milestone = manifest["milestones"][1]
        milestone.pop("agent_design", None)
        milestone.pop("agent_design_contract", None)
        milestone.pop("product_code_contract", None)
        milestone.pop("runtime_closeout_contract", None)
        milestone.pop("auto_safe_contract", None)

    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        mutate=remove_action_contracts,
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--allow",
            "agent_closeout",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "needs auto_safe_contract" in result.stdout
    assert "needs agent_design_contract" in result.stdout
    assert "needs product_code_contract" in result.stdout
    assert "runtime_closeout_contract" in result.stdout


def test_manifest_runner_product_code_allowed_after_agent_design_implementation_plan(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
    )
    plan_result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )
    assert plan_result.exit_code == 0, plan_result.output

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Manifest Runner V4 verified one product_code implementation gate." in result.stdout


def test_manifest_runner_pt_ui_program_pm007_plan_generation_without_product_code(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        track_id="PT-UI-PROGRAM",
        milestone_id="PM-UI-PROGRAM-007",
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-UI-PROGRAM",
            "--allow",
            "auto_safe",
            "--allow",
            "agent_design",
            "--deny",
            "product_code",
            "--max-actions",
            "10",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "PM-UI-PROGRAM-007" in result.stdout
    assert "Manifest Runner V4" not in result.stdout
    assert plan_path.exists()
    plan_text = plan_path.read_text(encoding="utf-8")
    assert "6A Label Structural UiFrame Text Proof" in plan_text
    assert "Stop before product/runtime code unless the command is rerun with `--allow product_code --allow product_implementation`" in plan_text
    assert load_yaml(production_path)["tracks"][0]["milestones"][1]["state"] == "active"


def test_manifest_runner_product_code_pt_ui_program_blocks_before_6a(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
        track_id="PT-UI-PROGRAM",
        milestone_id="PM-UI-PROGRAM-007",
    )

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "run-track",
            "--track",
            "PT-UI-PROGRAM",
            "--allow",
            "product_code",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "PM-UI-PROGRAM-007" in result.stdout
    assert "Track Expansion must create or link" in result.stdout
    assert "--allow auto_safe" in result.stdout


def test_manifest_backed_production_validation_passes_valid_manifest(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    planning = ProductionPlanningState.model_validate(valid_production_state())

    assert validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root) == []


def test_manifest_backed_validation_requires_truth_claims(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data.pop("truth_claims")
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("manifest-backed tracks must declare truth_claims" in error for error in errors)


def test_manifest_backed_validation_rejects_missing_completed_design_output(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][1]["state"] = "completed"
    production_data["tracks"][0]["milestones"][1]["completion_quality"] = "bounded_contract"
    production_data["tracks"][0]["milestones"][1]["completion_audit"] = (
        "docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md"
    )
    production_data["tracks"][0]["milestones"][1]["evidence_gates"] = [
        {
            "path": "docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            "required_status": "completed",
            "reason": "Test closeout.",
        }
    ]
    roadmap_data = valid_state()
    roadmap_data["items"][1]["planning_state"] = "completed"
    roadmap_data["items"][1]["completion_quality"] = "bounded_contract"
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][1]["agent_design_contract"]["expected_output_paths"] = [
        "docs-site/src/content/docs/design/active/missing-test-output.md"
    ]
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("completed design expected_output_path is missing" in error for error in errors)


def test_active_architecture_quality_allows_blocked_architecture_claim(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Architecture is intentionally not proven yet.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Architecture contracts remain unimplemented."],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert not any("architecture_runtime_proven requires a satisfied architecture_contract" in error for error in errors)


def test_completed_architecture_quality_requires_satisfied_architecture_claim(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["state"] = "completed"
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("architecture_runtime_proven requires a satisfied architecture_contract truth claim" in error for error in errors)


def test_proof_slice_runtime_quality_rejects_overclaiming_wording(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "proof_slice_runtime_proven"
    production_data["tracks"][0]["strategic_goal"] = "Prove the final architecture platform."
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "proof-slice-test",
            "claim_kind": "proof_slice",
            "claim_level": "proof_slice_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Bounded proof slices passed.",
            "required_docs": [
                {
                    "evidence_kind": "doc_exists",
                    "path": "tools/workflow/test_workflow.py",
                    "reason": "Fixture truth evidence path exists.",
                }
            ],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("production wording claims stronger truth than proof_slice_runtime_proven" in error for error in errors)


def test_satisfied_architecture_truth_claim_requires_code_contracts(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Architecture exists.",
            "required_docs": [],
            "required_code_contracts": [
                {
                    "evidence_kind": "module_path_exists",
                    "path": "domain/missing_test_architecture_module",
                    "reason": "Architecture module must exist.",
                }
            ],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("requires module path domain/missing_test_architecture_module" in error for error in errors)


def test_blocked_truth_claim_blocks_downstream_milestone(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"].append(
        {
            "claim_id": "blocked-architecture",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Architecture is not ready.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Architecture module missing."],
            "supersedes": [],
            "blocks_downstream": ["PM-TEST-002"],
        }
    )
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("truth claim blocked-architecture blocks downstream PM-TEST-002" in error for error in errors)


def test_manifest_next_surfaces_truth_claims(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())

    result = CliRunner().invoke(
        track_manifest_app,
        [
            "next",
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

    assert result.exit_code == 0
    assert "Truth claims:" in result.stdout
    assert "test-track-proof" in result.stdout


def test_goal_output_surfaces_truth_claims(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())

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

    assert result.exit_code == 0
    assert "Truth claims:" in result.stdout
    assert "test-track-proof" in result.stdout


def test_manifest_backed_validation_accepts_generated_scope_marker(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["write_scope"] = [
        "tools/workflow",
        "generated: production docs from task production:render",
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    assert validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root) == []


def test_manifest_backed_validation_rejects_uncovered_wr_write_scope(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["write_scope"] = ["docs-site/src/content/docs/workspace/track-execution-manifest.md"]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("manifest write_scope docs-site/src/content/docs/workspace/track-execution-manifest.md" in error for error in errors)
    assert any("owning WR WR-001 write_scopes" in error for error in errors)


def test_manifest_backed_validation_requires_generated_scope_marker(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["write_scope"] = ["generated production docs from task production:render"]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("must use 'generated:' or 'derived:'" in error for error in errors)


def test_pt_ui_program_manifest_write_scope_is_wr_covered() -> None:
    planning = load_production_tracks()
    roadmap = load_roadmap()
    track = find_track(planning, "PT-UI-PROGRAM")
    loaded_manifest = load_track_execution_manifest("PT-UI-PROGRAM")
    assert loaded_manifest is not None

    errors = audit_manifest(loaded_manifest, track=track, roadmap=roadmap)

    assert [error for error in errors if "manifest write_scope" in error] == []


def test_pt_ui_program_remaining_contracts_are_completed() -> None:
    loaded_manifest = load_track_execution_manifest("PT-UI-PROGRAM")
    assert loaded_manifest is not None
    manifest_by_id = loaded_manifest.manifest.by_milestone_id
    for milestone_id in (
        "PM-UI-PROGRAM-008",
        "PM-UI-PROGRAM-009",
        "PM-UI-PROGRAM-010",
        "PM-UI-PROGRAM-011",
        "PM-UI-PROGRAM-012",
    ):
        entry = manifest_by_id[milestone_id]
        assert entry.agent_design_contract is not None
        assert entry.product_code_contract is not None
        assert entry.runtime_closeout_contract is not None
        assert entry.product_code_contract.generated_from_template_version == "v1"
        assert entry.product_code_contract.exact_allowed_implementation_write_scopes

    six_f = manifest_by_id["PM-UI-PROGRAM-012"]
    six_f_text = "\n".join(six_f.product_code_contract.runtime_evidence_required)
    assert "must not implement missing prior-slice behavior" in "\n".join(six_f.agent_design_contract.required_decisions)
    assert "Missing behavior must return to the owning milestone" in six_f_text

    closeout = manifest_by_id["PM-UI-PROGRAM-013"]
    assert closeout.agent_closeout_contract is not None
    assert closeout.handoff_contract is not None
    assert "MaterialProgram implementation" in "\n".join(closeout.handoff_contract.proof_path_rules)
    assert any("foundation/meta extraction" in scope for scope in closeout.handoff_contract.forbidden_scopes)


def test_manifest_backed_production_validation_rejects_docs_only_code_permission(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][1]["may_create_code"] = True
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert "PM-TEST-002: docs-only manifest milestones cannot authorize code" in errors[0]


def test_production_validate_cli_checks_manifest_backed_tracks(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][1]["may_create_code"] = True
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        production_state_app,
        [
            "validate",
            "--source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "docs-only manifest milestones cannot authorize code" in result.stdout


def test_manifest_backed_production_validation_rejects_runtime_proven_docs_only(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][1]["completion_quality"] = "runtime_proven"
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert "PM-TEST-002: runtime_proven milestones cannot be docs-only" in errors[0]


def test_manifest_backed_completed_milestone_requires_closeout_and_completed_wr(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    milestone = production_data["tracks"][0]["milestones"][0]
    milestone["state"] = "completed"
    milestone["completion_quality"] = "bounded_contract"
    milestone["evidence_gates"] = [
        {
            "path": "docs-site/src/content/docs/reports/closeouts/wrong/closeout.md",
            "required_status": "completed",
            "reason": "Wrong closeout.",
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("completed manifest-backed milestone must reference expected closeout" in error for error in errors)
    assert any("WR-001" in error and "expected 'completed'" in error for error in errors)


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


def test_production_goal_stack_routes_to_first_incomplete_dependency_track() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_stack_state())
    roadmap = RoadmapState.model_validate(valid_state())
    root_track = find_track(planning, "PT-END")

    rendered = render_stack_goal(planning, roadmap, root_track)

    assert "Production Stack /goal Kickoff: PT-END" in rendered
    assert "- PT-BASE - Base production track: PM-BASE-001 -> execute_next_wr_implementation_contract" in rendered
    assert "- PT-END - Final production audit: PM-END-001 -> wait_for_dependency_completion" in rendered
    assert "Current single-track command: task ai:goal -- --track PT-BASE" in rendered
    assert "Use task ai:goal -- --track PT-END --stack as the stack coordinator" in rendered
    assert "Cross-track dependency waits are routing signals in stack mode" in rendered


def test_production_goal_stack_cli_renders_dependency_stack(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_stack_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-END",
            "--stack",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
        ],
    )

    assert result.exit_code == 0
    assert "Production Stack /goal Kickoff: PT-END" in result.stdout
    assert "task ai:goal -- --track PT-BASE" in result.stdout


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


def test_new_write_scope_paths_require_existing_parent_only() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = ["new: tools/workflow/future_scope_test.py"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_existing_write_scope_paths([roadmap.items[0]]) == []
    assert validate_changed_paths(["tools/workflow/future_scope_test.py"], roadmap.items[0].write_scopes) == []


def test_new_write_scope_paths_reject_missing_parent() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = ["new: missing/parent/future_scope_test.py"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_existing_write_scope_paths([roadmap.items[0]]) == [
        "WR-001:missing/parent/future_scope_test.py parent does not exist for new write scope"
    ]


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


def test_current_repository_next_batch_has_no_active_runtime_candidate_after_ui_program_6f_closeout() -> None:
    roadmap = load_roadmap()
    selected = select_batch_candidates(roadmap)
    dependency_puml = (REPO_ROOT / "docs-site/src/content/docs/workspace/diagrams/value-weighted-dependency-roadmap.puml").read_text(encoding="utf-8")
    candidates_puml = (REPO_ROOT / "docs-site/src/content/docs/workspace/diagrams/current-roadmap-candidates.puml").read_text(encoding="utf-8")

    assert [item.id for item in selected] == []
    assert roadmap.by_id["WR-146"].planning_state == "completed"
    assert roadmap.by_id["WR-089"].planning_state == "completed"
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
