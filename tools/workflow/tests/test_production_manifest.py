from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_track_manifest_plan_track_creates_scaffold(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        production_track_cli_app,
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
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
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
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
        ],
    )

    assert result.exit_code == 1
    assert "production:next failed" in result.stdout
    assert "missing Track Execution Manifest" in result.stdout
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
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
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
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
        ],
    )

    assert result.exit_code == 1
    assert "production:next failed" in result.stdout
    assert "validation errors for" in result.stdout
    assert "Field required" in result.stdout
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
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
        ],
    )

    assert result.exit_code == 1
    assert "Track Execution Manifest audit blockers" in result.stdout
    assert "invalid closeout path:" in result.stdout
    assert "expected_closeout_path must be a Markdown closeout/report path" in result.stdout
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
        production_track_cli_app,
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
        production_track_cli_app,
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
    assert "missing Track Execution Manifest" in result.stdout
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
        production_track_cli_app,
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

def test_public_plan_track_scaffolds_missing_manifest_through_source_layer(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, *_ = write_product_code_fixture(tmp_path, monkeypatch)
    manifest_path = manifest_root / "pt-test.yaml"
    manifest_path.unlink()

    result = CliRunner().invoke(
        production_track_cli_app,
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

    assert result.exit_code == 0, result.stdout
    assert "Track Execution Manifest scaffold written" in result.stdout
    scaffold = load_yaml(manifest_path)
    assert scaffold["track_id"] == "PT-TEST"
    assert scaffold["milestones"][0]["milestone_id"] == "PM-TEST-001"
    assert scaffold["truth_claims"][0]["claim_status"] == "blocked"

def test_authority_paths_do_not_import_legacy_manifest_runner() -> None:
    authority_paths = [
        REPO_ROOT / "tools/workflow/production_goal.py",
        REPO_ROOT / "tools/workflow/production_state.py",
        REPO_ROOT / "tools/workflow/production_track_cli.py",
        *sorted((REPO_ROOT / "tools/workflow/execution").glob("*.py")),
        *sorted((REPO_ROOT / "tools/workflow/track_sources").glob("*.py")),
    ]

    offenders = [
        repo_path(path)
        for path in authority_paths
        if "from track_execution_manifest import" in path.read_text(encoding="utf-8")
        or "import track_execution_manifest" in path.read_text(encoding="utf-8")
    ]

    assert offenders == []

def test_public_production_tasks_use_track_cli_adapter() -> None:
    taskfile = (REPO_ROOT / "Taskfile.yml").read_text(encoding="utf-8")

    assert "uv run python tools/workflow/production_track_cli.py run-track" in taskfile
    assert "uv run python tools/workflow/production_track_cli.py next" in taskfile
    assert "uv run python tools/workflow/production_track_cli.py audit-track" in taskfile
    assert "uv run python tools/workflow/track_execution_manifest.py run-track" not in taskfile
    assert "uv run python tools/workflow/track_execution_manifest.py next" not in taskfile
    assert "uv run python tools/workflow/track_execution_manifest.py audit-track" not in taskfile

def test_complete_track_contracts_uses_plan_sidecar_without_filling_manifest_contracts(
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
            str(tmp_path / "contract-packs"),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Execution Contract Pack written." in result.stdout
    manifest = load_yaml(manifest_root / "pt-test.yaml")
    milestone = manifest["milestones"][1]
    assert "auto_safe_contract" not in milestone
    assert "agent_design_contract" not in milestone
    assert "product_code_contract" not in milestone
    assert "runtime_closeout_contract" not in milestone

def test_complete_track_contracts_ignores_legacy_template_key_when_sidecar_exists(
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
            str(tmp_path / "contract-packs"),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Execution Contract Pack written." in result.stdout

def test_manifest_backed_production_validation_passes_valid_manifest(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    planning = ProductionPlanningState.model_validate(valid_production_state())

    assert validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root) == []

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

