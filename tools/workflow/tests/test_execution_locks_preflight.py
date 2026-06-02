from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_invalid_blocker_is_rejected() -> None:
    state = valid_state()
    state["items"][0]["blocker"] = 6
    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)

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
        production_track_cli_app,
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
    assert "implementation_writer.strategy must not be no_writer" in result.stdout

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

def test_full_automation_preflight_fails_when_validation_command_is_prose(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(_production: dict, _roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        entry = manifest["milestones"][1]
        entry["validation_commands"] = ["focused tests named by the owning production plan"]
        entry["product_code_contract"]["validation_commands"] = ["focused tests named by the owning production plan"]
        entry["implementation_writer"]["validation_commands"] = ["focused tests named by the owning production plan"]

    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 1
    assert "prose/non-executable" in result.stdout

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
        production_track_cli_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--mode",
            "agent-track",
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
            "--run-ledger-root",
            str(tmp_path / "runs"),
        ],
    )

    assert result.exit_code == 1
    assert "Track Execution Manifest full-automation blockers" in result.stdout
    assert load_yaml(production_path)["tracks"][0]["milestones"][1].get("roadmap_links", []) == []
    assert not (tmp_path / "roadmap-deferred.yaml").exists()

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
    assert "full automation must not require foundation_extraction" in result.stdout

def test_full_automation_preflight_passes_for_fully_specified_contract(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)

    result = invoke_full_automation_audit(production_path, roadmap_path, manifest_root)

    assert result.exit_code == 0, result.output
    assert "execution preflight passed" in result.stdout

def test_public_full_automation_audit_is_read_only_without_contract_pack(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    pack_path = production_path.parent / "contract-packs" / "pt-test.yaml"
    before = {
        "production": sha256(production_path.read_bytes()).hexdigest(),
        "roadmap": sha256(roadmap_path.read_bytes()).hexdigest(),
        "manifest": sha256((manifest_root / "pt-test.yaml").read_bytes()).hexdigest(),
    }

    result = CliRunner().invoke(
        production_track_cli_app,
        [
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
            str(production_path.parent / "contract-packs"),
        ],
    )

    assert result.exit_code == 1
    assert "Execution Contract Pack" in result.stdout
    assert "execution:compile explicitly" in result.stdout
    assert not pack_path.exists()
    assert before == {
        "production": sha256(production_path.read_bytes()).hexdigest(),
        "roadmap": sha256(roadmap_path.read_bytes()).hexdigest(),
        "manifest": sha256((manifest_root / "pt-test.yaml").read_bytes()).hexdigest(),
    }

def test_public_complete_track_contracts_compiles_contract_pack(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(tmp_path, monkeypatch)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)
    pack_root = production_path.parent / "contract-packs"

    result = CliRunner().invoke(
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
            str(pack_root),
        ],
    )

    assert result.exit_code == 0, result.stdout
    assert (pack_root / "pt-test.yaml").exists()
    assert "Execution Contract Pack written" in result.stdout

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

