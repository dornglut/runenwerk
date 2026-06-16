from __future__ import annotations

from pathlib import Path

import yaml

from production_plan import slugify
from production_state import ProductionPlanningState, load_production_tracks
from roadmap_state import RoadmapState, WorkflowError, combine_roadmap_data, load_roadmap, load_yaml, repo_path, split_source_paths
from track_sources.manifest import load_track_execution_manifest, manifest_write_scope_path
from truth.certificates import (
    certificate_errors_for_claim,
    load_certificate,
    strong_claim_requires_certificate,
    write_certificate,
)
from truth.verifiers import run_verifier

from execution.contracts import ActionContract


def workspace_source(workspace: Path, source_path: str) -> Path:
    candidate = Path(source_path)
    if candidate.is_absolute():
        return workspace / candidate.name if not (workspace / source_path.strip("/")).exists() else workspace / source_path.strip("/")
    return workspace / source_path


def workspace_production_source(action: ActionContract, workspace: Path) -> Path:
    return workspace_source(workspace, action.production_source_path)


def workspace_roadmap_source(action: ActionContract, workspace: Path) -> Path:
    return workspace_source(workspace, action.roadmap_source_path)


def workspace_manifest_path(action: ActionContract, workspace: Path) -> Path:
    if action.manifest_source_path:
        return workspace_source(workspace, action.manifest_source_path)
    return workspace / "docs-site/src/content/docs/workspace/track-execution-manifests" / f"{action.track_id.lower()}.yaml"


def write_yaml(path: Path, data: dict) -> None:
    path.write_text(yaml.safe_dump(data, sort_keys=False, width=4096), encoding="utf-8", newline="\n")


def allocate_next_wr_id(roadmap: RoadmapState) -> str:
    numbers = [int(item.id.split("-")[1]) for item in roadmap.items if item.id.startswith("WR-") and item.id[3:].isdigit()]
    return "WR-001" if not numbers else f"WR-{max(numbers) + 1:03d}"


def manifest_report_path(track_id: str) -> str:
    return f"docs-site/src/content/docs/reports/track-execution-manifests/{track_id.lower()}/manifest.md"


def implementation_plan_path(wr_id: str, milestone_title: str) -> str:
    return (
        "docs-site/src/content/docs/reports/implementation-plans/"
        f"{wr_id.lower()}-{slugify(milestone_title)}/plan.md"
    )


def production_generated_scopes() -> list[str]:
    return [
        "generated: production docs from task production:render",
        "generated: roadmap docs from task roadmap:render",
    ]


def manifest_write_scopes_for_entry(entry) -> list[str]:
    return [
        normalized
        for normalized in (manifest_write_scope_path(scope) for scope in entry.write_scope)
        if normalized is not None
    ]


def exact_planning_expansion_write_scopes(
    *,
    action: ActionContract,
    entry,
    milestone_title: str,
    wr_id: str,
    production_source: Path,
    roadmap_source: Path,
    manifest_path: Path,
    deferred_source: Path,
) -> list[str]:
    scopes = [
        "docs-site/src/content/docs/workspace/production-tracks.yaml",
        "docs-site/src/content/docs/workspace/roadmap-archive.yaml",
        "docs-site/src/content/docs/workspace/roadmap-deferred.yaml",
        f"docs-site/src/content/docs/workspace/track-execution-manifests/{action.track_id.lower()}.yaml",
        manifest_report_path(action.track_id),
        implementation_plan_path(wr_id, milestone_title),
        entry.expected_closeout_path,
        *production_generated_scopes(),
    ]
    scopes.extend(manifest_write_scopes_for_entry(entry))
    return list(dict.fromkeys(scopes))


def active_and_deferred_data(roadmap_source: Path) -> tuple[dict, Path, dict, dict | None]:
    active_data = load_yaml(roadmap_source)
    archive_source, deferred_source = split_source_paths(roadmap_source)
    archive_data = load_yaml(archive_source) if archive_source.exists() else None
    if deferred_source.exists():
        deferred_data = load_yaml(deferred_source)
    else:
        deferred_data = {
            "version": active_data.get("version", 1),
            "roadmap": active_data.get("roadmap", {}),
            "items": [],
        }
    return active_data, deferred_source, deferred_data, archive_data


