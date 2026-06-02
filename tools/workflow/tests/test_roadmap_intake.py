from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_production_goal_surfaces_switch_current_candidate_action() -> None:
    production_data = valid_production_state()
    production_data["tracks"][0]["milestones"][0]["roadmap_links"] = ["WR-003"]
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(valid_state_with_switch_target())
    track = find_track(planning, "PT-TEST")

    steps = build_goal_steps(planning, roadmap, track)
    rendered = render_track_goal(planning, roadmap, track)

    assert steps[0].roadmap_actions[0].action == "switch_current_candidate"
    assert steps[0].next_action == "switch_current_candidate"
    assert "After a failed roadmap:promote or gate command" in rendered

def test_deferred_activation_moves_one_row_to_active_ready_next(monkeypatch: pytest.MonkeyPatch) -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        active = valid_state()
        active["items"] = [item("WR-001", dependencies=[], write_scopes=["tools/workflow"])]
        active["edges"] = [{"source": "WR-001", "target": "WR-169", "label": "enables"}]
        deferred = {
            "version": active["version"],
            "roadmap": active["roadmap"],
            "items": [
                item(
                    "WR-169",
                    blocker=4,
                    planning_state="blocked_deferred",
                    dependencies=["WR-001"],
                    write_scopes=["tools/workflow/test_workflow.py"],
                )
            ],
        }
        source = root / "roadmap-items.yaml"
        deferred_source = root / "roadmap-deferred.yaml"
        source.write_text(yaml.safe_dump(active, sort_keys=False), encoding="utf-8")
        deferred_source.write_text(yaml.safe_dump(deferred, sort_keys=False), encoding="utf-8")
        monkeypatch.setattr("roadmap_intake.render_and_check", lambda *_args, **_kwargs: None)

        roadmap = activate_deferred_roadmap_item(
            "WR-169",
            state="ready_next",
            evidence="PM-011 draft is ready for active contract review.",
            source=source,
        )

        active_after = load_yaml(source)
        deferred_after = load_yaml(deferred_source)
        activated = next(row for row in active_after["items"] if row["id"] == "WR-169")
        assert [row["id"] for row in deferred_after["items"]] == []
        assert activated["planning_state"] == "ready_next"
        assert activated["current_decision"] == "PM-011 draft is ready for active contract review."
        assert roadmap.by_id["WR-169"].planning_state == "ready_next"

def test_deferred_activation_rejects_policy_deferred_ready_next(monkeypatch: pytest.MonkeyPatch) -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        active = valid_state()
        active["items"] = [item("WR-001", dependencies=[], write_scopes=["tools/workflow"])]
        active["edges"] = []
        deferred = {
            "version": active["version"],
            "roadmap": active["roadmap"],
            "items": [item("WR-170", blocker=5, planning_state="blocked_deferred", dependencies=[])],
        }
        source = root / "roadmap-items.yaml"
        deferred_source = root / "roadmap-deferred.yaml"
        source.write_text(yaml.safe_dump(active, sort_keys=False), encoding="utf-8")
        original_deferred = yaml.safe_dump(deferred, sort_keys=False)
        deferred_source.write_text(original_deferred, encoding="utf-8")
        monkeypatch.setattr("roadmap_intake.render_and_check", lambda *_args, **_kwargs: None)

        with pytest.raises(WorkflowError, match="activation to ready_next requires B4 or lower"):
            activate_deferred_roadmap_item(
                "WR-170",
                state="ready_next",
                evidence="Policy deferred row is not ready.",
                source=source,
            )

        assert deferred_source.read_text(encoding="utf-8") == original_deferred

def test_deferred_activation_to_current_candidate_uses_b2_gate(monkeypatch: pytest.MonkeyPatch) -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        active = valid_state()
        active["items"] = [item("WR-001", dependencies=[], write_scopes=["tools/workflow"])]
        active["edges"] = []
        deferred = {
            "version": active["version"],
            "roadmap": active["roadmap"],
            "items": [
                item(
                    "WR-169",
                    blocker=4,
                    planning_state="blocked_deferred",
                    dependencies=["WR-001"],
                    write_scopes=["tools/workflow/test_workflow.py"],
                )
            ],
        }
        source = root / "roadmap-items.yaml"
        source.write_text(yaml.safe_dump(active, sort_keys=False), encoding="utf-8")
        (root / "roadmap-deferred.yaml").write_text(yaml.safe_dump(deferred, sort_keys=False), encoding="utf-8")
        monkeypatch.setattr("roadmap_intake.render_and_check", lambda *_args, **_kwargs: None)

        with pytest.raises(WorkflowError, match="B4 is above the B2 implementation gate"):
            activate_deferred_roadmap_item(
                "WR-169",
                state="current_candidate",
                evidence="Attempt current candidate before implementation gate.",
                source=source,
            )

def test_deferred_activation_helper_rejects_active_duplicate() -> None:
    active = valid_state()
    deferred = {
        "version": active["version"],
        "roadmap": active["roadmap"],
        "items": [item("WR-001", planning_state="blocked_deferred", dependencies=[])],
    }

    with pytest.raises(WorkflowError, match="already present in active roadmap source"):
        roadmap_data_with_deferred_activation(
            active,
            deferred,
            item_id="WR-001",
            state="ready_next",
            evidence="Duplicate activation.",
        )

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

def test_promote_rejects_current_candidate_with_unmet_decision_gate() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][1]["planning_state"] = "ready_next"
    state["items"][1]["blocker"] = 2
    state["items"][1]["gate"] = "Ready next"
    state["items"][1]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]

    with pytest.raises(WorkflowError, match="does not match required"):
        roadmap_data_with_promotion(
            state,
            item_id="WR-002",
            state="current_candidate",
            evidence="Ready after review.",
        )

def test_switch_current_candidate_updates_two_items_atomically(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source = tmp_path / "roadmap.yaml"
    write_yaml(source, valid_state_with_switch_target())
    monkeypatch.setattr("roadmap_intake.render_and_check", lambda _roadmap, *, skip_checks=False: None)

    switch_current_candidate(
        from_id="WR-001",
        to_id="WR-003",
        evidence="Switch to Workbench handles.",
        source=source,
    )
    roadmap = RoadmapState.model_validate(load_yaml(source))

    assert roadmap.by_id["WR-001"].planning_state == "ready_next"
    assert roadmap.by_id["WR-003"].planning_state == "current_candidate"
    assert roadmap.by_id["WR-003"].current_decision == "Switch to Workbench handles."

def test_switch_current_candidate_writes_nothing_when_validation_fails(tmp_path: Path) -> None:
    source = tmp_path / "roadmap.yaml"
    state = valid_state_with_switch_target()
    state["items"][2]["blocker"] = 3
    write_yaml(source, state)
    before = source.read_text(encoding="utf-8")

    with pytest.raises(WorkflowError, match="above the B2 implementation gate"):
        switch_current_candidate(
            from_id="WR-001",
            to_id="WR-003",
            evidence="Switch to Workbench handles.",
            source=source,
        )

    assert source.read_text(encoding="utf-8") == before

