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

from fixtures.source_model import *

def fake_codex_writer_changes_lib(workspace: Path, prompt: str) -> subprocess.CompletedProcess[str]:
    target = next(path for path in workspace.rglob("lib.rs") if "product" in path.parts and "src" in path.parts)
    target.write_text("// changed by agent writer\n", encoding="utf-8")
    return subprocess.CompletedProcess(args=["codex", "exec"], returncode=0, stdout="changed scoped file", stderr="")

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
    workflow_test_path = tmp_path / "tools/workflow/test_workflow.py"
    workflow_test_path.parent.mkdir(parents=True, exist_ok=True)
    workflow_test_path.write_text("def test_fixture_validation_passes():\n    assert True\n", encoding="utf-8")
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
    plan_scope = plan_path.relative_to(tmp_path).as_posix()
    implementation_scope = implementation_path.relative_to(tmp_path).as_posix()
    entry["agent_design_contract"]["allowed_write_scopes"] = [plan_scope]
    entry["agent_design_contract"]["planning_write_scope"] = [plan_scope]
    entry["agent_design_contract"]["expected_output_paths"] = [plan_scope]
    entry["product_code_contract"]["exact_allowed_implementation_write_scopes"] = [f"new: {implementation_scope}"]
    entry["runtime_closeout_contract"]["files_changed_report"] = [implementation_scope]
    entry["implementation_writer"]["allowed_files"] = [f"new: {implementation_scope}"]
    entry["implementation_writer"]["templates"][0]["file"] = implementation_scope
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
    (tmp_path / evidence_output).parent.mkdir(parents=True, exist_ok=True)
    entry["write_scope"] = [implementation_scope, evidence_output]
    entry["product_code_contract"]["exact_allowed_implementation_write_scopes"].append(f"new: {evidence_output}")
    entry["runtime_closeout_contract"]["files_changed_report"].append(evidence_output)
    entry["implementation_writer"]["allowed_files"].append(f"new: {evidence_output}")
    for item in roadmap.get("items", []):
        if item.get("id") == entry.get("owning_wr"):
            item["write_scopes"] = [
                implementation_scope,
                evidence_output,
            ]
    if plan_path.exists():
        plan_path.write_text(
            plan_path.read_text(encoding="utf-8") + f"\n- Execution evidence output: `{evidence_output}`\n",
            encoding="utf-8",
        )
    if mutate is not None:
        mutate(production, roadmap, manifest, plan_path, implementation_path, closeout_path)
    sidecar_execution_kind = entry.get("execution_kind")
    write_yaml(
        plan_path.parent / "plan.contract.yaml",
        {
            "version": 1,
            "milestone_id": entry["milestone_id"],
            "wr_id": entry["owning_wr"],
            "execution_kind": sidecar_execution_kind,
            "executor_kind": "proof_aggregation" if sidecar_execution_kind == "proof_aggregation" else "product_implementation",
            "authority_level": entry["authority_level"],
            "permissions_required": ["product_code", "product_implementation"],
            "writer_strategy": entry["implementation_writer"]["strategy"],
            "allowed_outputs": [implementation_scope],
            "new_outputs": [evidence_output],
            "forbidden_outputs": ["foundation/meta"],
            "forbidden_patterns": [],
            "template_outputs": {
                implementation_scope: "// changed by product implementation\n",
            },
            "validation_commands": list(entry["implementation_writer"]["validation_commands"]),
            "evidence_required": [
                {
                    "kind": "runtime_test",
                    "name": "runtime test",
                    "paths": [evidence_output],
                    "validation_command_ids": ["uv:pytest"],
                }
            ],
            "closeout_contract": {
                "path": entry["expected_closeout_path"],
                "completion_quality": "runtime_proven",
                "evidence_required": [
                    {
                        "kind": "runtime_test",
                        "name": "runtime test",
                        "paths": [evidence_output],
                        "validation_command_ids": ["uv:pytest"],
                    }
                ],
            },
            "rollback_policy": "reject import and leave repository unchanged on validation, scope, or digest failure",
            "stop_conditions": list(entry["implementation_writer"]["stop_conditions"]),
        },
    )
    write_yaml(production_path, production)
    write_yaml(roadmap_path, roadmap)
    write_yaml(manifest_root / "pt-test.yaml", manifest)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
    return production_path, roadmap_path, manifest_root, plan_path, implementation_path, closeout_path

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
        contract_pack_root=pack_root,
    )
    pack_path = write_contract_pack(pack, root=pack_root)
    lock = build_execution_lock(
        track_id,
        locked_by="test",
        contract_pack_root=pack_root,
        granted_permissions=allow or sorted(FULL_TRACK_PERMISSION_SET),
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
    if not (contract_pack_root / "pt-test.yaml").exists():
        compile_result = CliRunner().invoke(
            production_track_cli_app,
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
                "--contract-pack-root",
                str(contract_pack_root),
            ],
        )
        if compile_result.exit_code != 0:
            return compile_result
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
        production_track_cli_app,
        args,
    )

def write_test_pack(
    tmp_path: Path,
    pack_root: Path,
    *,
    action: ActionContract | None = None,
    track_id: str = "PT-TEST",
) -> ContractPack:
    source = tmp_path / "source.yaml"
    source.write_text("version: 1\n", encoding="utf-8")
    pack = ContractPack(
        track_id=track_id,
        generated_at="2026-01-01T00:00:00Z",
        source_digests={str(source): sha256(source.read_bytes()).hexdigest()},
        actions=[action or execution_test_action()],
    )
    write_contract_pack(pack, root=pack_root)
    return pack

