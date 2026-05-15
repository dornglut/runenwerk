from __future__ import annotations

import tempfile
import subprocess
from pathlib import Path

import pytest
import yaml
from typer.testing import CliRunner

from generate_roadmap_docs import render_current_candidates_roadmap, render_dependency_roadmap, render_outputs
from parallel_batch import (
    app as batch_app,
    batch_execution_state,
    build_manifest,
    default_batch_id,
    default_kickoff_goal,
    kickoff_next_step_lines,
    run_host_batch_validation,
    run_official_batch_validation,
    refresh_base_manifest,
    render_worker_prompt,
    validation_commands_for_items,
    write_validation_result,
    worktree_path_for_item,
)
from roadmap_state import (
    REPO_ROOT,
    BatchManifest,
    RoadmapState,
    WorkflowError,
    changed_files_for_worktree,
    load_batch_manifest,
    load_roadmap,
    select_batch_candidates,
    validate_batch_against_roadmap,
    validate_changed_paths,
    validate_existing_write_scope_paths,
    validate_write_scopes,
    write_schema_files,
)
from roadmap_intake import (
    apply_intake_proposal,
    build_intake_proposal,
    load_intake_proposal,
    proposal_to_yaml_data,
    roadmap_data_with_promotion,
    roadmap_data_with_proposal,
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
    state["items"][0]["planning_state"] = "completed"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Ready next"
    state["items"][1]["planning_state"] = "current_candidate"
    roadmap = RoadmapState.model_validate(state)

    dependency = render_dependency_roadmap(roadmap)
    candidates = render_current_candidates_roadmap(roadmap)

    assert "Level 0 - Completed / Support Substrate" in dependency
    assert "Parallel" + " Now" not in dependency
    assert "state=completed" in dependency
    assert "Current Implementation Candidates" in candidates
    assert "state=current_candidate" in candidates
    assert "state=completed" in candidates


def test_schema_generation_check_detects_missing_files() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        from roadmap_state import BATCH_SCHEMA, ROADMAP_SCHEMA

        assert ROADMAP_SCHEMA.name == "roadmap-items.schema.json"
        assert BATCH_SCHEMA.name == "batch-manifest.schema.json"


def test_batch_manifest_and_worker_prompt() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest("batch-test", "test", [roadmap.items[0]], Path("docs-site/src/content/docs/reports/batches/batch-test"))
    assert manifest.items[0].branch == "codex/batch-test-wr-001"
    assert manifest.items[0].prompt_path.endswith("/wr-001.md")
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
        "task batch:closeout -- --batch reports/batch-test/batch.toml --write",
    ]


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
