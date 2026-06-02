from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *
from truth.certificates import digest_path


def test_track_control_go_delegates_to_execution_kernel(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pack_root = tmp_path / "packs"
    lock_root = tmp_path / "locks"
    action = execution_test_action()
    write_test_pack(tmp_path, pack_root, action=action)
    calls: list[dict] = []

    def fake_run_command(**kwargs) -> None:
        calls.append(kwargs)
        empty_pack = ContractPack(
            track_id="PT-TEST",
            generated_at="2026-01-01T00:00:00Z",
            source_digests=load_contract_pack("PT-TEST", root=pack_root).source_digests,
            actions=[],
        )
        write_contract_pack(empty_pack, root=pack_root)

    monkeypatch.setattr(track_control_cli, "execution_run_command", fake_run_command)
    monkeypatch.setattr(track_control_cli, "compile_or_refresh_pack", lambda *_args, **_kwargs: load_contract_pack("PT-TEST", root=pack_root))
    monkeypatch.setattr(track_control_cli, "post_completion_errors", lambda *_args, **_kwargs: [])
    monkeypatch.setattr(track_control_cli, "repo_visible_completion_drift", lambda **_kwargs: [])

    result = CliRunner().invoke(
        track_control_app,
        [
            "go",
            "--track",
            "PT-TEST",
            "--contract-pack-root",
            str(pack_root),
            "--lock-root",
            str(lock_root),
            "--intent-lock-root",
            str(tmp_path / "intent"),
            "--production-source",
            str(tmp_path / "production.yaml"),
            "--roadmap-source",
            str(tmp_path / "roadmap.yaml"),
            "--manifest-source-root",
            str(tmp_path / "manifests"),
        ],
    )

    assert result.exit_code == 0
    assert calls
    assert calls[0]["mode"] == "full-track"
    assert set(calls[0]["allow"]) == {"agent_closeout", "product_code", "product_implementation"}


def test_track_control_status_blocks_on_latest_current_action_failure(tmp_path: Path) -> None:
    pack_root = tmp_path / "packs"
    ledger_root = tmp_path / "ledgers"
    action = execution_test_action()
    pack = write_test_pack(tmp_path, pack_root, action=action)
    ledger_path = ledger_root / "pt-test" / "pt-test-run.yaml"
    ledger_path.parent.mkdir(parents=True)
    ledger_path.write_text(
        yaml.safe_dump(
            {
                "version": 1,
                "track_id": "PT-TEST",
                "run_id": "pt-test-run",
                "actions": [
                    {
                        "status": "failed",
                        "action_id": action.action_id,
                        "executor_kind": action.executor_kind,
                        "writer_strategy": action.writer_strategy,
                        "post_action_digests": pack.source_digests,
                        "error": f"{action.action_id}: validation command changed undeclared file Cargo.lock",
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )

    payload = track_control_cli.status_payload(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=tmp_path / "locks",
        intent_lock_root=tmp_path / "intent",
        manifest_source_root=tmp_path / "manifests",
        run_ledger_root=ledger_root,
    )

    assert payload["verdict"] == "blocked"
    assert "validation_commands[].allowed_outputs" in "\n".join(payload["blockers"])
    assert payload["next_command"] == "task track -- --track PT-TEST"


def test_track_control_status_reports_complete_uncheckpointed_for_visible_drift(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pack_root = tmp_path / "packs"
    write_contract_pack(
            ContractPack(
                track_id="PT-TEST",
                generated_at="2026-01-01T00:00:00Z",
                source_digests={"Taskfile.yml": digest_path(REPO_ROOT / "Taskfile.yml")},
                actions=[],
            ),
        root=pack_root,
    )
    monkeypatch.setattr(track_control_cli, "repo_visible_completion_drift", lambda **_kwargs: [" M docs/source.yaml"])

    payload = track_control_cli.status_payload(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=tmp_path / "locks",
        intent_lock_root=tmp_path / "intent",
        manifest_source_root=tmp_path / "manifests",
        run_ledger_root=tmp_path / "ledgers",
    )

    assert payload["verdict"] == "complete_uncheckpointed"
    assert "repository-visible files are not checkpointed" in "\n".join(payload["blockers"])


def test_track_control_status_reports_truth_blocked_for_strong_claim_repair_action(tmp_path: Path) -> None:
    pack_root = tmp_path / "packs"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    action = execution_test_action()
    write_test_pack(tmp_path, pack_root, action=action)
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "perfectionist_verified",
            "claim_status": "blocked",
            "claim_statement": "Architecture truth needs behavior probes.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Semantic proof is incomplete."],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    payload = track_control_cli.status_payload(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=tmp_path / "locks",
        intent_lock_root=tmp_path / "intent",
        manifest_source_root=manifest_root,
        run_ledger_root=tmp_path / "ledgers",
    )

    assert payload["verdict"] == "truth_blocked"
    assert "architecture-test" in "\n".join(payload["truth_findings"])
    assert payload["next_command"] == "task track:go -- --track PT-TEST"


def test_track_control_status_does_not_block_completion_on_downstream_gate_claims(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pack_root = tmp_path / "packs"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    source = tmp_path / "source.yaml"
    source.write_text("version: 1\n", encoding="utf-8")
    write_contract_pack(
        ContractPack(
            track_id="PT-TEST",
            generated_at="2026-01-01T00:00:00Z",
            source_digests={source.as_posix(): digest_path(source)},
            actions=[],
        ),
        root=pack_root,
    )
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "materialprogram-readiness-gate",
            "claim_kind": "handoff",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Downstream MaterialProgram planning remains blocked.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Downstream handoff is intentionally blocked."],
            "supersedes": [],
            "blocks_downstream": ["MaterialProgram implementation"],
        },
        {
            "claim_id": "foundation-meta-extraction-gate",
            "claim_kind": "extraction_gate",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Foundation/meta extraction remains blocked.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Extraction is intentionally blocked."],
            "supersedes": [],
            "blocks_downstream": ["foundation/meta extraction"],
        },
    ]
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    monkeypatch.setattr(track_control_cli, "repo_visible_completion_drift", lambda **_kwargs: [])

    payload = track_control_cli.status_payload(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=tmp_path / "locks",
        intent_lock_root=tmp_path / "intent",
        manifest_source_root=manifest_root,
        run_ledger_root=tmp_path / "ledgers",
    )

    assert payload["verdict"] == "complete"
    assert "materialprogram-readiness-gate" in "\n".join(payload["truth_claims"])
    assert payload["truth_findings"] == []


def test_track_control_status_ignores_repaired_production_validation_failure(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pack_root = tmp_path / "packs"
    ledger_root = tmp_path / "ledgers"
    lock_root = tmp_path / "locks"
    action = execution_test_action()
    pack = write_test_pack(tmp_path, pack_root, action=action)
    ledger_path = ledger_root / "pt-test" / "pt-test-run.yaml"
    ledger_path.parent.mkdir(parents=True)
    ledger_path.write_text(
        yaml.safe_dump(
            {
                "version": 1,
                "track_id": "PT-TEST",
                "run_id": "pt-test-run",
                "actions": [
                    {
                        "status": "failed",
                        "action_id": action.action_id,
                        "executor_kind": action.executor_kind,
                        "writer_strategy": action.writer_strategy,
                        "post_action_digests": pack.source_digests,
                        "error": "validation failed: task production:validate -> exit 201",
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )
    lock = build_execution_lock(
        "PT-TEST",
        locked_by="test",
        contract_pack_root=pack_root,
        granted_permissions=sorted(track_control_cli.required_permissions(pack)),
        denied_permissions=["foundation_extraction"],
    )
    write_execution_lock(lock, root=lock_root)
    monkeypatch.setattr(track_control_cli, "production_validation_errors", lambda **_: [])

    payload = track_control_cli.status_payload(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=lock_root,
        intent_lock_root=tmp_path / "intent",
        manifest_source_root=tmp_path / "manifests",
        run_ledger_root=ledger_root,
    )

    assert payload["verdict"] == "ready"
    assert payload["blockers"] == []


def test_track_control_status_ignores_repaired_workflow_test_failure(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pack_root = tmp_path / "packs"
    ledger_root = tmp_path / "ledgers"
    lock_root = tmp_path / "locks"
    action = execution_test_action()
    pack = write_test_pack(tmp_path, pack_root, action=action)
    ledger_path = ledger_root / "pt-test" / "pt-test-run.yaml"
    ledger_path.parent.mkdir(parents=True)
    ledger_path.write_text(
        yaml.safe_dump(
            {
                "version": 1,
                "track_id": "PT-TEST",
                "run_id": "pt-test-run",
                "actions": [
                    {
                        "status": "failed",
                        "action_id": action.action_id,
                        "executor_kind": action.executor_kind,
                        "writer_strategy": action.writer_strategy,
                        "post_action_digests": pack.source_digests,
                        "error": "validation failed: task workflow:test -> exit 201",
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )
    lock = build_execution_lock(
        "PT-TEST",
        locked_by="test",
        contract_pack_root=pack_root,
        granted_permissions=sorted(track_control_cli.required_permissions(pack)),
        denied_permissions=["foundation_extraction"],
    )
    write_execution_lock(lock, root=lock_root)
    monkeypatch.setattr(track_control_cli, "readonly_validation_command_fails", lambda _argv: False)

    payload = track_control_cli.status_payload(
        "PT-TEST",
        contract_pack_root=pack_root,
        lock_root=lock_root,
        intent_lock_root=tmp_path / "intent",
        manifest_source_root=tmp_path / "manifests",
        run_ledger_root=ledger_root,
    )

    assert payload["verdict"] == "ready"
    assert payload["blockers"] == []
