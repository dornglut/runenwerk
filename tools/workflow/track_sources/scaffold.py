from __future__ import annotations

from pathlib import Path

import yaml

from production_state import PRODUCTION_SOURCE, ProductionMilestone, ProductionTrack, load_production_tracks
from roadmap_state import ROADMAP_SOURCE, WorkflowError, load_roadmap, repo_path
from track_sources.manifest import TRACK_EXECUTION_MANIFEST_ROOT, manifest_source_path


def future_wr_candidate_for_milestone(milestone: ProductionMilestone) -> str:
    suffix = milestone.id.removeprefix("PM-")
    return f"WR-TBD-{suffix}"


def manifest_type_for_milestone(milestone: ProductionMilestone) -> str:
    if milestone.kind == "implementation":
        return "implementation"
    if milestone.kind == "hardening":
        return "hardening"
    if milestone.kind == "release":
        return "closeout"
    return "design_only"


def execution_kind_for_milestone(milestone: ProductionMilestone) -> str:
    if milestone.kind == "implementation":
        return "implementation_proof"
    if milestone.kind == "hardening":
        return "proof_aggregation"
    if milestone.kind == "release":
        return "handoff_closeout"
    return "design_contract"


def milestone_permissions(milestone: ProductionMilestone) -> list[str]:
    if milestone.kind == "implementation":
        return ["agent_design", "product_code", "product_implementation", "runtime_closeout"]
    if milestone.kind == "hardening":
        return ["agent_design", "product_code", "product_implementation", "runtime_closeout"]
    if milestone.kind == "release":
        return ["agent_closeout", "handoff"]
    return ["agent_design", "agent_closeout"]


def accepted_design_dependencies(track: ProductionTrack) -> list[dict]:
    seen: set[str] = set()
    dependencies: list[dict] = []
    for milestone in track.milestones:
        for gate in milestone.design_gates:
            if gate.path in seen:
                continue
            seen.add(gate.path)
            dependencies.append(
                {
                    "path": gate.path,
                    "required_status": gate.required_status,
                    "reason": gate.reason,
                }
            )
    if dependencies:
        return dependencies
    return [
        {
            "path": "docs-site/src/content/docs/workspace/track-execution-manifest.md",
            "required_status": "active",
            "reason": "Manifest scaffolds without track-specific design gates must still anchor to the execution-manifest governance guide.",
        }
    ]


def scaffold_write_scope(track_id: str, milestone: ProductionMilestone, roadmap_item_scopes: list[str] | None) -> list[str]:
    if roadmap_item_scopes:
        return list(roadmap_item_scopes)
    return [
        "docs-site/src/content/docs/workspace/production-tracks.yaml",
        "docs-site/src/content/docs/workspace/roadmap-deferred.yaml",
        f"docs-site/src/content/docs/workspace/track-execution-manifests/{track_id.lower()}.yaml",
    ]


def closeout_path_for_milestone(milestone: ProductionMilestone) -> str:
    slug = milestone.title.lower().replace("/", " ").replace("&", " ").replace("+", " ")
    slug = "-".join(part for part in slug.split() if part)
    return f"docs-site/src/content/docs/reports/closeouts/{milestone.id.lower()}-{slug}/closeout.md"


def manifest_scaffold(track: ProductionTrack, *, roadmap_source: Path = ROADMAP_SOURCE) -> dict:
    roadmap = load_roadmap(roadmap_source)
    milestones: list[dict] = []
    for milestone in track.milestones:
        owning_wr = milestone.roadmap_links[0] if milestone.roadmap_links else None
        roadmap_item = roadmap.by_id.get(owning_wr) if owning_wr else None
        entry: dict = {
            "milestone_id": milestone.id,
            "title": milestone.title,
            "stage": milestone.kind,
            "authority_level": "planning_and_sequencing_only",
            "milestone_type": manifest_type_for_milestone(milestone),
            "predecessor_dependencies": list(milestone.dependencies),
            "write_scope": scaffold_write_scope(track.id, milestone, roadmap_item.write_scopes if roadmap_item else None),
            "forbidden_scope": [
                "product code unless an active WR and plan explicitly authorize it",
                "crate creation unless exact crate paths are authorized",
                "foundation/meta extraction",
            ],
            "required_contracts": ["Track Execution Manifest", "WR roadmap authority", "implementation plan before execution"],
            "validation_commands": ["task production:validate", "task roadmap:validate", "task docs:validate", "task planning:validate"],
            "evidence_gates": ["closeout evidence required before completion"],
            "expected_closeout_path": milestone.completion_audit or closeout_path_for_milestone(milestone),
            "stop_conditions": ["stop if validation fails", "stop if WR or plan authority is missing"],
            "next_legal_action": "Create or validate the owning WR and implementation plan before execution.",
            "may_create_code": milestone.kind in {"implementation", "hardening"},
            "may_create_crates": False,
            "may_modify_production_behavior": milestone.kind in {"implementation", "hardening"},
            "execution_kind": execution_kind_for_milestone(milestone),
            "permission_classes_required": milestone_permissions(milestone),
            "required_evidence_categories": ["handoff"] if milestone.kind == "release" else ["runtime_test"],
        }
        if owning_wr:
            entry["owning_wr"] = owning_wr
        else:
            entry["future_wr_candidate"] = future_wr_candidate_for_milestone(milestone)
        milestones.append(entry)
    return {
        "version": 1,
        "track_id": track.id,
        "authority_level": "planning_and_sequencing_only",
        "accepted_design_dependencies": accepted_design_dependencies(track),
        "global_forbidden_scope": [
            "no implementation authority from manifest scaffolding alone",
            "no crate creation without exact locked authority",
            "no foundation/meta extraction",
        ],
        "global_validation_commands": ["task production:validate", "task roadmap:validate", "task docs:validate", "task planning:validate"],
        "global_stop_conditions": ["stop if manifest data conflicts with production or roadmap state"],
        "next_legal_action": "Review and complete this scaffold before using it for execution.",
        "ai_executable": False,
        "full_automation_target": False,
        "truth_claims": [
            {
                "claim_id": f"{track.id.lower()}-truth-contract",
                "claim_kind": "product_behavior",
                "claim_level": track.target_completion_quality,
                "claim_status": "blocked",
                "claim_statement": "This scaffold records planning structure only; concrete truth claims require review.",
                "required_docs": [],
                "required_code_contracts": [],
                "required_validations": [],
                "required_closeout_evidence": [],
                "known_gaps": ["Generated scaffold has not been reviewed as an execution authority."],
                "supersedes": [],
                "blocks_downstream": [],
            }
        ],
        "milestones": milestones,
    }


def create_manifest_scaffold(
    track_id: str,
    *,
    production_source: Path = PRODUCTION_SOURCE,
    roadmap_source: Path = ROADMAP_SOURCE,
    manifest_root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> Path:
    planning = load_production_tracks(production_source)
    track = next((candidate for candidate in planning.tracks if candidate.id == track_id), None)
    if track is None:
        raise WorkflowError(f"{track_id}: not present in production tracks source")
    path = manifest_source_path(track_id, root=manifest_root)
    if path.exists():
        raise WorkflowError(f"{track_id}: Track Execution Manifest already exists at {repo_path(path)}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(manifest_scaffold(track, roadmap_source=roadmap_source), sort_keys=False, width=4096), encoding="utf-8", newline="\n")
    return path
