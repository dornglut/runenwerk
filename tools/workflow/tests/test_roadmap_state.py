from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *


def test_a_wsjf_score_is_computed() -> None:
    roadmap = RoadmapState.model_validate(valid_state())
    assert roadmap.items[0].score == 2.7

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

def test_unmet_decision_gate_excludes_and_rejects_current_candidate() -> None:
    state = valid_state()
    state["items"][0]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]
    roadmap = RoadmapState.model_validate(state)

    assert select_batch_candidates(roadmap, level="L0") == []
    with pytest.raises(WorkflowError, match="decision gate unmet"):
        select_batch_candidates(roadmap, item_ids=("WR-001",))

def test_accepted_decision_gate_allows_current_candidate() -> None:
    state = valid_state()
    state["items"][0]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md")
    ]
    roadmap = RoadmapState.model_validate(state)

    assert [item.id for item in select_batch_candidates(roadmap, level="L0")] == ["WR-001"]

def test_missing_production_milestone_dependency_is_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["dependencies"] = ["PM-TEST-999"]

    with pytest.raises(ValueError, match="unknown milestone dependency"):
        ProductionPlanningState.model_validate(state)

def test_production_milestone_dependency_cycle_is_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["dependencies"] = ["PM-TEST-002"]

    with pytest.raises(ValueError, match="production milestone dependency cycle"):
        ProductionPlanningState.model_validate(state)

def test_missing_wr_roadmap_link_is_rejected() -> None:
    state = valid_production_state()
    state["tracks"][0]["milestones"][0]["roadmap_links"] = ["WR-999"]
    planning = ProductionPlanningState.model_validate(state)

    assert validate_roadmap_links(planning) == ["PM-TEST-001: unknown roadmap link WR-999"]

def test_current_candidate_row_classifies_as_implementation_contract() -> None:
    context = production_plan_context(roadmap_state="current_candidate")

    assert classify_plan_action(context) == "write_implementation_contract"

def test_manifest_backed_validation_rejects_uncovered_wr_write_scope(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["milestones"][0]["write_scope"] = ["docs-site/src/content/docs/workspace/track-execution-manifest.md"]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("manifest write_scope docs-site/src/content/docs/workspace/track-execution-manifest.md" in error for error in errors)
    assert any("owning WR WR-001 write_scopes" in error for error in errors)

def test_pt_ui_program_manifest_write_scope_is_wr_covered() -> None:
    planning = load_production_tracks()
    roadmap = load_roadmap()
    track = find_track(planning, "PT-UI-PROGRAM")
    loaded_manifest = load_track_execution_manifest("PT-UI-PROGRAM")
    assert loaded_manifest is not None

    errors = audit_manifest(loaded_manifest, track=track, roadmap=roadmap)

    assert [error for error in errors if "manifest write_scope" in error] == []

def test_production_goal_stack_routes_to_first_incomplete_dependency_track() -> None:
    planning = ProductionPlanningState.model_validate(valid_production_stack_state())
    roadmap = RoadmapState.model_validate(valid_state())
    root_track = find_track(planning, "PT-END")

    rendered = render_stack_goal(planning, roadmap, root_track)

    assert "Production Stack /goal Kickoff: PT-END" in rendered
    assert QUALITY_DOCTRINE_ID in rendered
    assert "- PT-BASE - Base production track: PM-BASE-001 -> execute_next_wr_implementation_contract" in rendered
    assert "- PT-END - Final production audit: PM-END-001 -> wait_for_dependency_completion" in rendered
    assert "Current single-track command: task ai:goal -- --track PT-BASE" in rendered
    assert "Use task ai:goal -- --track PT-END --stack as the stack coordinator" in rendered
    assert "Cross-track dependency waits are routing signals in stack mode" in rendered

def test_production_goal_stack_cli_renders_dependency_stack(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    write_yaml(production_path, valid_production_stack_state())
    write_yaml(roadmap_path, valid_state())

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-END",
            "--stack",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
        ],
    )

    assert result.exit_code == 0
    assert "Production Stack /goal Kickoff: PT-END" in result.stdout
    assert "task ai:goal -- --track PT-BASE" in result.stdout

