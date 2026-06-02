from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import copy

import json

import tempfile

import subprocess

from collections.abc import Callable, Iterator

from hashlib import sha256

from pathlib import Path

from types import SimpleNamespace

import pytest

import typer

import yaml

from typer.testing import CliRunner

import track_sources.audit as track_source_audit_module

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
    render_contract_prompt,
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

from production_track_cli import app as production_track_cli_app

import track_control_cli

from track_control_cli import app as track_control_app

from track_sources.audit import audit_manifest, first_current_manifest_entry, new_file_scope_errors

from track_sources.manifest import FULL_TRACK_PERMISSION_SET, load_track_execution_manifest

from execution.cli import app as execution_app

from execution.compiler import compile_contract_pack, harness_source_digest_map, load_contract_pack, write_contract_pack

from execution.contracts import (
    ActionContract,
    CloseoutContract,
    ContractPack,
    EvidenceRequirement,
    RollbackPolicy,
    ValidationCommand,
    validation_command_from_string,
)

from execution.locks import build_execution_lock, execution_lock_errors, write_execution_lock

from execution.preflight import preflight_action, preflight_pack

from execution.evidence import passed_record, write_evidence_record

from execution.closeout_claims import closeout_claim_errors

from execution.runner import agent_transcript_root, run_action, run_next_action

from execution.writers import AgentResult, AgentWriterError, CodexExecBackend, action_prompt, run_writer, subprocess_output_text

from prompt_doctrine import QUALITY_DOCTRINE_ID, audit_prompt_doctrine

from truth.certificates import TruthCertificate, TruthFinding, certificate_errors_for_claim, write_certificate

from truth.cli import app as truth_app

from truth.conformance.evidence import verify_evidence_records

from truth.conformance.rust_module_shape import verify_rust_module_shape

from truth.conformance.spec import ConformanceSpec

from track_sources.manifest import ManifestTruthClaim

from truth.verifiers import TRACK_EXECUTION_HARNESS_VERIFIER, run_verifier, verifier_source_paths

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

from ai_task import build_shapes, render_shape

from repo_hygiene import batch_manifest_errors, local_branches

from roadmap_intake import (
    activate_deferred_roadmap_item,
    apply_intake_proposal,
    build_intake_proposal,
    load_intake_proposal,
    proposal_to_yaml_data,
    roadmap_data_with_deferred_activation,
    roadmap_data_with_promotion,
    roadmap_data_with_proposal,
    switch_current_candidate,
    validate_intake_item_scopes,
    write_intake_proposal,
)

def preserve_repo_test_artifacts() -> Iterator[None]:
    roots = (
        REPO_ROOT / "docs-site/src/content/docs/reports/execution-evidence/pt-test",
        REPO_ROOT / "docs-site/src/content/docs/reports/track-execution-runs/pt-test",
    )
    isolated_files = (
        REPO_ROOT / "docs-site/src/content/docs/workspace/execution-contract-packs/pt-test.yaml",
        REPO_ROOT / "docs-site/src/content/docs/workspace/execution-locks/pt-test.yaml",
    )
    snapshots: dict[Path, bytes] = {}
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.is_file():
                snapshots[path] = path.read_bytes()
    isolated_snapshots = {path: path.read_bytes() for path in isolated_files if path.exists()}
    for path in isolated_files:
        if path.exists():
            path.unlink()

    yield

    for path in isolated_files:
        if path.exists():
            path.unlink()
        if path in isolated_snapshots:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(isolated_snapshots[path])
    for root in roots:
        if root.exists():
            for path in sorted((candidate for candidate in root.rglob("*") if candidate.is_file()), reverse=True):
                if path not in snapshots:
                    path.unlink()
        for path, content in snapshots.items():
            if path.is_relative_to(root):
                path.parent.mkdir(parents=True, exist_ok=True)
                if not path.exists() or path.read_bytes() != content:
                    path.write_bytes(content)
        if root.exists():
            for directory in sorted((candidate for candidate in root.rglob("*") if candidate.is_dir()), reverse=True):
                try:
                    directory.rmdir()
                except OSError:
                    pass

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
                "claim_level": "bounded_contract",
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
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
    return production_path, roadmap_path, manifest_root, plan_path, design_path

