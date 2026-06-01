from __future__ import annotations

from pathlib import Path

import yaml

from roadmap_state import WorkflowError, load_yaml, split_source_paths

from execution.contracts import ActionContract, now_utc_iso
from execution.evidence import load_evidence_records
from execution.planning import (
    active_and_deferred_data,
    workspace_manifest_path,
    workspace_production_source,
    workspace_roadmap_source,
    write_yaml,
)


def validation_result_lines(action: ActionContract, *, workspace_root: Path) -> list[str]:
    evidence_root = workspace_root / "docs-site/src/content/docs/reports/execution-evidence"
    records = load_evidence_records(action.track_id, action.milestone_id, root=evidence_root)
    lines: list[str] = []
    for record in records:
        for command in record.validation_commands:
            if command not in lines:
                lines.append(command)
    return lines


def closeout_markdown(action: ActionContract, *, files_changed: list[str], validation_results: list[str]) -> str:
    evidence_categories = [requirement.kind for requirement in action.closeout_contract.evidence_required]
    validations = [" ".join(command.argv) for command in action.validation_commands]
    result_lines = validation_results or [f"{command}: validation result unavailable" for command in validations]
    return "\n".join(
        [
            "---",
            f"title: {action.milestone_id} Runtime Closeout",
            "status: completed",
            "closeout_evidence:",
            f"  milestone_id: {action.milestone_id}",
            f"  wr_id: {action.wr_id}",
            f"  completion_quality: {action.closeout_contract.completion_quality}",
            "  evidence_categories:",
            *[f"    - {category}" for category in evidence_categories],
            "  validation_commands:",
            *[f"    - {command}" for command in validations],
            "  validation_results:",
            *[f"    - {result!r}" for result in result_lines],
            "  files_changed:",
            *[f"    - {path}" for path in files_changed],
            "  known_gaps: []",
            f"  closeout_path: {action.closeout_contract.path}",
            f"  produced_at: {now_utc_iso()}",
            "---",
            "",
            f"# {action.milestone_id} Runtime Closeout",
            "",
            "The clean Track Execution Harness created this closeout from resolver-backed evidence and declared validation commands.",
            "",
        ]
    )


def archive_wr_item(
    *,
    active_data: dict,
    deferred_data: dict,
    archive_data: dict,
    wr_id: str,
    completion_quality: str,
    closeout_path: str,
) -> None:
    found_item: dict | None = None
    for source_data in (active_data, deferred_data):
        items = source_data.setdefault("items", [])
        for index, item in enumerate(list(items)):
            if item.get("id") == wr_id:
                found_item = dict(item)
                del items[index]
                break
        if found_item is not None:
            break
    if found_item is None:
        raise WorkflowError(f"{wr_id}: runtime closeout could not find owning WR in active or deferred roadmap state")
    found_item["planning_state"] = "completed"
    found_item["completion_quality"] = completion_quality
    found_item["completion_audit"] = closeout_path
    write_scopes = list(found_item.get("write_scopes") or [])
    if closeout_path not in write_scopes:
        write_scopes.append(closeout_path)
    found_item["write_scopes"] = write_scopes
    found_item["current_decision"] = f"Completed by Track Execution Harness runtime closeout for {closeout_path}."
    archive_items = archive_data.setdefault("items", [])
    if not any(item.get("id") == wr_id for item in archive_items):
        archive_items.append(found_item)