def test_public_agent_track_mode_prepares_lock_and_runs_one_clean_kernel_action(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, _plan_path, implementation_path, _closeout = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    lock_root = tmp_path / "locks"
    pack_root = tmp_path / "contract-packs"
    ledger_root = tmp_path / "runs"
    evidence_root = tmp_path / "docs-site/src/content/docs/reports/execution-evidence"

    result = CliRunner().invoke(
        production_track_cli_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--mode",
            "agent-track",
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
            "--contract-pack-root",
            str(pack_root),
            "--run-ledger-root",
            str(ledger_root),
            "--evidence-root",
            str(evidence_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Execution Contract Pack prepared." in result.stdout
    assert "Execution Lock written." in result.stdout
    assert "Execution Harness ran one ActionContract." in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// changed by product implementation\n"
    assert (lock_root / "pt-test.yaml").exists()

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

def test_execution_preflight_rejects_directory_new_outputs() -> None:
    action = execution_test_action(new_outputs=["domain/ui/ui_controls"])
    pack = ContractPack(
        track_id="PT-TEST",
        generated_at="2026-01-01T00:00:00Z",
        source_digests={"source.yaml": "abc"},
        actions=[action],
    )

    errors = preflight_pack(pack)

    assert any("must name an exact file, not a directory scope" in error for error in errors)

def test_execution_preflight_rejects_unresolved_runtime_test_command_id() -> None:
    action = ActionContract(
        action_id="PT-TEST:PM-TEST-002:WR-002",
        track_id="PT-TEST",
        milestone_id="PM-TEST-002",
        wr_id="WR-002",
        execution_kind="implementation_proof",
        executor_kind="product_implementation",
        authority_level="implementation_runtime_proof",
        permissions_required=["product_code", "product_implementation"],
        allowed_outputs=["src/lib.rs"],
        new_outputs=[
            "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
        ],
        forbidden_outputs=["foundation/meta"],
        writer_strategy="agent_writer",
        validation_commands=["uv run pytest tools/workflow/test_workflow.py"],
        evidence_required=[
            EvidenceRequirement(
                kind="runtime_test",
                name="runtime test",
                paths=[
                    "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
                ],
                validation_command_ids=["missing:runtime_test_validation"],
            )
        ],
        closeout_contract=CloseoutContract(
            path="docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            completion_quality="runtime_proven",
            evidence_required=[
                EvidenceRequirement(
                    kind="runtime_test",
                    name="runtime test",
                    paths=[
                        "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
                    ],
                    validation_command_ids=["missing:runtime_test_validation"],
                )
            ],
        ),
        rollback_policy=RollbackPolicy(policy="reject import on scope, digest, or validation failure"),
        stop_conditions=["stop after one implementation action"],
    )
    pack = ContractPack(
        track_id="PT-TEST",
        generated_at="2026-01-01T00:00:00Z",
        source_digests={"source.yaml": "abc"},
        actions=[action],
    )

    errors = preflight_pack(pack)

    assert any("unresolved validation command id" in error for error in errors)

def test_execution_preflight_rejects_truth_validation_before_evidence_phase() -> None:
    action = execution_test_action()
    action.validation_commands = [
        "python3 --version",
        "task truth:verify -- --track PT-TEST --claim test-claim",
    ]

    errors = preflight_action(action)

    assert any(
        "truth verification/certification belongs after resolver-backed evidence is written" in error
        for error in errors
    )

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

def test_track_control_status_complete_skips_execution_lock_drift(tmp_path: Path) -> None:
    pack_root = tmp_path / "packs"
    lock_root = tmp_path / "locks"
    source = tmp_path / "source.yaml"
    source.write_text("version: 1\n", encoding="utf-8")
    pack = ContractPack(
        track_id="PT-TEST",
        generated_at="2026-01-01T00:00:00Z",
        source_digests={str(source): sha256(source.read_bytes()).hexdigest()},
        actions=[],
    )
    write_contract_pack(pack, root=pack_root)
    lock = build_execution_lock(
        "PT-TEST",
        locked_by="test",
        contract_pack_root=pack_root,
        granted_permissions=[],
        denied_permissions=["foundation_extraction"],
    )
    write_execution_lock(lock, root=lock_root)
    lock_path = lock_root / "pt-test.yaml"
    lock_data = yaml.safe_load(lock_path.read_text(encoding="utf-8"))
    lock_data["contract_pack_digest"] = "stale"
    lock_path.write_text(yaml.safe_dump(lock_data, sort_keys=False), encoding="utf-8")

    result = CliRunner().invoke(
        track_control_app,
        [
            "status",
            "--track",
            "PT-TEST",
            "--contract-pack-root",
            str(pack_root),
            "--lock-root",
            str(lock_root),
            "--intent-lock-root",
            str(tmp_path / "intent"),
            "--manifest-source-root",
            str(tmp_path / "manifests"),
        ],
    )

    assert result.exit_code == 0
    assert "Verdict: complete" in result.stdout

def test_track_control_allow_requires_reason_and_writes_intent_lock(tmp_path: Path) -> None:
    result_without_reason = CliRunner().invoke(
        track_control_app,
        ["allow", "--track", "PT-TEST", "--permission", "crate_creation", "--intent-lock-root", str(tmp_path / "intent")],
    )
    assert result_without_reason.exit_code != 0

    result = CliRunner().invoke(
        track_control_app,
        [
            "allow",
            "--track",
            "PT-TEST",
            "--permission",
            "crate_creation",
            "--reason",
            "Exact new Cargo.toml output is declared by the Contract Pack.",
            "--intent-lock-root",
            str(tmp_path / "intent"),
        ],
    )

    assert result.exit_code == 0
    lock_path = tmp_path / "intent" / "pt-test.yaml"
    data = yaml.safe_load(lock_path.read_text(encoding="utf-8"))
    assert "crate_creation" in data["granted_permissions"]
    assert "foundation_extraction" in data["denied_permissions"]

def test_track_control_prepare_refreshes_stale_execution_lock_when_intent_grants_match(tmp_path: Path) -> None:
    pack_root = tmp_path / "packs"
    lock_root = tmp_path / "locks"
    intent_root = tmp_path / "intent"
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
    action = execution_test_action(
        allowed_outputs=["domain/ui/ui_schema/Cargo.toml", "src/lib.rs", evidence_output],
        new_outputs=["domain/ui/ui_schema/Cargo.toml", evidence_output],
    )
    action.permissions_required.append("crate_creation")
    pack = write_test_pack(tmp_path, pack_root, action=action)
    intent = track_control_cli.TrackIntentLock(track_id="PT-TEST", updated_at="2026-01-01T00:00:00Z")
    intent.granted_permissions["crate_creation"] = track_control_cli.PermissionAuthorization(
        permission="crate_creation",
        by="test",
        at="2026-01-01T00:00:00Z",
        reason="Exact crate scope is test-authorized.",
    )
    track_control_cli.ensure_default_denials(intent, by="test")
    track_control_cli.write_intent_lock(intent, root=intent_root)
    write_execution_lock(
        build_execution_lock(
            "PT-TEST",
            locked_by="test",
            contract_pack_root=pack_root,
            granted_permissions=["product_code", "product_implementation", "crate_creation"],
            denied_permissions=["foundation_extraction"],
        ),
        root=lock_root,
    )
    source = tmp_path / "source.yaml"
    source.write_text("version: 2\n", encoding="utf-8")
    refreshed_pack = ContractPack(
        track_id="PT-TEST",
        generated_at="2026-01-01T00:00:00Z",
        source_digests={str(source): sha256(source.read_bytes()).hexdigest()},
        actions=[action],
    )
    write_contract_pack(refreshed_pack, root=pack_root)

    payload = track_control_cli.prepare_execution(
        "PT-TEST",
        production_source=tmp_path / "production.yaml",
        roadmap_source=tmp_path / "roadmap.yaml",
        manifest_source_root=tmp_path / "manifests",
        contract_pack_root=pack_root,
        lock_root=lock_root,
        intent_lock_root=intent_root,
        locked_by="test",
    )

    assert payload["verdict"] == "ready"
    assert payload["lock_refreshed"] is True
    assert execution_lock_errors(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=lock_root,
        requested_permissions=set(payload["required_permissions"]),
    ) == []

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