def test_document_frontmatter_status_handles_crlf(tmp_path: Path) -> None:
    doc = tmp_path / "adr.md"
    doc.write_text("---\r\nstatus: accepted\r\n---\r\n# ADR\r\n", encoding="utf-8", newline="")

    assert document_frontmatter_status(doc) == "accepted"

def test_roadmap_write_scopes_reject_absolute_workspace_paths() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = [
        "/var/folders/example/execution-workspace/docs-site/src/content/docs/workspace/production-tracks.yaml"
    ]

    with pytest.raises(ValueError, match="write_scopes must be repo-relative"):
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

def test_new_write_scope_paths_require_existing_parent_only() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = ["new: tools/workflow/future_scope_test.py"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_existing_write_scope_paths([roadmap.items[0]]) == []
    assert validate_changed_paths(["tools/workflow/future_scope_test.py"], roadmap.items[0].write_scopes) == []

def test_new_write_scope_paths_allow_intermediate_directories_under_existing_owner() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = ["new: domain/ui/future_ui_owner/src/lib.rs"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_existing_write_scope_paths([roadmap.items[0]]) == []
    assert validate_changed_paths(["domain/ui/future_ui_owner/src/lib.rs"], roadmap.items[0].write_scopes) == []

def test_new_write_scope_paths_reject_missing_parent() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = ["new: missing/parent/future_scope_test.py"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_existing_write_scope_paths([roadmap.items[0]]) == [
        "WR-001:missing/parent/future_scope_test.py parent does not exist for new write scope"
    ]

def test_generated_write_scope_paths_do_not_require_existing_files() -> None:
    state = valid_state()
    state["items"][0]["write_scopes"] = [
        "generated: production docs from task production:render",
        "derived: roadmap docs from task roadmap:render",
    ]
    roadmap = RoadmapState.model_validate(state)

    assert validate_existing_write_scope_paths([roadmap.items[0]]) == []
    assert validate_write_scopes(roadmap.items) == []

def test_schema_generation_check_detects_missing_files() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        from roadmap_state import BATCH_SCHEMA, ROADMAP_ITEM_SOURCE_SCHEMA, ROADMAP_SCHEMA

        assert ROADMAP_SCHEMA.name == "roadmap-items.schema.json"
        assert ROADMAP_ITEM_SOURCE_SCHEMA.name == "roadmap-item-source.schema.json"
        assert BATCH_SCHEMA.name == "batch-manifest.schema.json"

def test_split_roadmap_sources_combine_active_archive_and_deferred_rows() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        root = Path(temp_dir)
        state = valid_state()
        state["items"] = [
            item("WR-030", dependencies=["WR-001"], write_scopes=["tools/workflow"]),
        ]
        state["edges"] = [{"source": "WR-001", "target": "WR-030", "label": "baseline"}]
        archive = {
            "version": state["version"],
            "roadmap": state["roadmap"],
            "items": [item("WR-001", planning_state="completed", blocker=1, dependencies=[])],
        }
        deferred = {
            "version": state["version"],
            "roadmap": state["roadmap"],
            "items": [item("WR-011", planning_state="blocked_deferred", blocker=5, dependencies=[])],
        }
        source = root / "roadmap-items.yaml"
        source.write_text(yaml.safe_dump(state, sort_keys=False), encoding="utf-8")
        (root / "roadmap-archive.yaml").write_text(yaml.safe_dump(archive, sort_keys=False), encoding="utf-8")
        (root / "roadmap-deferred.yaml").write_text(yaml.safe_dump(deferred, sort_keys=False), encoding="utf-8")

        roadmap = load_roadmap(source)
        validate_roadmap_with_json_schema(source)

        assert [item.id for item in roadmap.active_items] == ["WR-030"]
        assert [item.id for item in roadmap.archived_items] == ["WR-001"]
        assert [item.id for item in roadmap.deferred_items] == ["WR-011"]
        assert sorted(roadmap.by_id) == ["WR-001", "WR-011", "WR-030"]

def test_split_roadmap_sources_reject_completed_or_deferred_active_rows() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    combined = combine_roadmap_data(
        state,
        archive_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
        deferred_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
    )

    with pytest.raises(ValueError, match="active roadmap source must not contain completed items"):
        RoadmapState.model_validate(combined)

def test_split_roadmap_sources_reject_wrong_archive_or_deferred_states() -> None:
    state = valid_state()
    state["items"] = [item("WR-030", dependencies=[])]
    state["edges"] = []
    archive = {
        "version": state["version"],
        "roadmap": state["roadmap"],
        "items": [item("WR-001", planning_state="ready_next", blocker=2, dependencies=[])],
    }
    combined = combine_roadmap_data(
        state,
        archive_data=archive,
        deferred_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
    )

    with pytest.raises(ValueError, match="archive source items must be completed"):
        RoadmapState.model_validate(combined)

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

def test_roadmap_completion_rejects_vague_evidence_without_path() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = "Validated and complete."
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items) == [
        "WR-001: completed items must reference an existing completed closeout or batch evidence path"
    ]

def test_roadmap_completion_rejects_missing_closeout_path(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    state["items"][0]["write_scopes"] = [evidence_path]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == [
        f"WR-001: completion evidence path does not exist: {evidence_path}"
    ]

def test_roadmap_completion_rejects_non_completed_closeout_frontmatter(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    closeout = tmp_path / evidence_path
    closeout.parent.mkdir(parents=True)
    closeout.write_text("---\nstatus: draft\n---\n# Draft\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    state["items"][0]["write_scopes"] = [evidence_path]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == [
        f"WR-001: completion closeout evidence status 'draft' is not 'completed': {evidence_path}"
    ]

def test_roadmap_completion_requires_evidence_path_in_write_scopes(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    closeout = tmp_path / evidence_path
    closeout.parent.mkdir(parents=True)
    closeout.write_text("---\nstatus: completed\n---\n# Done\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == [
        "WR-001: completed items must include a completed closeout or batch evidence path in write_scopes"
    ]

def test_roadmap_completion_accepts_completed_closeout_evidence(tmp_path: Path) -> None:
    evidence_path = "docs-site/src/content/docs/reports/closeouts/wr-001-test/closeout.md"
    closeout = tmp_path / evidence_path
    closeout.parent.mkdir(parents=True)
    closeout.write_text("---\nstatus: completed\n---\n# Done\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = f"Closeout evidence landed in {evidence_path}."
    state["items"][0]["write_scopes"] = [evidence_path]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_evidence(roadmap.items, repo_root=tmp_path) == []

def test_completed_roadmap_items_require_completion_quality() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items) == [
        "WR-001: completed items must set completion_quality"
    ]

def test_perfectionist_roadmap_quality_rejects_gaps_or_missing_audit(tmp_path: Path) -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["completion_quality"] = "perfectionist_verified"
    state["items"][0]["known_quality_gaps"] = ["still has a UI gap"]
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items, repo_root=tmp_path) == [
        "WR-001: perfectionist_verified items must not list known_quality_gaps",
        "WR-001: perfectionist_verified items must reference a completed audit",
    ]

def test_perfectionist_roadmap_quality_requires_completed_audit(tmp_path: Path) -> None:
    audit_path = "docs-site/src/content/docs/reports/audits/wr-001-audit.md"
    audit = tmp_path / audit_path
    audit.parent.mkdir(parents=True)
    audit.write_text("---\nstatus: draft\n---\n# Audit\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["completion_quality"] = "perfectionist_verified"
    state["items"][0]["completion_audit"] = audit_path
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items, repo_root=tmp_path) == [
        f"WR-001: completion_audit status 'draft' is not 'completed': {audit_path}"
    ]

def test_perfectionist_roadmap_quality_accepts_completed_audit(tmp_path: Path) -> None:
    audit_path = "docs-site/src/content/docs/reports/audits/wr-001-audit.md"
    audit = tmp_path / audit_path
    audit.parent.mkdir(parents=True)
    audit.write_text("---\nstatus: completed\n---\n# Audit\n", encoding="utf-8")
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["completion_quality"] = "perfectionist_verified"
    state["items"][0]["completion_audit"] = audit_path
    roadmap = RoadmapState.model_validate(state)

    assert validate_completion_quality(roadmap.items, repo_root=tmp_path) == []

def test_duplicate_ids_are_rejected() -> None:
    state = valid_state()
    state["items"][1]["id"] = "WR-001"
    with pytest.raises(ValueError):
        RoadmapState.model_validate(state)

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

def test_non_executable_run_track_rejects_product_action(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    production_path, roadmap_path, manifest_root, plan_path, *_ = prepare_full_automation_product_fixture(
        tmp_path,
        monkeypatch,
    )
    manifest = load_yaml(manifest_root / "pt-test.yaml")
    manifest["ai_executable"] = False
    manifest["full_automation_target"] = False
    write_yaml(manifest_root / "pt-test.yaml", manifest)
    monkeypatch.setattr("execution.compiler.default_contract_path", lambda _item: plan_path)

    result = CliRunner().invoke(
        production_track_cli_app,
        [
            "run-track",
            "--track",
            "PT-TEST",
            "--allow",
            "product_code",
            "--allow",
            "product_implementation",
            "--max-actions",
            "1",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
            "--lock-source-root",
            str(tmp_path / "locks"),
        ],
    )

    assert result.exit_code == 1
    assert "non-executable track mutation supports only planning_expansion or" in result.stdout
    assert "design_authoring; next action is product_implementation" in result.stdout

def test_new_file_scope_errors_reject_untracked_existing_repo_file(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    candidate = tmp_path / "domain/ui/ui_widgets/src/untracked_existing.rs"
    candidate.parent.mkdir(parents=True)
    candidate.write_text("// existing but untracked\n", encoding="utf-8")
    monkeypatch.setattr(track_source_audit_module, "REPO_ROOT", tmp_path)
    monkeypatch.setattr(track_source_audit_module, "git_tracks_path", lambda _path: False)

    errors = new_file_scope_errors(
        "PM-TEST-NEW",
        ["domain/ui/ui_widgets/src/untracked_existing.rs"],
        label="product_code_contract",
    )

    assert errors == [
        "PM-TEST-NEW: product_code_contract new file scope must be marked with 'new:': "
        "domain/ui/ui_widgets/src/untracked_existing.rs"
    ]

def test_ready_next_rows_may_carry_future_implementation_gates() -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "ready_next"
    state["items"][0]["decision_gates"] = [
        decision_gate("docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md")
    ]
    roadmap = RoadmapState.model_validate(state)

    assert [item.id for item in select_batch_candidates(roadmap, item_ids=("WR-001",), include_discovery=True)] == [
        "WR-001"
    ]

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

def test_completed_items_are_rejected_from_current_docs(tmp_path: Path) -> None:
    state = valid_state()
    state["items"][0]["planning_state"] = "completed"
    state["items"][0]["next_evidence"] = "Closeout evidence."
    roadmap = RoadmapState.model_validate(state)
    doc = tmp_path / "roadmap-index.md"
    doc.write_text("WR-001 is the current implementation candidate.\n", encoding="utf-8")
    expected_path = str(doc).replace("\\", "/")

    assert validate_completed_items_not_current_in_docs(roadmap.items, [doc]) == [
        f"{expected_path}:1: completed item WR-001 is still described as current work"
    ]

