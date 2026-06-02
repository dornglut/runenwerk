from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


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

def test_agent_track_creates_missing_wr_and_plan_without_product_code(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, _scope_paths = write_implementation_agent_design_fixture(
        tmp_path,
        monkeypatch,
    )
    (tmp_path / "Taskfile.yml").write_text(
        'version: "3"\n\ntasks:\n'
        '  docs:validate:\n    cmds:\n      - "true"\n'
        '  planning:validate:\n    cmds:\n      - "true"\n'
        '  production:validate:\n    cmds:\n      - "true"\n'
        '  roadmap:validate:\n    cmds:\n      - "true"\n',
        encoding="utf-8",
    )
    lock_root = tmp_path / "locks"
    pack_root = tmp_path / "contract-packs"

    result = CliRunner().invoke(
        production_track_cli_app,
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
            "--contract-pack-root",
            str(pack_root),
            "--run-ledger-root",
            str(tmp_path / "runs"),
        ],
    )

    assert result.exit_code == 1
    assert "completed but did not advance" in result.stdout
    assert "Contract Pack state" in result.stdout
    assert plan_path.exists()
    assert (lock_root / "pt-test.yaml").exists()

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
    assert "validation command is prose/non-executable" in result.stdout
    assert "cannot generate plan.contract.yaml with invalid validation command" in result.stdout

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