def roadmap_item_for_planning_expansion(
    *,
    wr_id: str,
    action: ActionContract,
    milestone_title: str,
    predecessor_wr_ids: list[str],
    write_scopes: list[str],
    validation_commands: list[str],
) -> dict:
    return {
        "id": wr_id,
        "title": milestone_title,
        "diagram_title": milestone_title[:48],
        "alias": wr_id.replace("-", ""),
        "lane": "Product planning",
        "dependency_level": "L4",
        "gate": "Track Execution Harness planning expansion gate",
        "planning_state": "blocked_deferred",
        "priority": "P2",
        "value": 4,
        "blocker": 4,
        "tc": 2,
        "rr_oe": 2,
        "du": 2,
        "effort": 5,
        "confidence": 0.5,
        "expected_score": 1.0,
        "rice": "N/A",
        "kano": "Basic",
        "dependencies": predecessor_wr_ids,
        "write_scopes": sorted(set(write_scopes)),
        "validations": validation_commands,
        "next_evidence": f"{action.milestone_id} requires a dedicated production plan and closeout evidence before completion.",
        "current_decision": "Created by the clean Track Execution Harness planning_expansion executor; it authorizes planning metadata, not implementation.",
        "current_call": f"Run task production:plan -- --milestone {action.milestone_id} --roadmap {wr_id}.",
        "first_move": f"Run task production:plan -- --milestone {action.milestone_id} --roadmap {wr_id}.",
        "main_blocker": "Dedicated production plan and closeout evidence are still missing.",
        "why_not_ready": "Track Expansion linked WR authority mechanically; the milestone work itself has not started.",
        "completion_quality": "not_applicable",
        "known_quality_gaps": [],
        "completion_audit": "",
        "diagram_call": ["track expansion", "no implementation"],
        "decision_gates": [],
        "ddd_owner": "workspace governance owns production sequencing for this planning expansion.",
        "adr_requirement": "No new ADR unless the owning design changes architecture or extraction gates.",
        "fitness_function_requirement": "Production, roadmap, docs, and planning validation before closeout.",
        "ownership_mode": "Governance-owned sequencing; no product implementation authority.",
    }


def replace_future_plan_output_paths(paths: list[str], *, future_wr_candidate: str, wr_id: str, milestone_title: str) -> list[str]:
    replacement = implementation_plan_path(wr_id, milestone_title)
    future_key = future_wr_candidate.lower()
    updated: list[str] = []
    for path in paths:
        if "wr-tbd" in path.lower() or future_key in path.lower():
            updated.append(replacement)
        else:
            updated.append(path)
    return list(dict.fromkeys(updated))


def workspace_contract_pack_root(*, contract_pack_root: Path, workspace_root: Path, repo_root: Path) -> Path:
    try:
        relative = contract_pack_root.resolve().relative_to(repo_root.resolve())
        return workspace_root / relative
    except ValueError:
        return workspace_root / contract_pack_root.name


def refresh_existing_contract_packs(
    action: ActionContract,
    *,
    workspace_root: Path,
    repo_root: Path,
    contract_pack_root: Path,
    production_source: Path,
    roadmap_source: Path,
    manifest_root: Path,
) -> None:
    from execution.compiler import compile_contract_pack, write_contract_pack

    pack_root = workspace_contract_pack_root(
        contract_pack_root=contract_pack_root,
        workspace_root=workspace_root,
        repo_root=repo_root,
    )
    if not pack_root.exists():
        return
    for pack_path in sorted(pack_root.glob("*.yaml")):
        data = yaml.safe_load(pack_path.read_text(encoding="utf-8"))
        if not isinstance(data, dict) or not isinstance(data.get("track_id"), str):
            raise WorkflowError(f"{action.action_id}: cannot refresh malformed Contract Pack {repo_path(pack_path)}")
        pack = compile_contract_pack(
            data["track_id"],
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_root=manifest_root,
            contract_pack_root=pack_root,
        )
        write_contract_pack(pack, root=pack_root)


def refresh_stale_truth_certificates(
    *,
    planning: ProductionPlanningState,
    manifest_root: Path,
    workspace_root: Path,
) -> list[Path]:
    cert_root = workspace_root / "docs-site/src/content/docs/reports/truth-certificates"
    written: list[Path] = []
    for track in planning.tracks:
        loaded = load_track_execution_manifest(track.id, root=manifest_root)
        if loaded is None:
            continue
        for claim in loaded.manifest.truth_claims:
            if not strong_claim_requires_certificate(claim):
                continue
            if load_certificate(track.id, claim.claim_id, root=cert_root) is None:
                continue
            errors = certificate_errors_for_claim(
                track.id,
                claim,
                root=cert_root,
                repo_root=workspace_root,
            )
            if not errors:
                continue
            verifier = claim.truth_verifier
            if not verifier:
                raise WorkflowError(f"{track.id}: truth claim {claim.claim_id} cannot be refreshed without truth_verifier")
            certificate = run_verifier(
                track_id=track.id,
                claim_id=claim.claim_id,
                verifier=verifier,
                repo_root=workspace_root,
            )
            if certificate.status != "passed":
                raise WorkflowError(
                    f"{track.id}: truth verifier {verifier} failed for {claim.claim_id}; "
                    "planning expansion cannot refresh a non-passing certificate"
                )
            written.append(write_certificate(certificate, root=cert_root))
    return written