def execution_test_action(
    *,
    writer_strategy: str = "agent_writer",
    allowed_outputs: list[str] | None = None,
    new_outputs: list[str] | None = None,
    forbidden_outputs: list[str] | None = None,
    forbidden_patterns: list[str] | None = None,
) -> ActionContract:
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
    resolved_new_outputs = list(new_outputs) if new_outputs is not None else [evidence_output]
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
        new_outputs=resolved_new_outputs,
        forbidden_outputs=forbidden_outputs or ["foundation/meta"],
        forbidden_patterns=forbidden_patterns or [],
        writer_strategy=writer_strategy,
        validation_commands=["python3 --version"],
        evidence_required=[
            EvidenceRequirement(
                kind="runtime_test",
                name="runtime test",
                paths=[evidence_output],
                validation_command_ids=["python3:version"],
            )
        ],
        closeout_contract=CloseoutContract(
            path="docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            completion_quality="runtime_proven",
            evidence_required=[
                EvidenceRequirement(
                    kind="runtime_test",
                    name="runtime test",
                    paths=[evidence_output],
                    validation_command_ids=["python3:version"],
                )
            ],
        ),
        rollback_policy=RollbackPolicy(policy="reject import on scope, digest, or validation failure"),
        stop_conditions=["stop after one implementation action"],
    )

class FakeExecutionAgent:
    def __init__(self, edits: dict[str, str]) -> None:
        self.edits = edits

    def run(self, *, workspace: Path, prompt: str, transcript_dir: Path | None = None) -> AgentResult:
        for relative, content in self.edits.items():
            target = workspace / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        transcript_paths: tuple[Path, ...] = ()
        if transcript_dir is not None:
            transcript_dir.mkdir(parents=True, exist_ok=True)
            prompt_path = transcript_dir / "prompt.md"
            stdout_path = transcript_dir / "stdout.log"
            summary_path = transcript_dir / "summary.yaml"
            prompt_path.write_text(prompt, encoding="utf-8")
            stdout_path.write_text("ok\n", encoding="utf-8")
            summary_path.write_text("returncode: 0\n", encoding="utf-8")
            transcript_paths = (prompt_path, stdout_path, summary_path)
        return AgentResult(returncode=0, stdout="ok", stderr="", transcript_paths=transcript_paths)

class FailingExecutionAgent:
    def run(self, *, workspace: Path, prompt: str, transcript_dir: Path | None = None) -> AgentResult:
        transcript_paths: tuple[Path, ...] = ()
        if transcript_dir is not None:
            transcript_dir.mkdir(parents=True, exist_ok=True)
            prompt_path = transcript_dir / "prompt.md"
            stderr_path = transcript_dir / "stderr.log"
            summary_path = transcript_dir / "summary.yaml"
            prompt_path.write_text(prompt, encoding="utf-8")
            stderr_path.write_text("failed\n", encoding="utf-8")
            summary_path.write_text("returncode: 42\n", encoding="utf-8")
            transcript_paths = (prompt_path, stderr_path, summary_path)
        return AgentResult(returncode=42, stdout="", stderr="failed", transcript_paths=transcript_paths)

def cargo_lock_test_action(*, new_cargo_output: bool = False, crate_creation: bool = False) -> ActionContract:
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
    cargo_output = "domain/ui/ui_new/Cargo.toml" if new_cargo_output else "Cargo.toml"
    action = execution_test_action(
        allowed_outputs=["src/lib.rs"] if new_cargo_output else ["src/lib.rs", cargo_output],
        new_outputs=[cargo_output, evidence_output] if new_cargo_output else [evidence_output],
    )
    action.validation_commands = [
        ValidationCommand(
            command_id="cargo:test",
            argv=["cargo", "test", "-p", "pt_test", "contract"],
            allowed_outputs=["Cargo.lock"],
            raw="cargo test -p pt_test contract",
        )
    ]
    runtime_evidence = EvidenceRequirement(
        kind="runtime_test",
        name="runtime test",
        paths=[evidence_output],
        validation_command_ids=["cargo:test"],
    )
    action.evidence_required = [runtime_evidence]
    action.closeout_contract = CloseoutContract(
        path="docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
        completion_quality="runtime_proven",
        evidence_required=[runtime_evidence],
    )
    if crate_creation and "crate_creation" not in action.permissions_required:
        action.permissions_required.append("crate_creation")
    return action

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
) -> SimpleNamespace:
    pack_root = production_path.parent / "contract-packs"
    try:
        pack = compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
            contract_pack_root=pack_root,
        )
        write_contract_pack(pack, root=pack_root)
        first = run_next_action(
            pack,
            lock_validated=True,
            repo_root=production_path.parent,
            run_validations=False,
            contract_pack_root=pack_root,
        )
        pack = compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
            contract_pack_root=pack_root,
        )
        write_contract_pack(pack, root=pack_root)
        second = run_next_action(
            pack,
            lock_validated=True,
            repo_root=production_path.parent,
            run_validations=False,
            contract_pack_root=pack_root,
        )
        pack = compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
            contract_pack_root=pack_root,
        )
        errors = preflight_pack(pack)
        if errors:
            raise WorkflowError("\n".join(errors))
    except WorkflowError as error:
        return SimpleNamespace(exit_code=1, output=str(error), stdout=str(error))
    output = "\n".join([first.next_action, second.next_action])
    return SimpleNamespace(exit_code=0, output=output, stdout=output)

