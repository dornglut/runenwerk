from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *
from execution.compiler import evidence_requirements_satisfied
from truth.certificates import digest_path


def test_execution_planning_expansion_creates_and_links_deferred_wr(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    pack_root = tmp_path / "contract-packs"
    manifest_root.mkdir()
    production_data, roadmap_data, manifest_data = valid_track_expansion_state()
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, roadmap_data)
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
        contract_pack_root=pack_root,
    )
    write_contract_pack(pack, root=pack_root)

    result = run_next_action(
        pack,
        lock_validated=True,
        repo_root=tmp_path,
        run_validations=False,
        contract_pack_root=pack_root,
    )

    assert result.action_id == "PT-TEST:PM-TEST-002:WR-TBD-TEST-002"
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
    assert all(path.is_relative_to(tmp_path) for path in result.written_paths)

def test_execution_contract_compiler_compiles_missing_plan_to_design_authoring(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = write_product_code_fixture(
        tmp_path,
        monkeypatch,
        write_plan=False,
    )
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    assert pack.actions
    action = pack.actions[0]
    assert action.executor_kind == "design_authoring"
    assert action.writer_strategy == "template_writer"
    assert action.permissions_required == ["agent_design"]
    assert plan_path.relative_to(tmp_path).as_posix() in action.new_outputs
    assert (plan_path.parent / "plan.contract.yaml").relative_to(tmp_path).as_posix() in action.new_outputs
    assert "product_code" not in action.permissions_required
    assert "product_implementation" not in action.permissions_required

def test_execution_contract_compiler_requires_plan_sidecar(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    (plan_path.parent / "plan.contract.yaml").unlink()
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    with pytest.raises(WorkflowError, match="structured implementation-plan authority is missing"):
        compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
        )

def test_execution_compiler_rejects_stale_resolver_evidence_before_closeout(tmp_path: Path) -> None:
    subject = tmp_path / "subject.txt"
    subject.write_text("current\n", encoding="utf-8")
    ledger = tmp_path / "ledger.yaml"
    ledger.write_text("version: 1\n", encoding="utf-8")
    evidence = tmp_path / "evidence.yaml"
    evidence.write_text(
        yaml.safe_dump(
            {
                "evidence_kind": "artifact",
                "status": "passed",
                "subject_paths": ["subject.txt"],
                "subject_digests": {"subject.txt": digest_path(subject)},
                "validation_provenance": [
                    {
                        "command_id": "task:workflow:test",
                        "argv": ["task", "workflow:test"],
                        "returncode": 0,
                        "run_ledger_path": "ledger.yaml",
                        "run_action_id": "PT-TEST:PM-TEST-001:WR-001",
                        "validation_result_digest": "digest",
                        "subject_digests": {"subject.txt": digest_path(subject)},
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )
    requirement = EvidenceRequirement(
        kind="artifact",
        name="artifact",
        paths=["evidence.yaml"],
        subject_paths=["subject.txt"],
    )

    assert evidence_requirements_satisfied([requirement], production_source=tmp_path / "production.yaml")

    subject.write_text("changed\n", encoding="utf-8")

    assert not evidence_requirements_satisfied([requirement], production_source=tmp_path / "production.yaml")

def test_execution_contract_compiler_uses_sidecar_executable_authority(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, implementation_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    evidence_output = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-runtime-test.yaml"
    sidecar = load_yaml(plan_path.parent / "plan.contract.yaml")
    sidecar["writer_strategy"] = "patch_writer"
    sidecar["template_outputs"] = {}
    sidecar["patches"] = [
        {
            "file": implementation_path.relative_to(tmp_path).as_posix(),
            "find": "// implementation fixture\n",
            "replace": "// patched by sidecar authority\n",
        }
    ]
    sidecar["allowed_outputs"] = [implementation_path.relative_to(tmp_path).as_posix()]
    sidecar["new_outputs"] = [evidence_output]
    write_yaml(plan_path.parent / "plan.contract.yaml", sidecar)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    action = pack.actions[0]
    assert action.writer_strategy == "patch_writer"
    assert action.template_outputs == {}
    assert len(action.patches) == 1
    assert action.patches[0].replace == "// patched by sidecar authority\n"

def test_execution_contract_compiler_expands_agent_subactions_as_resumable_actions(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, implementation_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    implementation_output = implementation_path.relative_to(tmp_path).as_posix()
    evidence_one = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-contract-a.yaml"
    evidence_two = "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002/runtime_test-contract-b.yaml"
    sidecar = load_yaml(plan_path.parent / "plan.contract.yaml")
    sidecar["writer_strategy"] = "agent_writer"
    sidecar["template_outputs"] = {}
    sidecar["patches"] = []
    sidecar["allowed_outputs"] = [implementation_output]
    sidecar["new_outputs"] = [evidence_one, evidence_two]
    sidecar["validation_commands"] = ["python3 --version"]
    sidecar["evidence_required"] = [
        {
            "kind": "runtime_test",
            "name": "contract a",
            "paths": [evidence_one],
            "validation_command_ids": ["python3:version"],
        },
        {
            "kind": "runtime_test",
            "name": "contract b",
            "paths": [evidence_two],
            "validation_command_ids": ["python3:version"],
        },
    ]
    sidecar["closeout_contract"]["evidence_required"] = sidecar["evidence_required"]
    sidecar["agent_subactions"] = [
        {
            "sub_action_id": "contract-a",
            "title": "Contract A",
            "prompt": "Implement only the first bounded contract output.",
            "allowed_outputs": [implementation_output],
            "new_outputs": [evidence_one],
            "validation_commands": ["python3 --version"],
            "evidence_required": [sidecar["evidence_required"][0]],
            "stop_conditions": ["stop after contract-a"],
        },
        {
            "sub_action_id": "contract-b",
            "title": "Contract B",
            "prompt": "Implement only the second bounded contract output.",
            "allowed_outputs": [implementation_output],
            "new_outputs": [evidence_two],
            "validation_commands": ["python3 --version"],
            "evidence_required": [sidecar["evidence_required"][1]],
            "stop_conditions": ["stop after contract-b"],
        },
    ]
    write_yaml(plan_path.parent / "plan.contract.yaml", sidecar)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    assert [action.agent_subaction.sub_action_id for action in pack.actions] == ["contract-a", "contract-b"]
    assert pack.actions[0].parent_action_id == "PT-TEST:PM-TEST-002:WR-002"
    assert pack.actions[0].action_id.endswith(":sub:contract-a")
    assert evidence_one in pack.actions[0].new_outputs
    assert evidence_two not in pack.actions[0].new_outputs

    first_evidence = tmp_path / evidence_one
    first_evidence.parent.mkdir(parents=True, exist_ok=True)
    (tmp_path / "ledger.yaml").write_text("version: 1\n", encoding="utf-8")
    first_evidence.write_text(
        yaml.safe_dump(
            {
                "evidence_kind": "runtime_test",
                "status": "passed",
                "validation_provenance": [
                    {
                        "command_id": "python3:version",
                        "argv": ["python3", "--version"],
                        "returncode": 0,
                        "run_ledger_path": "ledger.yaml",
                        "run_action_id": pack.actions[0].action_id,
                        "validation_result_digest": "digest",
                        "subject_digests": {},
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )
    resumed_pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    assert [action.agent_subaction.sub_action_id for action in resumed_pack.actions] == ["contract-b"]

def test_execution_contract_compiler_keeps_draft_sidecar_non_executable(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    sidecar = load_yaml(plan_path.parent / "plan.contract.yaml")
    sidecar["status"] = "draft"
    write_yaml(plan_path.parent / "plan.contract.yaml", sidecar)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    pack = compile_contract_pack(
        "PT-TEST",
        production_source=production_path,
        roadmap_source=roadmap_path,
        manifest_root=manifest_root,
    )

    action = pack.actions[0]
    assert action.executor_kind == "design_authoring"
    assert action.writer_strategy == "verification_writer"
    assert action.permissions_required == ["agent_design"]
    assert "product_code" not in action.permissions_required
    assert "product_implementation" not in action.permissions_required

def test_execution_contract_compiler_rejects_sidecar_broader_than_manifest_permissions(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def mutate(production: dict, roadmap: dict, manifest: dict, _plan: Path, _implementation: Path, _closeout: Path) -> None:
        entry = manifest["milestones"][1]
        entry["may_create_code"] = False
        entry["may_modify_production_behavior"] = False
        entry["permission_classes_required"] = ["agent_design"]
        for item in roadmap["items"]:
            if item["id"] == entry["owning_wr"]:
                item["planning_state"] = "blocked_deferred"
                item["blocker"] = 4

    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
        mutate=mutate,
    )
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    with pytest.raises(WorkflowError) as error:
        compile_contract_pack(
            "PT-TEST",
            production_source=production_path,
            roadmap_source=roadmap_path,
            manifest_root=manifest_root,
        )

    message = str(error.value)
    assert "manifest may_create_code is false" in message
    assert "outside manifest permission_classes_required" in message
    assert "requires owning WR WR-002 to be current_candidate" in message

def test_execution_cli_writes_run_ledger_after_successful_action(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    (tmp_path / "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test-002").mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action(writer_strategy="template_writer")
    action.template_outputs["src/lib.rs"] = "// generated\n"
    action.validation_commands = ["python3 --version"]
    digest_subject = REPO_ROOT / "tools/workflow/test_workflow.py"
    pack_root = tmp_path / "packs"
    lock_root = tmp_path / "locks"
    ledger_root = tmp_path / "ledgers"
    write_contract_pack(
        ContractPack(
            track_id="PT-TEST",
            generated_at="2026-06-01T00:00:00Z",
            source_digests={"tools/workflow/test_workflow.py": sha256(digest_subject.read_bytes()).hexdigest()},
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
    assert repo_path(source) in ledger["actions"][0]["files_changed"]

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

def test_execution_compiler_uses_source_models_instead_of_legacy_runner() -> None:
    compiler_source = (REPO_ROOT / "tools/workflow/execution/compiler.py").read_text(encoding="utf-8")
    cli_source = (REPO_ROOT / "tools/workflow/execution/cli.py").read_text(encoding="utf-8")

    assert "from track_sources.manifest import" in compiler_source
    assert "from track_sources.manifest import" in cli_source
    assert "from track_execution_manifest import" not in compiler_source
    assert "from track_execution_manifest import" not in cli_source

def test_contract_pack_source_digests_include_public_command_adapters() -> None:
    digests = harness_source_digest_map()

    assert "tools/workflow/production_track_cli.py" in digests
    assert "tools/workflow/production_goal.py" in digests
    assert "tools/workflow/production_state.py" in digests
    assert "tools/workflow/roadmap_state.py" in digests

def test_agent_backend_timeout_output_is_normalized_to_text() -> None:
    assert subprocess_output_text(b"codex exec timed out\n") == "codex exec timed out\n"
    assert subprocess_output_text("already text") == "already text"
    assert subprocess_output_text(None) == ""
