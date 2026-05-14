from __future__ import annotations

import tempfile
import subprocess
from pathlib import Path

import pytest
import yaml

from generate_roadmap_docs import render_outputs
from parallel_batch import build_manifest, render_worker_prompt
from roadmap_state import RoadmapState, WorkflowError, changed_files_for_worktree, load_roadmap, select_batch_candidates, validate_batch_against_roadmap, validate_changed_paths, validate_write_scopes, write_schema_files


def valid_state() -> dict:
    return {
        "version": 1,
        "roadmap": {"title": "Test Roadmap", "last_reviewed": "2026-05-14", "owner": "workspace"},
        "render": {
            "decision_register": "decision.md",
            "dependency_roadmap": "roadmap.puml",
            "triage": "triage.md",
        },
        "items": [
            item("WR-001", dependencies=[], write_scopes=["apps/a"]),
            item("WR-002", value=2, blocker=5, dependencies=["WR-001"], write_scopes=["apps/b"]),
        ],
        "edges": [{"source": "WR-001", "target": "WR-002", "label": "depends"}],
    }


def item(
    item_id: str,
    *,
    value: int = 4,
    blocker: int = 2,
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
        "status": "implement_now",
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
        "write_scopes": write_scopes or ["apps/a"],
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


def test_overlapping_write_scopes_are_detected() -> None:
    state = valid_state()
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Supporting now"
    state["items"][1]["write_scopes"] = ["apps/a/subsystem"]
    roadmap = RoadmapState.model_validate(state)
    assert validate_write_scopes(roadmap.items) == ["WR-002:apps/a/subsystem overlaps WR-001:apps/a"]


def test_render_check_can_detect_stale_files() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        state = valid_state()
        state["render"] = {
            "decision_register": str(root / "decision.md"),
            "dependency_roadmap": str(root / "roadmap.puml"),
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


def test_batch_approval_validation_rejects_blocked_items() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[1]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )

    assert validate_batch_against_roadmap(manifest, roadmap) == [
        "WR-002: roadmap gate is not implementation-ready"
    ]


def test_batch_approval_validation_rejects_stale_scope() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    manifest = build_manifest(
        "batch-test",
        "test",
        [roadmap.items[0]],
        Path("docs-site/src/content/docs/reports/batches/batch-test"),
    )
    stale_item = manifest.items[0].model_copy(update={"write_scopes": ["apps/other"]})
    stale_manifest = manifest.model_copy(update={"items": [stale_item]})

    assert validate_batch_against_roadmap(stale_manifest, roadmap) == [
        "WR-001: write_scopes are stale"
    ]


def test_scope_enforcement_rejects_out_of_scope_paths() -> None:
    violations = validate_changed_paths(["apps/a/file.rs", "engine/src/lib.rs"], ["apps/a"])
    assert violations == ["engine/src/lib.rs"]


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
