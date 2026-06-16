from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


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
    pack_root = tmp_path / "contract-packs"
    ledger_root = tmp_path / "runs"
    evidence_root = tmp_path / "docs-site/src/content/docs/reports/execution-evidence"

    from execution.writers import AgentResult, CodexExecBackend

    def fake_agent_run(self, *, workspace: Path, prompt: str, transcript_dir: Path | None = None) -> AgentResult:
        target = workspace / implementation_path.relative_to(tmp_path)
        target.write_text("// changed by agent writer\n", encoding="utf-8")
        transcript_paths: tuple[Path, ...] = ()
        if transcript_dir is not None:
            transcript_dir.mkdir(parents=True, exist_ok=True)
            summary = transcript_dir / "summary.yaml"
            summary.write_text("returncode: 0\n", encoding="utf-8")
            transcript_paths = (summary,)
        return AgentResult(returncode=0, stdout="changed", stderr="", transcript_paths=transcript_paths)

    monkeypatch.setattr(CodexExecBackend, "run", fake_agent_run)

    result = CliRunner().invoke(
        production_track_cli_app,
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
            "--contract-pack-root",
            str(pack_root),
            "--run-ledger-root",
            str(ledger_root),
            "--evidence-root",
            str(evidence_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "Execution Lock written." in result.stdout
    assert "Transcript:" in result.stdout
    assert implementation_path.read_text(encoding="utf-8") == "// changed by agent writer\n"
    assert (lock_root / "pt-test.yaml").exists()

def test_wr169_typed_graph_contract_declares_cargo_lock_as_validation_output_only() -> None:
    sidecar_path = (
        REPO_ROOT
        / "docs-site/src/content/docs/reports/implementation-plans/wr-169-perfectionist-architecture-conformance-closure/plan.contract.yaml"
    )
    sidecar = yaml.safe_load(sidecar_path.read_text(encoding="utf-8"))
    typed_graph = next(
        subaction
        for subaction in sidecar["agent_subactions"]
        if subaction["sub_action_id"] == "typed-graph-control-contracts"
    )

    assert "Cargo.lock" not in typed_graph["allowed_outputs"]
    assert "Cargo.lock" not in typed_graph["new_outputs"]
    cargo_commands = [
        command for command in typed_graph["validation_commands"] if isinstance(command, dict) and command["command_id"].startswith("cargo:")
    ]
    assert cargo_commands
    assert all(command["allowed_outputs"] == ["Cargo.lock"] for command in cargo_commands)
    assert "foundation/meta" in sidecar["forbidden_outputs"]
    assert "domain/material" in sidecar["forbidden_outputs"]
    assert "domain/material_program" in sidecar["forbidden_outputs"]

def test_agent_writer_prompt_includes_quality_doctrine(tmp_path: Path) -> None:
    prompt = action_prompt(execution_test_action(), workspace=tmp_path)

    assert QUALITY_DOCTRINE_ID in prompt

def test_codex_exec_backend_writes_streamed_transcripts(tmp_path: Path) -> None:
    codex_bin = tmp_path / "fake-codex"
    codex_bin.write_text(
        "#!/usr/bin/env python3\n"
        "import sys\n"
        "_prompt = sys.stdin.read()\n"
        "print('stdout line')\n"
        "print('stderr line', file=sys.stderr)\n",
        encoding="utf-8",
    )
    codex_bin.chmod(0o755)
    transcript_dir = tmp_path / "transcript"

    result = CodexExecBackend(codex_bin=str(codex_bin), timeout_seconds=5).run(
        workspace=tmp_path,
        prompt="agent prompt",
        transcript_dir=transcript_dir,
    )

    assert result.returncode == 0
    assert "stdout line" in result.stdout
    assert "stderr line" in result.stderr
    prompt_text = (transcript_dir / "prompt.md").read_text(encoding="utf-8")
    assert prompt_text.startswith("---\n")
    assert "status: completed" in prompt_text.split("\n---\n", 1)[0]
    assert prompt_text.endswith("agent prompt")
    assert "stdout line" in (transcript_dir / "stdout.log").read_text(encoding="utf-8")
    assert "stderr line" in (transcript_dir / "stderr.log").read_text(encoding="utf-8")
    assert "returncode: 0" in (transcript_dir / "summary.yaml").read_text(encoding="utf-8")

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

def test_execution_agent_writer_result_records_transcript_paths(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action()

    result = run_action(
        action,
        backend=FakeExecutionAgent({"src/lib.rs": "// after\n"}),
        lock_validated=True,
        repo_root=tmp_path,
        run_id="run-001",
    )

    assert source.read_text(encoding="utf-8") == "// after\n"
    assert result.transcript_paths
    assert all(path.exists() for path in result.transcript_paths)
    assert any("prompt.md" == path.name for path in result.transcript_paths)

def test_execution_agent_writer_failure_carries_transcript_paths(tmp_path: Path) -> None:
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir(parents=True)
    source.write_text("// before\n", encoding="utf-8")
    action = execution_test_action()

    with pytest.raises(AgentWriterError) as error:
        run_action(
            action,
            backend=FailingExecutionAgent(),
            lock_validated=True,
            repo_root=tmp_path,
            run_id="run-002",
        )

    assert source.read_text(encoding="utf-8") == "// before\n"
    assert error.value.transcript_paths
    assert all(path.exists() for path in error.value.transcript_paths)

def test_snapshot_mirrors_external_cargo_path_workspace(tmp_path: Path) -> None:
    projects = tmp_path / "projects"
    repo_root = projects / "game" / "repo"
    external_root = projects / "spatial_streaming"
    repo_spatial = repo_root / "domain" / "spatial"
    external_spatial = external_root / "crates" / "spatial"
    external_demo = external_root / "demos" / "chunk_streaming_demo"
    grid_root = projects / "grid"
    tile_topology = grid_root / "crates" / "tile_topology"
    repo_spatial.mkdir(parents=True)
    external_spatial.mkdir(parents=True)
    external_demo.mkdir(parents=True)
    tile_topology.mkdir(parents=True)
    (repo_root / "Cargo.toml").write_text(
        "[workspace]\n"
        'members = ["domain/spatial"]\n',
        encoding="utf-8",
    )
    (repo_spatial / "Cargo.toml").write_text(
        "[package]\n"
        'name = "spatial_wrapper"\n'
        'version = "0.1.0"\n'
        'edition = "2024"\n'
        "\n"
        "[dependencies]\n"
        'spatial_core = { package = "spatial", path = "../../../../spatial_streaming/crates/spatial" }\n',
        encoding="utf-8",
    )
    (external_root / "Cargo.toml").write_text(
        "[workspace]\n"
        'members = ["crates/spatial", "demos/chunk_streaming_demo"]\n'
        "\n"
        "[workspace.package]\n"
        'version = "0.1.0"\n'
        'edition = "2024"\n',
        encoding="utf-8",
    )
    (external_spatial / "Cargo.toml").write_text(
        "[package]\n"
        'name = "spatial"\n'
        "version.workspace = true\n"
        "edition.workspace = true\n",
        encoding="utf-8",
    )
    (external_demo / "Cargo.toml").write_text(
        "[package]\n"
        'name = "chunk_streaming_demo"\n'
        "version.workspace = true\n"
        "edition.workspace = true\n"
        "\n"
        "[dependencies]\n"
        'tile_topology = { path = "../../../grid/crates/tile_topology" }\n',
        encoding="utf-8",
    )
    (grid_root / "Cargo.toml").write_text(
        "[workspace]\n"
        'members = ["crates/tile_topology"]\n'
        "\n"
        "[workspace.package]\n"
        'version = "0.1.0"\n'
        'edition = "2024"\n',
        encoding="utf-8",
    )
    (tile_topology / "Cargo.toml").write_text(
        "[package]\n"
        'name = "tile_topology"\n'
        "version.workspace = true\n"
        "edition.workspace = true\n",
        encoding="utf-8",
    )

    from execution.workspace import create_full_snapshot, dispose_workspace

    workspace = create_full_snapshot(execution_test_action(), repo_root=repo_root)
    try:
        mirrored = workspace.temp_root / "spatial_streaming" / "crates" / "spatial" / "Cargo.toml"
        mirrored_grid = workspace.temp_root / "grid" / "crates" / "tile_topology" / "Cargo.toml"
        assert mirrored.exists()
        assert (workspace.temp_root / "spatial_streaming" / "Cargo.toml").exists()
        assert mirrored_grid.exists()
        assert (workspace.temp_root / "grid" / "Cargo.toml").exists()
    finally:
        dispose_workspace(workspace)

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

def test_validation_command_declared_cargo_lock_output_imports(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    (tmp_path / "src").mkdir(parents=True)
    (tmp_path / "src/lib.rs").write_text("// before\n", encoding="utf-8")
    (tmp_path / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    (tmp_path / "Cargo.lock").write_text("# before\n", encoding="utf-8")
    action = cargo_lock_test_action()

    def fake_cargo_run(argv, *, cwd, **_kwargs):
        if argv[:2] == ["cargo", "test"]:
            (Path(cwd) / "Cargo.lock").write_text("# after\n", encoding="utf-8")
        return SimpleNamespace(returncode=0, stdout="", stderr="")

    monkeypatch.setattr("execution.runner.subprocess.run", fake_cargo_run)

    result = run_action(
        action,
        backend=FakeExecutionAgent({"src/lib.rs": "// after\n"}),
        lock_validated=True,
        repo_root=tmp_path,
        run_id="test-run",
    )

    assert (tmp_path / "src/lib.rs").read_text(encoding="utf-8") == "// after\n"
    assert (tmp_path / "Cargo.lock").read_text(encoding="utf-8") == "# after\n"
    assert tmp_path / "Cargo.lock" in result.validation_files_changed

def test_agent_phase_validation_only_cargo_lock_side_effect_is_reproduced_by_validation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    (tmp_path / "src").mkdir(parents=True)
    (tmp_path / "src/lib.rs").write_text("// before\n", encoding="utf-8")
    (tmp_path / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    (tmp_path / "Cargo.lock").write_text("# before\n", encoding="utf-8")
    action = cargo_lock_test_action()

    def fake_cargo_run(argv, *, cwd, **_kwargs):
        if argv[:2] == ["cargo", "test"]:
            (Path(cwd) / "Cargo.lock").write_text("# validation\n", encoding="utf-8")
        return SimpleNamespace(returncode=0, stdout="", stderr="")

    monkeypatch.setattr("execution.runner.subprocess.run", fake_cargo_run)

    result = run_action(
        action,
        backend=FakeExecutionAgent({"src/lib.rs": "// after\n", "Cargo.lock": "# agent\n"}),
        lock_validated=True,
        repo_root=tmp_path,
        run_id="test-run",
    )

    assert (tmp_path / "src/lib.rs").read_text(encoding="utf-8") == "// after\n"
    assert (tmp_path / "Cargo.lock").read_text(encoding="utf-8") == "# validation\n"
    assert tmp_path / "Cargo.lock" in result.validation_files_changed

def test_agent_phase_cargo_lock_still_fails_without_validation_output_authority(tmp_path: Path) -> None:
    (tmp_path / "src").mkdir(parents=True)
    (tmp_path / "src/lib.rs").write_text("// before\n", encoding="utf-8")
    (tmp_path / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    (tmp_path / "Cargo.lock").write_text("# before\n", encoding="utf-8")
    action = cargo_lock_test_action()
    action.validation_commands[0].allowed_outputs = []

    with pytest.raises(WorkflowError, match="changed undeclared file Cargo.lock"):
        run_action(
            action,
            backend=FakeExecutionAgent({"src/lib.rs": "// after\n", "Cargo.lock": "# agent\n"}),
            lock_validated=True,
            repo_root=tmp_path,
            run_id="test-run",
        )

    assert (tmp_path / "src/lib.rs").read_text(encoding="utf-8") == "// before\n"
    assert (tmp_path / "Cargo.lock").read_text(encoding="utf-8") == "# before\n"

def test_cargo_lock_validation_output_rejects_non_cargo_command() -> None:
    action = cargo_lock_test_action()
    action.validation_commands = [
        ValidationCommand(
            command_id="python3:version",
            argv=["python3", "--version"],
            allowed_outputs=["Cargo.lock"],
        )
    ]

    assert any("Cargo.lock validation output requires a cargo:* command" in error for error in preflight_action(action))

def test_cargo_lock_validation_output_requires_manifest_authority() -> None:
    action = cargo_lock_test_action()
    action.allowed_outputs = ["src/lib.rs"]

    assert any("Cargo.lock validation output requires exact Cargo.toml action output" in error for error in preflight_action(action))

def test_cargo_lock_validation_output_requires_crate_creation_for_new_crate() -> None:
    action = cargo_lock_test_action(new_cargo_output=True, crate_creation=False)

    assert any("Cargo.lock validation output for new crates requires crate_creation permission" in error for error in preflight_action(action))

def test_validation_output_digest_drift_blocks_import(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    (tmp_path / "src").mkdir(parents=True)
    (tmp_path / "src/lib.rs").write_text("// before\n", encoding="utf-8")
    (tmp_path / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    (tmp_path / "Cargo.lock").write_text("# before\n", encoding="utf-8")
    action = cargo_lock_test_action()

    def fake_cargo_run(argv, *, cwd, **_kwargs):
        if argv[:2] == ["cargo", "test"]:
            (Path(cwd) / "Cargo.lock").write_text("# workspace\n", encoding="utf-8")
            (tmp_path / "Cargo.lock").write_text("# drift\n", encoding="utf-8")
        return SimpleNamespace(returncode=0, stdout="", stderr="")

    monkeypatch.setattr("execution.runner.subprocess.run", fake_cargo_run)

    with pytest.raises(WorkflowError, match="target digest drifted before import: Cargo.lock"):
        run_action(
            action,
            backend=FakeExecutionAgent({"src/lib.rs": "// after\n"}),
            lock_validated=True,
            repo_root=tmp_path,
            run_id="test-run",
        )

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

def test_agent_transcript_root_uses_ignored_local_state(tmp_path: Path) -> None:
    action = execution_test_action()

    transcript_root = agent_transcript_root(action, repo_root=tmp_path, run_id="run-001")

    assert transcript_root is not None
    assert transcript_root == tmp_path / ".runenwerk/track-execution-transcripts/pt-test/run-001"

def test_scope_enforcement_rejects_out_of_scope_paths() -> None:
    violations = validate_changed_paths(["apps/a/file.rs", "engine/src/lib.rs"], ["apps/a"])
    assert violations == ["engine/src/lib.rs"]
