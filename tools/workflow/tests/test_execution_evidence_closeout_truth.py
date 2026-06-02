from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *
from execution.closeouts import archive_wr_item


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

def test_execution_preflight_allows_closeout_to_read_existing_evidence_without_output_authority() -> None:
    evidence_output = (
        "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
    )
    action = ActionContract(
        action_id="PT-TEST:PM-TEST-002:WR-002",
        track_id="PT-TEST",
        milestone_id="PM-TEST-002",
        wr_id="WR-002",
        execution_kind="implementation_proof",
        executor_kind="runtime_closeout",
        authority_level="runtime_closeout",
        permissions_required=["agent_closeout"],
        allowed_outputs=[
            "docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            "docs-site/src/content/docs/workspace/production-tracks.yaml",
        ],
        new_outputs=[],
        forbidden_outputs=["foundation/meta"],
        writer_strategy="template_writer",
        validation_commands=["task production:validate"],
        evidence_required=[],
        closeout_contract=CloseoutContract(
            path="docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            completion_quality="runtime_proven",
            evidence_required=[
                EvidenceRequirement(
                    kind="runtime_test",
                    name="runtime test",
                    paths=[evidence_output],
                    validation_command_ids=["cargo:test"],
                )
            ],
        ),
        rollback_policy=RollbackPolicy(policy="reject import on scope, digest, or validation failure"),
        stop_conditions=["stop after closeout"],
    )
    pack = ContractPack(
        track_id="PT-TEST",
        generated_at="2026-01-01T00:00:00Z",
        source_digests={"source.yaml": "abc"},
        actions=[action],
    )

    errors = preflight_pack(pack)

    assert not any("evidence output is not declared in action outputs" in error for error in errors)
    assert not any("runtime_test evidence references validation command id cargo:test" in error for error in errors)

def test_execution_runner_writes_machine_readable_evidence(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    (tmp_path / "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002").mkdir(parents=True)
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
        run_id="test-run",
    )

    assert source.read_text(encoding="utf-8") == "// generated\n"
    assert len(result.evidence_paths) == 1
    evidence = load_yaml(result.evidence_paths[0])
    assert evidence["track_id"] == "PT-TEST"
    assert evidence["milestone_id"] == "PM-TEST-002"
    assert evidence["evidence_kind"] == "runtime_test"
    assert evidence["status"] == "passed"

def test_perfectionist_runtime_closeout_clears_wr_known_quality_gaps() -> None:
    active_data = {
        "version": 1,
        "roadmap": {"title": "Test"},
        "items": [
            {
                "id": "WR-001",
                "write_scopes": [],
                "known_quality_gaps": ["pre-closeout gap"],
            }
        ],
    }
    deferred_data = {"version": 1, "roadmap": {"title": "Deferred"}, "items": []}
    archive_data = {"version": 1, "roadmap": {"title": "Archive"}, "items": []}

    archive_wr_item(
        active_data=active_data,
        deferred_data=deferred_data,
        archive_data=archive_data,
        wr_id="WR-001",
        completion_quality="perfectionist_verified",
        closeout_path="docs-site/src/content/docs/reports/closeouts/pm-test/closeout.md",
    )

    archived = archive_data["items"][0]
    assert archived["completion_quality"] == "perfectionist_verified"
    assert archived["known_quality_gaps"] == []

def test_closeout_claim_rejects_non_runtime_evidence_without_subject_paths(tmp_path: Path) -> None:
    evidence_root = tmp_path / "docs-site/src/content/docs/reports/execution-evidence"
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/artifact-contract.yaml"
    subject = tmp_path / "src" / "lib.rs"
    subject.parent.mkdir(parents=True)
    subject.write_text("// proof subject\n", encoding="utf-8")
    action = ActionContract(
        action_id="PT-TEST:PM-TEST-002:WR-002",
        track_id="PT-TEST",
        milestone_id="PM-TEST-002",
        wr_id="WR-002",
        execution_kind="implementation_proof",
        executor_kind="runtime_closeout",
        authority_level="implementation_runtime_proof",
        permissions_required=["agent_closeout"],
        allowed_outputs=[],
        new_outputs=[],
        forbidden_outputs=[],
        writer_strategy="verification_writer",
        validation_commands=["python3 --version"],
        evidence_required=[],
        closeout_contract=CloseoutContract(
            path="docs-site/src/content/docs/reports/closeouts/pm-test-002/closeout.md",
            completion_quality="runtime_proven",
            evidence_required=[
                EvidenceRequirement(
                    kind="artifact",
                    name="contract",
                    paths=[evidence_output],
                    subject_paths=["src/lib.rs"],
                )
            ],
        ),
        rollback_policy=RollbackPolicy(policy="reject import on scope, digest, or validation failure"),
        stop_conditions=["stop after closeout"],
    )
    write_evidence_record(
        passed_record(
            track_id="PT-TEST",
            milestone_id="PM-TEST-002",
            action_id=action.action_id,
            evidence_kind="artifact",
            name="contract",
            paths=[evidence_output],
            validation_commands=["python3:version"],
        ),
        root=evidence_root,
    )

    errors = closeout_claim_errors(action, evidence_root=evidence_root)

    assert any("has no subject_paths" in error for error in errors)

    write_evidence_record(
        passed_record(
            track_id="PT-TEST",
            milestone_id="PM-TEST-002",
            action_id=action.action_id,
            evidence_kind="artifact",
            name="contract",
            paths=[evidence_output],
            subject_paths=["src/lib.rs"],
            validation_commands=["python3:version"],
        ),
        root=evidence_root,
    )

    assert closeout_claim_errors(action, evidence_root=evidence_root) == []

def test_execution_proof_aggregation_requires_prior_machine_evidence(tmp_path: Path) -> None:
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-003/runtime_test-runtime-test.yaml"
    (tmp_path / evidence_output).parent.mkdir(parents=True)
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
        new_outputs=[evidence_output],
        forbidden_outputs=["product/src/lib.rs"],
        writer_strategy="proof_aggregation_writer",
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
            path="docs-site/src/content/docs/reports/closeouts/pm-test-003/closeout.md",
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

    result = run_action(action, lock_validated=True, repo_root=tmp_path, evidence_root=tmp_path / "evidence", run_id="test-run")

    assert result.written_paths == (tmp_path / evidence_output,)
    assert len(result.evidence_paths) == 1

def test_current_pm012_proof_aggregation_writer_contract_validates() -> None:
    planning = load_production_tracks()
    track = next(candidate for candidate in planning.tracks if candidate.id == "PT-UI-PROGRAM")
    roadmap = load_roadmap()
    loaded = load_track_execution_manifest("PT-UI-PROGRAM")
    assert loaded is not None

    errors = audit_manifest(loaded, track=track, roadmap=roadmap)

    assert errors == []

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

def test_non_runtime_evidence_requires_exact_subject_paths(tmp_path: Path) -> None:
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/artifact-contract.yaml"

    with pytest.raises(ValueError, match="artifact evidence requires subject_paths"):
        EvidenceRequirement(
            kind="artifact",
            name="contract",
            paths=[evidence_output],
        )

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
