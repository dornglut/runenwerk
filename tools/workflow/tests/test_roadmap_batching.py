from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


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
        "task batch:finalize -- --batch reports/batch-test/batch.toml --target main --write --cleanup",
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

def test_batch_continue_selects_still_current_finalized_items() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    finalized_item = manifest.items[0].model_copy(
        update={
            "status": "integrated",
            "roadmap_outcome": "slice_landed_item_still_current",
        }
    )
    finalized = manifest.model_copy(
        update={
            "integration_status": "merged",
            "closeout_status": "completed",
            "items": [finalized_item],
        }
    )

    selected = continuation_items_for_manifest(finalized, roadmap)

    assert [item.id for item in selected] == ["WR-001"]
    assert default_continuation_goal(finalized, selected) == "Continue roadmap batch after batch-test: WR-001"

def test_batch_continue_rejects_open_batches() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )

    with pytest.raises(WorkflowError, match="batch must be finalized"):
        continuation_items_for_manifest(manifest, roadmap)

def test_batch_continue_writes_followup_manifest_from_cli() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        roadmap = RoadmapState.model_validate(valid_state())
        manifest = build_manifest(
            "batch-test",
            "test",
            [roadmap.items[0]],
            Path("docs-site/src/content/docs/reports/batches/batch-test"),
        )
        finalized_item = manifest.items[0].model_copy(
            update={
                "status": "integrated",
                "roadmap_outcome": "slice_landed_item_still_current",
            }
        )
        finalized = manifest.model_copy(
            update={
                "integration_status": "merged",
                "closeout_status": "completed",
                "items": [finalized_item],
            }
        )
        root = Path(temp_dir)
        source = root / "roadmap.yaml"
        batch = root / "batch.toml"
        out = root / "followup.toml"
        source.write_text(yaml.safe_dump(valid_state(), sort_keys=False), encoding="utf-8")
        batch.write_text(render_batch_manifest(finalized), encoding="utf-8")

        result = CliRunner().invoke(
            batch_app,
            [
                "continue",
                "--batch",
                str(batch),
                "--source",
                str(source),
                "--out",
                str(out),
                "--batch-id",
                "followup",
            ],
        )

        assert result.exit_code == 0, result.output
        followup = load_batch_manifest(out)
        assert followup.id == "followup"
        assert followup.approval_state == "proposed"
        assert [item.id for item in followup.items] == ["WR-001"]
        assert "task batch:approve" in result.output

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

def test_finalize_cleans_integrated_worktrees_and_branches(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker", "status": "slice_completed"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})
    removed_worktrees: list[str] = []
    deleted_branches: list[str] = []

    monkeypatch.setattr("parallel_batch.git_output", lambda _command: "abc123")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: True)
    monkeypatch.setattr("parallel_batch.branch_is_ancestor", lambda _branch, _target: True)
    monkeypatch.setattr("parallel_batch.changed_files_for_worktree", lambda _worktree, _base_sha: ["tools/workflow/file.py"])
    monkeypatch.setattr("parallel_batch.path_matches_ref", lambda _worktree, _target, _path: True)
    monkeypatch.setattr("parallel_batch.Path.exists", lambda _path: True)
    monkeypatch.setattr("parallel_batch.remove_worker_worktree_if_present", lambda path: removed_worktrees.append(str(path)))
    monkeypatch.setattr("parallel_batch.delete_worker_branch", lambda branch: deleted_branches.append(branch))

    finalized = finalize_batch_manifest(approved, roadmap, target="main", cleanup=True)

    assert removed_worktrees == ["worker"]
    assert deleted_branches == ["codex/batch-test-wr-001"]
    assert finalized.integration_status == "merged"
    assert finalized.closeout_status == "completed"
    assert finalized.integrated_target == "main"
    assert finalized.items[0].status == "integrated"
    assert finalized.items[0].roadmap_outcome == "slice_landed_item_still_current"
    assert finalized.items[0].worktree == ""
    assert finalized.items[0].worktree_cleanup == "cleaned after integration"

def test_batch_report_renders_cleaned_worktrees_and_roadmap_outcome() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    item = manifest.items[0].model_copy(
        update={
            "worktree": "",
            "worktree_cleanup": "cleaned after integration",
            "status": "integrated",
            "roadmap_outcome": "slice_landed_item_still_current",
        }
    )
    finalized = manifest.model_copy(
        update={
            "integration_status": "merged",
            "closeout_status": "completed",
            "integrated_target": "main",
            "integrated_sha": "abc123",
            "items": [item],
        }
    )
    report = render_batch_report(finalized)

    assert "Integrated into: main@abc123" in report
    assert "- Worktree: `cleaned after integration`" in report
    assert "- Roadmap outcome: `slice_landed_item_still_current`" in report