def apply_truth_claim_updates(manifest_data: dict, action: ActionContract) -> None:
    if not action.truth_claim_updates:
        return
    claims = manifest_data.get("truth_claims")
    if not isinstance(claims, list):
        raise WorkflowError(f"{action.track_id}: truth_claim_updates require manifest truth_claims")
    claims_by_id = {claim.get("claim_id"): claim for claim in claims if isinstance(claim, dict)}
    allowed_fields = {
        "claim_status",
        "claim_statement",
        "known_gaps",
        "blocks_downstream",
        "supersedes",
        "required_closeout_evidence",
    }
    for update in action.truth_claim_updates:
        claim_id = update.get("claim_id")
        if not isinstance(claim_id, str) or not claim_id.strip():
            raise WorkflowError(f"{action.action_id}: truth_claim_updates entries require claim_id")
        claim = claims_by_id.get(claim_id)
        if claim is None:
            raise WorkflowError(f"{action.action_id}: truth claim {claim_id} is not present in manifest")
        requested_status = update.get("claim_status")
        if requested_status == "satisfied" and action.closeout_contract.completion_quality not in {
            "runtime_proven",
            "proof_slice_runtime_proven",
            "architecture_runtime_proven",
            "perfectionist_verified",
        }:
            raise WorkflowError(
                f"{action.action_id}: cannot satisfy truth claim {claim_id} from "
                f"{action.closeout_contract.completion_quality} closeout"
            )
        for field, value in update.items():
            if field == "claim_id":
                continue
            if field not in allowed_fields:
                raise WorkflowError(f"{action.action_id}: unsupported truth_claim_updates field {field}")
            claim[field] = value


def run_runtime_closeout(action: ActionContract, *, workspace_root: Path) -> list[Path]:
    production_source = workspace_production_source(action, workspace_root)
    roadmap_source = workspace_roadmap_source(action, workspace_root)
    manifest_path = workspace_manifest_path(action, workspace_root)
    closeout_path = workspace_root / action.closeout_contract.path

    production_data = load_yaml(production_source)
    active_data, deferred_source, deferred_data, archive_data = active_and_deferred_data(roadmap_source)
    archive_source, _deferred_source = split_source_paths(roadmap_source)
    if archive_data is None:
        archive_data = {
            "version": active_data.get("version", 1),
            "roadmap": active_data.get("roadmap", {}),
            "items": [],
        }
    manifest_data = yaml.safe_load(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest_data, dict):
        raise WorkflowError(f"{action.manifest_source_path}: manifest must contain a YAML mapping")

    updated_milestone = False
    for track_data in production_data.get("tracks", []):
        if track_data.get("id") != action.track_id:
            continue
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") == action.milestone_id:
                milestone_data["state"] = "completed"
                milestone_data["completion_quality"] = action.closeout_contract.completion_quality
                milestone_data["completion_audit"] = action.closeout_contract.path
                milestone_data["evidence_gates"] = [
                    {
                        "path": action.closeout_contract.path,
                        "required_status": "completed",
                        "reason": f"{action.milestone_id} requires its completed runtime closeout evidence.",
                    }
                ]
                updated_milestone = True
        if track_data.get("milestones") and all(
            milestone.get("state") == "completed" for milestone in track_data.get("milestones", [])
        ):
            track_data["state"] = "completed"
    if not updated_milestone:
        raise WorkflowError(f"{action.milestone_id}: runtime closeout could not update production milestone")

    apply_truth_claim_updates(manifest_data, action)

    for milestone_data in manifest_data.get("milestones", []):
        if milestone_data.get("milestone_id") == action.milestone_id:
            milestone_data["next_legal_action"] = (
                f"{action.milestone_id} closed as {action.closeout_contract.completion_quality}; "
                "recompile the Contract Pack for the next milestone."
            )

    closeout_path.parent.mkdir(parents=True, exist_ok=True)
    closeout_path.write_text(
        closeout_markdown(
            action,
            files_changed=[*action.allowed_outputs, *action.new_outputs],
            validation_results=validation_result_lines(action, workspace_root=workspace_root),
        ),
        encoding="utf-8",
        newline="\n",
    )
    archive_wr_item(
        active_data=active_data,
        deferred_data=deferred_data,
        archive_data=archive_data,
        wr_id=action.wr_id,
        completion_quality=action.closeout_contract.completion_quality,
        closeout_path=action.closeout_contract.path,
    )

    write_yaml(production_source, production_data)
    write_yaml(roadmap_source, active_data)
    write_yaml(deferred_source, deferred_data)
    write_yaml(archive_source, archive_data)
    write_yaml(manifest_path, manifest_data)
    return [production_source, roadmap_source, deferred_source, archive_source, manifest_path, closeout_path]