def valid_product_code_state(
    *,
    plan_path: Path,
    closeout_path: Path,
    implementation_path: Path,
    test_path: Path,
) -> tuple[dict, dict, dict]:
    source_root = plan_path.parents[2]

    def fixture_repo_path(path: Path) -> str:
        return path.resolve().relative_to(source_root.resolve()).as_posix()

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
        fixture_repo_path(implementation_path),
        fixture_repo_path(test_path),
        fixture_repo_path(closeout_path),
    ]
    roadmap_data["items"][1]["validations"] = ["task docs:validate"]

    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["milestone_type"] = "implementation"
    manifest_data["milestones"][0]["may_create_code"] = True
    manifest_data["milestones"][0]["may_modify_production_behavior"] = True
    manifest_data["milestones"][1]["milestone_type"] = "implementation"
    manifest_data["milestones"][1]["required_evidence_categories"] = ["runtime_test"]
    manifest_data["milestones"][1]["execution_kind"] = "implementation_proof"
    manifest_data["milestones"][1]["closeout_strategy"] = "runtime_proven_closeout"
    manifest_data["milestones"][1]["write_scope"] = [
        fixture_repo_path(implementation_path),
        fixture_repo_path(test_path),
        fixture_repo_path(closeout_path),
    ]
    manifest_data["milestones"][1]["validation_commands"] = ["task docs:validate"]
    manifest_data["milestones"][1]["expected_closeout_path"] = fixture_repo_path(closeout_path)
    manifest_data["milestones"][1]["may_create_code"] = True
    manifest_data["milestones"][1]["may_create_crates"] = False
    manifest_data["milestones"][1]["may_modify_production_behavior"] = True
    manifest_data["milestones"][1]["next_legal_action"] = "Run product_code for PM-TEST-002."
    add_test_implementation_contracts(manifest_data["milestones"][1], exact_scope=fixture_repo_path(implementation_path))
    manifest_data["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [
        fixture_repo_path(implementation_path),
        fixture_repo_path(test_path),
        fixture_repo_path(closeout_path),
    ]
    manifest_data["milestones"][1]["runtime_closeout_contract"]["files_changed_report"] = [
        fixture_repo_path(implementation_path),
        fixture_repo_path(test_path),
        fixture_repo_path(closeout_path),
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
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
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

    source_root = roadmap_path.parent

    def fixture_repo_path(path: Path) -> str:
        try:
            return path.resolve().relative_to(source_root.resolve()).as_posix()
        except ValueError:
            return repo_path(path)

    production = load_yaml(production_path)
    roadmap = load_yaml(roadmap_path)
    manifest = load_yaml(manifest_path)

    for item_data in roadmap["items"]:
        if item_data["id"] == wr_id:
            item_data["validations"] = commands
            for scope in [
                fixture_repo_path(production_path),
                fixture_repo_path(roadmap_path),
                fixture_repo_path(archive_path),
                fixture_repo_path(deferred_path),
                fixture_repo_path(manifest_path),
                fixture_repo_path(manifest_report),
                fixture_repo_path(closeout_path),
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
    manifest["milestones"][1]["expected_closeout_path"] = fixture_repo_path(closeout_path)
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

    def fixture_repo_path(path: Path) -> str:
        return path.resolve().relative_to(tmp_path.resolve()).as_posix()

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
    manifest_data["ai_executable"] = True
    manifest_data["full_automation_target"] = True
    manifest_data["milestones"][1]["milestone_id"] = milestone_id
    manifest_data["milestones"][1]["title"] = "6A Label Structural UiFrame Text Proof"
    manifest_data["milestones"][1]["milestone_type"] = "implementation"
    manifest_data["milestones"][1]["required_evidence_categories"] = ["runtime_test"]
    manifest_data["milestones"][1].pop("owning_wr", None)
    manifest_data["milestones"][1]["future_wr_candidate"] = "WR-TBD-TEST-002"
    manifest_data["milestones"][1]["write_scope"] = [fixture_repo_path(path) for path in scope_paths]
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
    manifest_data["milestones"][1]["expected_closeout_path"] = fixture_repo_path(closeout_path)
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
        exact_scope=fixture_repo_path(scope_paths[0]),
    )
    manifest_data["milestones"][1]["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [
        f"new: {fixture_repo_path(path)}" for path in scope_paths
    ]
    manifest_data["milestones"][1]["runtime_closeout_contract"]["files_changed_report"] = [
        fixture_repo_path(path) for path in scope_paths
    ]
    manifest_data["milestones"][1]["implementation_writer"]["allowed_files"] = [
        f"new: {fixture_repo_path(path)}" for path in scope_paths
    ]
    manifest_data["next_legal_action"] = f"Create or link the WR for {milestone_id}."

    if mutate is not None:
        mutate(production_data, active_roadmap, manifest_data)

    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, active_roadmap)
    write_yaml(deferred_path, deferred_roadmap)
    write_yaml(manifest_path, manifest_data)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
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

def write_stub_taskfile(root: Path) -> None:
    root.joinpath("Taskfile.yml").write_text(
        'version: "3"\n\ntasks:\n'
        '  production:render:\n    cmds:\n      - "true"\n'
        '  production:validate:\n    cmds:\n      - "true"\n'
        '  production:check:\n    cmds:\n      - "true"\n'
        '  roadmap:render:\n    cmds:\n      - "true"\n'
        '  roadmap:validate:\n    cmds:\n      - "true"\n'
        '  roadmap:check:\n    cmds:\n      - "true"\n'
        '  docs:validate:\n    cmds:\n      - "true"\n'
        '  planning:validate:\n    cmds:\n      - "true"\n',
        encoding="utf-8",
    )