def test_hygiene_rejects_finalized_manifest_with_stale_worktree() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    item_with_stale_worktree = manifest.items[0].model_copy(
        update={"worktree": "worker", "prompt_path": "docs-site/src/content/docs/workspace/roadmap-index.md"}
    )
    finalized = manifest.model_copy(
        update={"integration_status": "merged", "closeout_status": "completed", "items": [item_with_stale_worktree]}
    )

    assert batch_manifest_errors(Path("reports/batch-test/batch.toml"), finalized) == [
        "reports/batch-test/batch.toml:WR-001: finalized batch still records active worktree "
        + str((REPO_ROOT / "worker")).replace("\\", "/")
    ]

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

def test_finalize_refuses_dirty_in_scope_worker_changes(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    batch_item = manifest.items[0].model_copy(update={"worktree": "worker"})
    approved = manifest.model_copy(update={"approval_state": "approved", "items": [batch_item]})

    monkeypatch.setattr("parallel_batch.git_output", lambda command: "target" if "main" in command else "")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: False)
    monkeypatch.setattr("parallel_batch.changed_files_for_worktree", lambda _worktree, _base_sha: ["tools/workflow/file.py"])
    monkeypatch.setattr("parallel_batch.path_matches_ref", lambda _worktree, _target, _path: False)
    monkeypatch.setattr("parallel_batch.Path.exists", lambda _path: True)

    assert batch_finalization_errors(approved, "main") == [
        "WR-001: dirty in-scope worker change is not integrated into main: tools/workflow/file.py"
    ]

def test_finalize_refuses_unmerged_worker_commits(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )

    monkeypatch.setattr("parallel_batch.git_output", lambda command: "target" if "main" in command else "branch")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: True)
    monkeypatch.setattr("parallel_batch.branch_is_ancestor", lambda _branch, _target: False)

    assert batch_finalization_errors(manifest, "main") == [
        "WR-001: worker branch codex/batch-test-wr-001 has commits not integrated into main"
    ]

def test_path_matches_ref_respects_git_text_normalization() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        repo = Path(temp_dir)
        subprocess.run(["git", "init"], cwd=repo, check=True, stdout=subprocess.DEVNULL)
        subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo, check=True)
        subprocess.run(["git", "config", "user.email", "test@example.invalid"], cwd=repo, check=True)
        (repo / ".gitattributes").write_text("*.md text\n", encoding="utf-8", newline="\n")
        (repo / "note.md").write_text("one\ntwo\n", encoding="utf-8", newline="\n")
        subprocess.run(["git", "add", ".gitattributes", "note.md"], cwd=repo, check=True)
        subprocess.run(["git", "commit", "-m", "seed"], cwd=repo, check=True, stdout=subprocess.DEVNULL)

        (repo / "note.md").write_text("one\ntwo\n", encoding="utf-8", newline="\r\n")

        assert path_matches_ref(repo, "HEAD", "note.md")

def test_finalize_preserves_branches_when_requested(monkeypatch: pytest.MonkeyPatch) -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    deleted_branches: list[str] = []

    monkeypatch.setattr("parallel_batch.git_output", lambda _command: "abc123")
    monkeypatch.setattr("parallel_batch.branch_exists", lambda _branch: True)
    monkeypatch.setattr("parallel_batch.branch_is_ancestor", lambda _branch, _target: True)
    monkeypatch.setattr("parallel_batch.delete_worker_branch", lambda branch: deleted_branches.append(branch))

    finalize_batch_manifest(manifest, roadmap, target="main", cleanup=True, keep_branches=True)

    assert deleted_branches == []

def test_hygiene_uses_portable_merged_branch_option_order(monkeypatch: pytest.MonkeyPatch) -> None:
    commands: list[list[str]] = []

    def fake_git_stdout(command: list[str]) -> str:
        commands.append(command)
        return "main\ncodex/done\n"

    monkeypatch.setattr("repo_hygiene.git_stdout", fake_git_stdout)

    assert local_branches(merged_only=True) == ["main", "codex/done"]
    assert commands == [["branch", "--format=%(refname:short)", "--merged"]]

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