def run_planning_expansion(
    action: ActionContract,
    *,
    workspace_root: Path,
    repo_root: Path,
    contract_pack_root: Path,
) -> list[Path]:
    production_source = workspace_production_source(action, workspace_root)
    roadmap_source = workspace_roadmap_source(action, workspace_root)
    manifest_path = workspace_manifest_path(action, workspace_root)

    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    loaded = load_track_execution_manifest(action.track_id, root=manifest_path.parent)
    if loaded is None:
        raise WorkflowError(f"{action.track_id}: missing Track Execution Manifest in action workspace")
    track = next((candidate for candidate in planning.tracks if candidate.id == action.track_id), None)
    if track is None:
        raise WorkflowError(f"{action.track_id}: not present in production source")
    milestone = next((candidate for candidate in track.milestones if candidate.id == action.milestone_id), None)
    if milestone is None:
        raise WorkflowError(f"{action.milestone_id}: not present in production track {action.track_id}")
    entry = loaded.manifest.by_milestone_id.get(action.milestone_id)
    if entry is None:
        raise WorkflowError(f"{action.milestone_id}: not present in Track Execution Manifest")
    if entry.owning_wr:
        raise WorkflowError(f"{action.milestone_id}: already owns WR {entry.owning_wr}")
    if not entry.future_wr_candidate:
        raise WorkflowError(f"{action.milestone_id}: planning_expansion requires future_wr_candidate")
    if milestone.roadmap_links:
        raise WorkflowError(f"{action.milestone_id}: production milestone already links WR rows {milestone.roadmap_links}")

    wr_id = allocate_next_wr_id(roadmap)
    production_data = load_yaml(production_source)
    for track_data in production_data.get("tracks", []):
        if track_data.get("id") != action.track_id:
            continue
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") == action.milestone_id:
                milestone_data["roadmap_links"] = [wr_id]

    active_data, deferred_source, deferred_data, archive_data = active_and_deferred_data(roadmap_source)
    if any(item.get("id") == wr_id for item in deferred_data.get("items", [])) or roadmap.by_id.get(wr_id):
        raise WorkflowError(f"{wr_id}: already present in roadmap state")

    predecessor_wr_ids: list[str] = []
    by_milestone = {candidate.id: candidate for candidate in track.milestones}
    for predecessor in entry.predecessor_dependencies:
        predecessor_milestone = by_milestone.get(predecessor)
        if predecessor_milestone is not None:
            predecessor_wr_ids.extend(predecessor_milestone.roadmap_links)
    deferred_data.setdefault("items", []).append(
        roadmap_item_for_planning_expansion(
            wr_id=wr_id,
            action=action,
            milestone_title=milestone.title,
            predecessor_wr_ids=list(dict.fromkeys(predecessor_wr_ids)),
            write_scopes=exact_planning_expansion_write_scopes(
                action=action,
                entry=entry,
                milestone_title=milestone.title,
                wr_id=wr_id,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_path=loaded.path,
                deferred_source=deferred_source,
            ),
            validation_commands=[" ".join(command.argv) for command in action.validation_commands],
        )
    )

    manifest_data = loaded.manifest.model_dump(mode="json", exclude_none=True)
    for milestone_data in manifest_data["milestones"]:
        if milestone_data["milestone_id"] == action.milestone_id:
            milestone_data.pop("future_wr_candidate", None)
            milestone_data["owning_wr"] = wr_id
            milestone_data["next_legal_action"] = (
                f"Run task production:plan -- --milestone {action.milestone_id} --roadmap {wr_id}; "
                "stop before implementation until the bounded contract is accepted."
            )
            design_contract = milestone_data.get("agent_design_contract")
            if isinstance(design_contract, dict):
                output_paths = design_contract.get("expected_output_paths")
                if isinstance(output_paths, list):
                    design_contract["expected_output_paths"] = replace_future_plan_output_paths(
                        [path for path in output_paths if isinstance(path, str)],
                        future_wr_candidate=entry.future_wr_candidate,
                        wr_id=wr_id,
                        milestone_title=milestone.title,
                    )

    ProductionPlanningState.model_validate(production_data)
    RoadmapState.model_validate(combine_roadmap_data(active_data, roadmap_source, archive_data=archive_data, deferred_data=deferred_data))
    write_yaml(production_source, production_data)
    write_yaml(deferred_source, deferred_data)
    write_yaml(loaded.path, manifest_data)
    refresh_existing_contract_packs(
        action,
        workspace_root=workspace_root,
        repo_root=repo_root,
        contract_pack_root=contract_pack_root,
        production_source=production_source,
        roadmap_source=roadmap_source,
        manifest_root=manifest_path.parent,
    )
    refreshed_certificates = refresh_stale_truth_certificates(
        planning=load_production_tracks(production_source),
        manifest_root=manifest_path.parent,
        workspace_root=workspace_root,
    )
    return [production_source, deferred_source, loaded.path, *refreshed_certificates]
