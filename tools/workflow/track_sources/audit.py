from __future__ import annotations

import re
import subprocess
from pathlib import Path
from typing import Iterable

import yaml

from production_plan import ProductionPlanContext, classify_plan_action
from production_state import (
    ProductionMilestone,
    ProductionPlanningState,
    ProductionTrack,
)
from roadmap_state import (
    REPO_ROOT,
    WorkflowError,
    RoadmapItem,
    RoadmapState,
    document_frontmatter_status,
    is_new_write_scope,
    normalize_repo_path,
    normalize_write_scope_path,
    normalized_write_scopes_with_generated_outputs,
    path_within_scope,
    repo_path,
)
from execution.contracts import validation_command_from_string
from track_sources.manifest import (
    FULL_AUTOMATION_EXECUTION_KINDS,
    FULL_AUTOMATION_PERMISSION_GRANTS,
    FULL_TRACK_PERMISSION_SET,
    MANIFEST_AUDIT_CATEGORY_ORDER,
    PROOF_AGGREGATION_REQUIRED_EVIDENCE_CATEGORIES,
    LoadedTrackExecutionManifest,
    ManifestCloseoutEvidenceRecord,
    ManifestImplementationWriter,
    ManifestTruthClaim,
    ManifestTruthEvidence,
    TrackExecutionManifest,
    TrackExecutionManifestMilestone,
    agent_design_contract_for_entry,
    implementation_writer_allowed_scopes,
    implementation_writer_forbidden_scopes,
    implementation_writer_output_scopes,
    is_generated_or_derived_scope,
    manifest_design_dependency_errors,
    manifest_write_scope_path,
    mentions_generated_or_derived_scope,
    normalize_evidence_category,
    product_forbidden_scopes_for_entry,
    product_implementation_scopes_for_entry,
    product_validation_commands_for_entry,
)


def ordered_track_milestones(track: ProductionTrack) -> list[ProductionMilestone]:
    by_id = {milestone.id: milestone for milestone in track.milestones}
    ordered: list[ProductionMilestone] = []
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(milestone: ProductionMilestone) -> None:
        if milestone.id in visited:
            return
        if milestone.id in visiting:
            raise WorkflowError(f"{track.id}: milestone dependency cycle includes {milestone.id}")
        visiting.add(milestone.id)
        for dependency in milestone.dependencies:
            dependency_milestone = by_id.get(dependency)
            if dependency_milestone is not None:
                visit(dependency_milestone)
        visiting.remove(milestone.id)
        visited.add(milestone.id)
        ordered.append(milestone)

    for milestone in track.milestones:
        visit(milestone)
    return ordered


def manifest_type_errors(entry: TrackExecutionManifestMilestone, *, milestone_kind: str) -> list[str]:
    if milestone_kind == "design" and entry.milestone_type not in {"docs_only", "design_only"}:
        return [f"manifest milestone_type {entry.milestone_type!r} does not match design milestone kind"]
    if milestone_kind == "implementation" and entry.milestone_type != "implementation":
        return [f"manifest milestone_type {entry.milestone_type!r} does not match implementation milestone kind"]
    if milestone_kind == "hardening" and entry.milestone_type != "hardening":
        return [f"manifest milestone_type {entry.milestone_type!r} does not match hardening milestone kind"]
    if milestone_kind == "release" and entry.milestone_type != "closeout":
        return [f"manifest milestone_type {entry.milestone_type!r} does not match release milestone kind"]
    return []


def manifest_alignment_errors(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
    ordered_milestone_ids: list[str],
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    manifest = loaded.manifest
    errors: list[str] = []
    if manifest.track_id != track.id:
        errors.append(f"{repo_path(loaded.path)}: track_id {manifest.track_id} does not match production track {track.id}")

    manifest_milestone_ids = [entry.milestone_id for entry in manifest.milestones]
    if manifest_milestone_ids != ordered_milestone_ids:
        errors.append(
            f"{track.id}: manifest milestone order {manifest_milestone_ids} "
            f"does not match production dependency order {ordered_milestone_ids}"
        )

    errors.extend(manifest_design_dependency_errors(manifest, repo_root=repo_root))

    production_by_id = {milestone.id: milestone for milestone in track.milestones}
    for entry in manifest.milestones:
        milestone = production_by_id.get(entry.milestone_id)
        if milestone is None:
            errors.append(f"{entry.milestone_id}: manifest milestone is not present in production track {track.id}")
            continue
        if entry.title != milestone.title:
            errors.append(
                f"{entry.milestone_id}: manifest title {entry.title!r} "
                f"does not match production title {milestone.title!r}"
            )
        if entry.predecessor_dependencies != milestone.dependencies:
            errors.append(
                f"{entry.milestone_id}: manifest dependencies {entry.predecessor_dependencies} "
                f"do not match production dependencies {milestone.dependencies}"
            )
        errors.extend(f"{entry.milestone_id}: {error}" for error in manifest_type_errors(entry, milestone_kind=milestone.kind))
        if entry.owning_wr:
            if milestone.roadmap_links != [entry.owning_wr]:
                errors.append(
                    f"{entry.milestone_id}: manifest owning_wr {entry.owning_wr} "
                    f"does not match production roadmap_links {milestone.roadmap_links}"
                )
            roadmap_item = roadmap.by_id.get(entry.owning_wr)
            if roadmap_item is None:
                errors.append(f"{entry.milestone_id}: manifest owning_wr {entry.owning_wr} is not present in roadmap")
            else:
                errors.extend(manifest_write_scope_coverage_errors(entry, roadmap_item))
                errors.extend(implementation_writer_write_scope_coverage_errors(entry, roadmap_item))
        if entry.future_wr_candidate and milestone.roadmap_links:
            errors.append(
                f"{entry.milestone_id}: manifest future_wr_candidate {entry.future_wr_candidate} "
                f"conflicts with production roadmap_links {milestone.roadmap_links}"
            )
    return errors


def manifest_write_scope_coverage_errors(
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
) -> list[str]:
    errors: list[str] = []
    wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
    for scope in entry.write_scope:
        if is_generated_or_derived_scope(scope):
            continue
        if mentions_generated_or_derived_scope(scope):
            errors.append(
                f"{entry.milestone_id}: manifest write_scope {scope!r} must use "
                "'generated:' or 'derived:' when it names generated/derived output"
            )
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            continue
        if not any(path_within_scope(normalized, wr_scope) for wr_scope in wr_scopes):
            errors.append(
                f"{entry.milestone_id}: manifest write_scope {normalized} is not covered by "
                f"owning WR {roadmap_item.id} write_scopes"
            )
    return errors


def implementation_writer_write_scope_coverage_errors(
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
) -> list[str]:
    writer = entry.implementation_writer
    if writer is None or writer.strategy == "no_writer":
        return []
    errors: list[str] = []
    wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
    for scope in implementation_writer_allowed_scopes(writer):
        if is_generated_or_derived_scope(scope):
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            continue
        if not any(path_within_scope(normalized, wr_scope) for wr_scope in wr_scopes):
            errors.append(
                f"{entry.milestone_id}: implementation_writer allowed scope {normalized} is not covered by "
                f"owning WR {roadmap_item.id} write_scopes"
            )
    return errors


def document_frontmatter(path: Path) -> dict | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = next(index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---")
    except StopIteration:
        return None
    frontmatter = yaml.safe_load("\n".join(lines[1:end])) or {}
    return frontmatter if isinstance(frontmatter, dict) else None


def manifest_path_reference(path: str) -> Path:
    candidate = Path(path)
    if candidate.is_absolute():
        return candidate
    return REPO_ROOT / normalize_repo_path(path)


def closeout_evidence_record(path: Path) -> ManifestCloseoutEvidenceRecord | None:
    frontmatter = document_frontmatter(path)
    if frontmatter is None:
        return None
    evidence = frontmatter.get("closeout_evidence")
    if evidence is None:
        return None
    if not isinstance(evidence, dict):
        raise ValueError("closeout_evidence frontmatter must be a mapping")
    return ManifestCloseoutEvidenceRecord.model_validate(evidence)


def audit_manifest_truth_claims(manifest: TrackExecutionManifest, track: ProductionTrack) -> list[str]:
    errors: list[str] = []
    if not manifest.truth_claims:
        return [f"{manifest.track_id}: manifest-backed tracks must declare truth_claims"]
    claim_ids = [claim.claim_id for claim in manifest.truth_claims]
    duplicates = sorted({claim_id for claim_id in claim_ids if claim_ids.count(claim_id) > 1})
    if duplicates:
        errors.append(f"{manifest.track_id}: truth_claims duplicate claim_id values: {', '.join(duplicates)}")
    for claim in manifest.truth_claims:
        errors.extend(truth_claim_errors(manifest, claim, track=track))
    errors.extend(production_truth_claim_alignment_errors(manifest, track))
    return errors


def truth_claim_errors(
    manifest: TrackExecutionManifest,
    claim: ManifestTruthClaim,
    *,
    track: ProductionTrack,
) -> list[str]:
    errors: list[str] = []
    if claim.claim_status == "satisfied":
        for evidence in (
            claim.required_docs
            + claim.required_code_contracts
            + claim.required_validations
            + claim.required_closeout_evidence
        ):
            errors.extend(truth_evidence_errors(manifest, claim, evidence))
    if claim.claim_status != "satisfied":
        for downstream in claim.blocks_downstream:
            downstream_milestone = next((milestone for milestone in track.milestones if milestone.id == downstream), None)
            if downstream_milestone and downstream_milestone.state in {"ready_next", "active", "completed"}:
                errors.append(
                    f"{manifest.track_id}: truth claim {claim.claim_id} blocks downstream {downstream}, "
                    f"but production milestone state is {downstream_milestone.state}"
                )
    return errors


def truth_evidence_errors(
    manifest: TrackExecutionManifest,
    claim: ManifestTruthClaim,
    evidence: ManifestTruthEvidence,
) -> list[str]:
    label = f"{manifest.track_id}: truth claim {claim.claim_id}"
    if evidence.evidence_kind == "doc_exists":
        assert evidence.path is not None
        path = manifest_path_reference(evidence.path)
        return [] if path.exists() else [f"{label} requires document {evidence.path} ({evidence.reason})"]
    if evidence.evidence_kind == "doc_frontmatter_status":
        assert evidence.path is not None
        assert evidence.required_status is not None
        path = manifest_path_reference(evidence.path)
        if not path.exists():
            return [f"{label} requires document {evidence.path} ({evidence.reason})"]
        status = document_frontmatter_status(path)
        if status is None:
            return [f"{label} requires document {evidence.path} with frontmatter status ({evidence.reason})"]
        if status.lower() != evidence.required_status.lower():
            return [
                f"{label} requires document {evidence.path} status {evidence.required_status!r}, "
                f"got {status!r} ({evidence.reason})"
            ]
        return []
    if evidence.evidence_kind == "module_path_exists":
        assert evidence.path is not None
        path = manifest_path_reference(evidence.path)
        return [] if path.exists() else [f"{label} requires module path {evidence.path} ({evidence.reason})"]
    if evidence.evidence_kind == "rust_symbol_exists":
        assert evidence.path is not None
        assert evidence.symbol is not None
        path = manifest_path_reference(evidence.path)
        if not path.exists():
            return [f"{label} requires Rust source {evidence.path} ({evidence.reason})"]
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as error:
            return [f"{label} cannot read Rust source {evidence.path}: {error}"]
        symbol = re.escape(evidence.symbol)
        if not re.search(rf"\b(?:struct|enum|trait|type|fn|mod)\s+{symbol}\b", text):
            return [f"{label} requires Rust symbol {evidence.symbol} in {evidence.path} ({evidence.reason})"]
        return []
    if evidence.evidence_kind == "validation_command":
        assert evidence.command is not None
        return validation_command_errors([evidence.command], label=f"{label} validation", product_code_eligible=True)
    if evidence.evidence_kind == "closeout_evidence_category":
        assert evidence.closeout_path is not None
        assert evidence.category is not None
        path = manifest_path_reference(evidence.closeout_path)
        if not path.exists():
            return [f"{label} requires closeout evidence {evidence.closeout_path} ({evidence.reason})"]
        try:
            record = closeout_evidence_record(path)
        except ValueError as error:
            return [f"{label} closeout evidence metadata is invalid in {evidence.closeout_path}: {error}"]
        if record is None:
            return [f"{label} requires closeout_evidence metadata in {evidence.closeout_path} ({evidence.reason})"]
        required_category = normalize_evidence_category(evidence.category)
        available_categories = {normalize_evidence_category(category) for category in record.evidence_categories}
        if required_category not in available_categories:
            return [
                f"{label} requires closeout evidence category {required_category} in "
                f"{evidence.closeout_path} ({evidence.reason})"
            ]
        return []
    return [f"{label} has unsupported truth evidence kind {evidence.evidence_kind}"]


def production_truth_claim_alignment_errors(manifest: TrackExecutionManifest, track: ProductionTrack) -> list[str]:
    errors: list[str] = []
    satisfied = {(claim.claim_kind, claim.claim_level) for claim in manifest.truth_claims if claim.claim_status == "satisfied"}
    blocked = {(claim.claim_kind, claim.claim_level) for claim in manifest.truth_claims if claim.claim_status == "blocked"}
    if track.target_completion_quality == "proof_slice_runtime_proven" and (
        "proof_slice",
        "proof_slice_runtime_proven",
    ) not in satisfied:
        errors.append(
            f"{track.id}: target_completion_quality proof_slice_runtime_proven requires a satisfied "
            "proof_slice truth claim at proof_slice_runtime_proven"
        )
    if track.target_completion_quality == "architecture_runtime_proven":
        architecture_key = ("architecture_contract", "architecture_runtime_proven")
        if architecture_key not in satisfied:
            if track.state == "completed":
                errors.append(
                    f"{track.id}: target_completion_quality architecture_runtime_proven requires a satisfied "
                    "architecture_contract truth claim at architecture_runtime_proven"
                )
            elif architecture_key not in blocked:
                errors.append(
                    f"{track.id}: active target_completion_quality architecture_runtime_proven requires either a "
                    "satisfied or blocked architecture_contract truth claim at architecture_runtime_proven"
                )
            else:
                for text in [track.strategic_goal, *track.success_criteria]:
                    normalized = text.lower()
                    if any(
                        phrase in normalized
                        for phrase in (
                            "architecture is implemented",
                            "architecture exists",
                            "architecture is proven",
                            "unblocks materialprogram",
                        )
                    ):
                        errors.append(
                            f"{track.id}: production wording claims architecture truth while the "
                            f"architecture_runtime_proven truth claim is blocked: {text}"
                        )
    if track.target_completion_quality == "proof_slice_runtime_proven":
        for text in [track.strategic_goal, *track.success_criteria]:
            normalized = text.lower()
            proof_slice_language = "proof-slice" in normalized or "proof slice" in normalized or "bounded" in normalized
            if ("is proven" in normalized or "proves " in normalized or normalized.startswith("prove ")) and not proof_slice_language:
                errors.append(f"{track.id}: production wording claims stronger truth than proof_slice_runtime_proven: {text}")
            if "enables the materialprogram" in normalized or "enables materialprogram" in normalized:
                errors.append(f"{track.id}: production wording enables MaterialProgram despite corrected truth claims: {text}")
    return errors


def completed_expected_output_path_errors(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
) -> list[str]:
    if milestone.state != "completed":
        return []
    if milestone.kind != "design" and entry.milestone_type not in {"docs_only", "design_only"}:
        return []
    contract = agent_design_contract_for_entry(entry)
    if contract is None:
        return []
    errors: list[str] = []
    for output_path in contract.expected_output_paths:
        if is_generated_or_derived_scope(output_path):
            continue
        normalized = manifest_write_scope_path(output_path)
        if normalized is None:
            errors.append(f"{entry.milestone_id}: completed design expected_output_paths includes non-path output {output_path}")
            continue
        if not (REPO_ROOT / normalized).exists():
            errors.append(f"{entry.milestone_id}: completed design expected_output_path is missing: {normalized}")
    return errors


def audit_manifest(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
) -> list[str]:
    errors = manifest_alignment_errors(
        loaded,
        track=track,
        roadmap=roadmap,
        ordered_milestone_ids=[milestone.id for milestone in ordered_track_milestones(track)],
    )
    manifest = loaded.manifest
    if not manifest.accepted_design_dependencies:
        errors.append(f"{manifest.track_id}: manifest must list accepted design dependencies")
    errors.extend(audit_manifest_truth_claims(manifest, track))
    for value in manifest.global_forbidden_scope + manifest.global_validation_commands + manifest.global_stop_conditions:
        if value.startswith("blocked:"):
            errors.append(f"{manifest.track_id}: global manifest field remains blocked: {value}")
    for entry in manifest.milestones:
        errors.extend(audit_manifest_milestone(entry))
        milestone = next((candidate for candidate in track.milestones if candidate.id == entry.milestone_id), None)
        if milestone is not None:
            errors.extend(completed_expected_output_path_errors(entry, milestone))
            errors.extend(audit_manifest_action_contracts(entry, milestone, track=track, manifest=manifest))
    return errors


def manifest_audit_error_category(error: str) -> str:
    if "truth claim" in error or "truth_claim" in error or "claims stronger truth" in error:
        return "truth claim errors"
    if "manifest write_scope" in error or "implementation_writer allowed scope" in error:
        return "WR scope mismatch"
    if "remains blocked" in error:
        return "invalid blocked fields"
    if "expected_closeout_path" in error or "closeout/report" in error:
        return "invalid closeout path"
    if "design dependency" in error or "gate" in error:
        return "missing gates"
    if (
        "does not match" in error
        or "conflicts with production" in error
        or "not present in production track" in error
        or "manifest milestone order" in error
        or "manifest title" in error
        or "manifest dependencies" in error
    ):
        return "alignment errors"
    if "owning_wr" in error or "future_wr_candidate" in error or "owning WR" in error:
        return "missing WR authority"
    return "other manifest audit blockers"


def grouped_manifest_audit_errors(errors: list[str]) -> dict[str, list[str]]:
    grouped = {category: [] for category in MANIFEST_AUDIT_CATEGORY_ORDER}
    for error in errors:
        grouped[manifest_audit_error_category(error)].append(error)
    return {category: grouped[category] for category in MANIFEST_AUDIT_CATEGORY_ORDER if grouped[category]}


def manifest_audit_blocker_lines(errors: list[str]) -> list[str]:
    lines = ["Track Execution Manifest audit blockers:"]
    for category, category_errors in grouped_manifest_audit_errors(errors).items():
        lines.append(f"{category}:")
        lines.extend(f"- {error}" for error in category_errors)
    return lines


def audit_manifest_or_raise(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
) -> None:
    errors = audit_manifest(loaded, track=track, roadmap=roadmap)
    if errors:
        raise WorkflowError("\n".join(manifest_audit_blocker_lines(errors)))


def remaining_manifest_entries(
    manifest: TrackExecutionManifest,
    track: ProductionTrack,
) -> list[tuple[TrackExecutionManifestMilestone, ProductionMilestone, int]]:
    remaining: list[tuple[TrackExecutionManifestMilestone, ProductionMilestone, int]] = []
    manifest_by_id = manifest.by_milestone_id
    for index, milestone in enumerate(ordered_track_milestones(track)):
        if milestone.state == "completed":
            continue
        remaining.append((manifest_by_id[milestone.id], milestone, index))
    return remaining


def execution_kind_compatibility_errors(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
) -> list[str]:
    execution_kind = entry.execution_kind
    if execution_kind is None:
        return [f"{entry.milestone_id}: execution_kind is required for full automation"]
    expected = {
        "design_contract": ({"docs_only", "design_only"}, {"design"}),
        "implementation_proof": ({"implementation", "hardening"}, {"implementation", "hardening"}),
        "proof_aggregation": ({"hardening"}, {"hardening"}),
        "handoff_closeout": ({"closeout"}, {"release"}),
        "extraction_gate": ({"closeout", "hardening"}, {"release", "hardening"}),
    }
    manifest_types, production_kinds = expected[execution_kind]
    errors: list[str] = []
    if entry.milestone_type not in manifest_types:
        errors.append(
            f"{entry.milestone_id}: execution_kind {execution_kind} is incompatible with "
            f"manifest milestone_type {entry.milestone_type}"
        )
    if milestone.kind not in production_kinds:
        errors.append(
            f"{entry.milestone_id}: execution_kind {execution_kind} is incompatible with "
            f"production milestone kind {milestone.kind}"
        )
    return errors


def full_automation_preflight_errors(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
    allow: set[str] | None = None,
) -> list[str]:
    errors = audit_manifest(loaded, track=track, roadmap=roadmap)
    if errors:
        return errors
    for entry, milestone, index in remaining_manifest_entries(loaded.manifest, track):
        errors.extend(full_automation_entry_errors(entry, milestone, track_index=index, allow=allow))
    return errors


def full_automation_entry_errors(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track_index: int,
    allow: set[str] | None,
) -> list[str]:
    errors: list[str] = []
    execution_kind = entry.execution_kind
    if execution_kind not in FULL_AUTOMATION_EXECUTION_KINDS:
        errors.append(
            f"{entry.milestone_id}: full automation execution_kind must be one of "
            f"{', '.join(sorted(FULL_AUTOMATION_EXECUTION_KINDS))}; got {execution_kind or 'missing'}"
        )
    else:
        errors.extend(execution_kind_compatibility_errors(entry, milestone))

    required_permissions = full_automation_required_permission_classes(entry)
    declared_permissions = set(entry.permission_classes_required)
    if not declared_permissions:
        errors.append(f"{entry.milestone_id}: full automation requires declared permission_classes_required")
    missing_declared = sorted(required_permissions - declared_permissions)
    if missing_declared:
        errors.append(f"{entry.milestone_id}: permission_classes_required missing " + ", ".join(missing_declared))
    if allow is not None:
        ungranted = sorted(permission for permission in required_permissions if not full_automation_permission_granted(permission, allow))
        if ungranted:
            errors.append(f"{entry.milestone_id}: full automation requires ungranted permissions " + ", ".join(ungranted))
    if "foundation_extraction" in declared_permissions:
        errors.append(f"{entry.milestone_id}: full automation must not require foundation_extraction")
    if entry.may_create_crates or "crate_creation" in declared_permissions:
        if allow is None or not full_automation_permission_granted("crate_creation", allow):
            errors.append(f"{entry.milestone_id}: full automation requires crate_creation permission for exact crate outputs")
        errors.extend(full_automation_crate_creation_errors(entry))

    errors.extend(full_automation_contract_errors(entry, execution_kind=execution_kind or ""))
    errors.extend(full_automation_scope_errors(entry))
    errors.extend(validation_command_errors(entry.validation_commands, label=f"{entry.milestone_id}: manifest validation_commands", product_code_eligible=True))
    if not entry.required_evidence_categories:
        errors.append(f"{entry.milestone_id}: full automation requires declared evidence categories")
    if track_index > 0 and not entry.predecessor_dependencies:
        errors.append(f"{entry.milestone_id}: full automation requires predecessor dependencies")
    if entry.expected_closeout_path.startswith("blocked:") or not entry.expected_closeout_path.endswith(".md"):
        errors.append(f"{entry.milestone_id}: full automation requires a declared Markdown closeout path")
    if milestone.kind == "release" and execution_kind == "handoff_closeout":
        errors.extend(full_automation_handoff_errors(entry))
    return errors


def full_automation_required_permission_classes(entry: TrackExecutionManifestMilestone) -> set[str]:
    required: set[str] = set()
    if entry.future_wr_candidate:
        required.add("auto_safe")
    execution_kind = entry.execution_kind
    if execution_kind == "design_contract":
        required.update({"agent_design", "agent_closeout"})
    elif execution_kind == "implementation_proof":
        required.update({"agent_design", "product_code", "product_implementation", "runtime_closeout"})
    elif execution_kind == "proof_aggregation":
        required.update({"agent_design", "product_code", "product_implementation", "runtime_closeout"})
    elif execution_kind == "handoff_closeout":
        required.update({"agent_closeout", "handoff"})
    elif execution_kind == "extraction_gate":
        required.update({"agent_design", "agent_closeout"})
    return required


def full_automation_permission_granted(permission: str, allow: set[str]) -> bool:
    grants = FULL_AUTOMATION_PERMISSION_GRANTS.get(permission, {permission})
    return bool(grants & allow)


def full_automation_contract_errors(
    entry: TrackExecutionManifestMilestone,
    *,
    execution_kind: str,
) -> list[str]:
    errors: list[str] = []
    expected_closeout_strategy = {
        "design_contract": "bounded_contract_closeout",
        "implementation_proof": "runtime_proven_closeout",
        "proof_aggregation": "runtime_proven_closeout",
        "handoff_closeout": "handoff_closeout",
        "extraction_gate": "extraction_gate_closeout",
    }.get(execution_kind)
    if expected_closeout_strategy is not None:
        if entry.closeout_strategy is None:
            errors.append(f"{entry.milestone_id}: full automation requires manifest-declared closeout_strategy")
        elif entry.closeout_strategy != expected_closeout_strategy:
            errors.append(
                f"{entry.milestone_id}: closeout_strategy {entry.closeout_strategy} "
                f"does not match execution_kind {execution_kind}; expected {expected_closeout_strategy}"
            )
    if execution_kind == "design_contract":
        if agent_design_contract_for_entry(entry) is None:
            errors.append(f"{entry.milestone_id}: full automation design_contract requires agent_design_contract")
        if entry.agent_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: full automation design_contract requires agent_closeout_contract")
    elif execution_kind in {"implementation_proof", "proof_aggregation"}:
        if agent_design_contract_for_entry(entry) is None:
            errors.append(f"{entry.milestone_id}: full automation implementation milestone requires agent_design_contract")
        if entry.product_code_contract is None:
            errors.append(f"{entry.milestone_id}: full automation implementation milestone requires product_code_contract")
        if entry.runtime_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: full automation implementation milestone requires runtime_closeout_contract")
        if entry.implementation_writer is None:
            errors.append(f"{entry.milestone_id}: full automation implementation milestone requires implementation_writer")
        elif entry.implementation_writer.strategy == "no_writer":
            errors.append(f"{entry.milestone_id}: implementation_writer.strategy must not be no_writer for full automation")
        if execution_kind == "proof_aggregation":
            writer = entry.implementation_writer
            if writer is None or writer.strategy != "proof_aggregation_writer":
                errors.append(f"{entry.milestone_id}: proof_aggregation milestone requires proof_aggregation_writer")
    elif execution_kind == "handoff_closeout":
        if entry.agent_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: full automation handoff_closeout requires agent_closeout_contract")
        if entry.handoff_contract is None:
            errors.append(f"{entry.milestone_id}: full automation handoff_closeout requires handoff_contract")
        if entry.product_code_contract is not None or entry.implementation_writer is not None:
            errors.append(f"{entry.milestone_id}: handoff_closeout must not declare product implementation contracts")
    return errors


def full_automation_scope_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    contract = agent_design_contract_for_entry(entry)
    if contract is not None:
        errors.extend(exact_scope_list_errors(contract.allowed_write_scopes, label=f"{entry.milestone_id}: agent_design_contract allowed_write_scopes"))
        errors.extend(validation_command_errors(contract.validation_commands, label=f"{entry.milestone_id}: agent_design_contract", product_code_eligible=True))
    if entry.product_code_contract is not None:
        product_scopes = entry.product_code_contract.exact_allowed_implementation_write_scopes
        errors.extend(exact_scope_list_errors(product_scopes, label=f"{entry.milestone_id}: product_code_contract exact_allowed_implementation_write_scopes"))
        errors.extend(new_file_scope_errors(entry.milestone_id, product_scopes, label="product_code_contract"))
        errors.extend(validation_command_errors(entry.product_code_contract.validation_commands, label=f"{entry.milestone_id}: product_code_contract", product_code_eligible=True))
        if not entry.product_code_contract.forbidden_implementation_scopes:
            errors.append(f"{entry.milestone_id}: product_code_contract must declare forbidden implementation scopes")
    if entry.implementation_writer is not None and entry.implementation_writer.strategy != "no_writer":
        writer_scopes = implementation_writer_allowed_scopes(entry.implementation_writer)
        errors.extend(exact_scope_list_errors(writer_scopes, label=f"{entry.milestone_id}: implementation_writer allowed scope"))
        errors.extend(new_file_scope_errors(entry.milestone_id, writer_scopes, label="implementation_writer"))
        if not implementation_writer_forbidden_scopes(entry.implementation_writer):
            errors.append(f"{entry.milestone_id}: implementation_writer must declare forbidden scopes")
        errors.extend(validation_command_errors(entry.implementation_writer.validation_commands, label=f"{entry.milestone_id}: implementation_writer", product_code_eligible=True))
    if entry.runtime_closeout_contract is not None:
        if not entry.runtime_closeout_contract.runtime_test_evidence_required:
            errors.append(f"{entry.milestone_id}: runtime_closeout_contract must declare runtime evidence")
        if entry.runtime_closeout_contract.closeout_path != entry.expected_closeout_path:
            errors.append(f"{entry.milestone_id}: runtime_closeout_contract closeout_path must match expected_closeout_path")
        errors.extend(validation_command_errors(entry.runtime_closeout_contract.validation_commands, label=f"{entry.milestone_id}: runtime_closeout_contract", product_code_eligible=True))
    if entry.agent_closeout_contract is not None:
        if entry.agent_closeout_contract.closeout_path != entry.expected_closeout_path:
            errors.append(f"{entry.milestone_id}: agent_closeout_contract closeout_path must match expected_closeout_path")
        errors.extend(validation_command_errors(entry.agent_closeout_contract.validation_commands, label=f"{entry.milestone_id}: agent_closeout_contract", product_code_eligible=True))
    if entry.handoff_contract is not None:
        errors.extend(validation_command_errors(entry.handoff_contract.validation_commands, label=f"{entry.milestone_id}: handoff_contract", product_code_eligible=True))
    return errors


def validation_command_errors(commands: Iterable[str], *, label: str, product_code_eligible: bool) -> list[str]:
    del product_code_eligible
    errors: list[str] = []
    for command in commands:
        parsed = validation_command_from_string(command)
        if parsed.get("command_id") == "blocked":
            errors.append(f"{label}: validation command {command!r} is invalid: {parsed.get('blocked_reason')}")
    return errors


def exact_scope_list_errors(scopes: list[str], *, label: str) -> list[str]:
    errors: list[str] = []
    broad_roots = {
        ".",
        "apps",
        "domain",
        "engine",
        "foundation",
        "src",
        "tools",
        "docs-site",
        "docs-site/src",
        "docs-site/src/content",
        "docs-site/src/content/docs",
    }
    for scope in scopes:
        if is_generated_or_derived_scope(scope):
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            errors.append(f"{label} includes ambiguous or non-path scope: {scope}")
            continue
        if "*" in normalized or "..." in normalized:
            errors.append(f"{label} must not use wildcard or ellipsis scope: {scope}")
            continue
        if normalized in broad_roots:
            errors.append(f"{label} is too broad: {scope}")
            continue
        if len(normalized.split("/")) < 3 and not is_tracked_file_scope(normalized):
            errors.append(f"{label} is too broad: {scope}")
    return errors


def new_file_scope_errors(milestone_id: str, scopes: list[str], *, label: str) -> list[str]:
    errors: list[str] = []
    for scope in scopes:
        if is_generated_or_derived_scope(scope):
            continue
        if is_new_write_scope(scope):
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            continue
        path = REPO_ROOT / normalized
        if not git_tracks_path(path):
            errors.append(f"{milestone_id}: {label} new file scope must be marked with 'new:': {normalized}")
    return errors


def full_automation_crate_creation_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    scopes: list[str] = []
    if entry.product_code_contract is not None:
        scopes.extend(entry.product_code_contract.exact_allowed_implementation_write_scopes)
    if entry.implementation_writer is not None:
        scopes.extend(implementation_writer_allowed_scopes(entry.implementation_writer))
    crate_manifests = [
        manifest_write_scope_path(scope)
        for scope in scopes
        if is_new_write_scope(scope) and manifest_write_scope_path(scope)
    ]
    if not any(path and path.endswith("/Cargo.toml") for path in crate_manifests):
        return [f"{entry.milestone_id}: crate_creation requires exact new: crate Cargo.toml outputs"]
    return []


def git_tracks_path(path: Path) -> bool:
    try:
        relative = path.resolve().relative_to(REPO_ROOT.resolve())
    except ValueError:
        return path.exists()
    result = subprocess.run(
        ["git", "ls-files", "--error-unmatch", "--", relative.as_posix()],
        cwd=REPO_ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
        check=False,
    )
    return result.returncode == 0


def is_tracked_file_scope(normalized: str) -> bool:
    path = REPO_ROOT / normalized
    return path.is_file() and git_tracks_path(path)


def full_automation_handoff_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    if entry.may_create_code or entry.may_modify_production_behavior:
        errors.append(f"{entry.milestone_id}: handoff_closeout must not authorize product code or production behavior")
    if entry.handoff_contract is None:
        return errors
    handoff_text = "\n".join(
        [
            entry.handoff_contract.handoff_target,
            *entry.handoff_contract.proof_path_rules,
            *entry.handoff_contract.forbidden_scopes,
            *entry.handoff_contract.stop_conditions,
        ]
    )
    if "implementation" not in handoff_text.lower():
        errors.append(f"{entry.milestone_id}: handoff_contract must explicitly forbid downstream implementation")
    if "foundation/meta" not in handoff_text:
        errors.append(f"{entry.milestone_id}: handoff_contract must explicitly keep foundation/meta extraction blocked")
    return errors


def audit_manifest_milestone(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    for field_name in (
        "write_scope",
        "forbidden_scope",
        "required_contracts",
        "validation_commands",
        "evidence_gates",
        "stop_conditions",
    ):
        for value in getattr(entry, field_name):
            if value.startswith("blocked:"):
                errors.append(f"{entry.milestone_id}: {field_name} remains blocked: {value}")
    if entry.expected_closeout_path.startswith("blocked:"):
        errors.append(f"{entry.milestone_id}: expected_closeout_path remains blocked")
    if entry.expected_closeout_path and not entry.expected_closeout_path.endswith(".md"):
        errors.append(f"{entry.milestone_id}: expected_closeout_path must be a Markdown closeout/report path")
    return errors


def audit_manifest_action_contracts(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track: ProductionTrack,
    manifest: TrackExecutionManifest,
) -> list[str]:
    errors: list[str] = []
    if milestone.state == "completed":
        return errors
    if entry.future_wr_candidate and entry.auto_safe_contract is None:
        errors.append(f"{entry.milestone_id}: remaining milestone needs auto_safe_contract before full-track execution")
    contract = agent_design_contract_for_entry(entry)
    if contract is not None:
        errors.extend(agent_design_contract_errors(entry, contract))
    if entry.milestone_type in {"docs_only", "design_only"}:
        if contract is None:
            errors.append(f"{entry.milestone_id}: remaining design milestone needs agent_design_contract")
        if entry.agent_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: remaining design milestone needs agent_closeout_contract")
    if entry.milestone_type in {"implementation", "hardening"}:
        if contract is None:
            errors.append(f"{entry.milestone_id}: remaining implementation milestone needs agent_design_contract")
        if entry.product_code_contract is None:
            errors.append(f"{entry.milestone_id}: remaining implementation milestone needs product_code_contract")
        if entry.runtime_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: remaining implementation milestone needs runtime_closeout_contract")
        errors.extend(implementation_writer_contract_errors(entry, track=track, manifest=manifest))
    if entry.milestone_type == "closeout":
        if entry.agent_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: remaining closeout milestone needs agent_closeout_contract")
        if entry.handoff_contract is None:
            errors.append(f"{entry.milestone_id}: remaining closeout milestone needs handoff_contract")
    return errors


def agent_design_contract_errors(entry: TrackExecutionManifestMilestone, contract) -> list[str]:
    errors: list[str] = []
    errors.extend(validation_command_errors(contract.validation_commands, label=f"{entry.milestone_id}: agent_design_contract", product_code_eligible=True))
    if contract.authoring_strategy == "codex_contract_writer":
        output_scopes = contract.expected_output_paths or contract.agent_required_outputs
        if not output_scopes:
            errors.append(f"{entry.milestone_id}: codex_contract_writer requires expected_output_paths")
        if not contract.agent_prompt:
            errors.append(f"{entry.milestone_id}: codex_contract_writer requires agent_prompt")
        if not contract.agent_diff_protocol_version:
            errors.append(f"{entry.milestone_id}: codex_contract_writer requires agent_diff_protocol_version")
        errors.extend(exact_scope_list_errors(output_scopes, label=f"{entry.milestone_id}: codex_contract_writer output scope"))
    return errors


def implementation_writer_contract_errors(
    entry: TrackExecutionManifestMilestone,
    *,
    track: ProductionTrack,
    manifest: TrackExecutionManifest,
) -> list[str]:
    errors: list[str] = []
    writer = entry.implementation_writer
    if writer is None or writer.strategy == "no_writer":
        return errors
    if not implementation_writer_allowed_scopes(writer):
        errors.append(f"{entry.milestone_id}: implementation_writer must declare exact allowed files or write scopes")
    if not writer.required_outputs:
        errors.append(f"{entry.milestone_id}: implementation_writer.required_outputs must describe proof evidence")
    if not writer.validation_commands:
        errors.append(f"{entry.milestone_id}: implementation_writer.validation_commands must be explicit")
    if not writer.stop_conditions:
        errors.append(f"{entry.milestone_id}: implementation_writer.stop_conditions must be explicit")
    missing_writer_commands = [command for command in writer.validation_commands if command not in product_validation_commands_for_entry(entry)]
    if missing_writer_commands:
        errors.append(
            f"{entry.milestone_id}: implementation_writer validation commands are not covered by product_code_contract: "
            + ", ".join(missing_writer_commands)
        )
    errors.extend(exact_scope_list_errors(implementation_writer_allowed_scopes(writer), label=f"{entry.milestone_id}: implementation_writer allowed scope"))
    if writer.strategy == "agent_writer":
        if not writer.agent_diff_protocol_version:
            errors.append(f"{entry.milestone_id}: agent_writer requires agent_diff_protocol_version")
        if not writer.agent_prompt:
            errors.append(f"{entry.milestone_id}: agent_writer requires a bounded agent_prompt")
        if writer.templates or writer.patches:
            errors.append(f"{entry.milestone_id}: agent_writer cannot also declare templates or patches")
    if writer.strategy == "proof_aggregation_writer":
        errors.extend(proof_aggregation_writer_contract_errors(entry, writer, track=track, manifest=manifest))
    return errors


def proof_aggregation_writer_contract_errors(
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
    *,
    track: ProductionTrack,
    manifest: TrackExecutionManifest,
) -> list[str]:
    errors: list[str] = []
    if not writer.aggregation_only:
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer must set aggregation_only: true")
    if not writer.required_prior_milestones:
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer requires required_prior_milestones")
    if writer.required_prior_completion_quality != "runtime_proven":
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer requires required_prior_completion_quality: runtime_proven")
    required_writer_categories = {normalize_evidence_category(category) for category in writer.required_evidence_categories}
    missing_categories = sorted(
        normalize_evidence_category(category)
        for category in PROOF_AGGREGATION_REQUIRED_EVIDENCE_CATEGORIES
        if normalize_evidence_category(category) not in required_writer_categories
    )
    if missing_categories:
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer missing required evidence categories: " + ", ".join(missing_categories))
    if writer.closeout_path != entry.expected_closeout_path:
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer closeout_path must match expected_closeout_path")
    if not [*writer.forbidden_files, *writer.forbidden_scopes]:
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer must declare forbidden files or scopes")
    errors.extend(proof_aggregation_prior_errors(entry, writer, track=track, manifest=manifest))
    return errors


def proof_aggregation_prior_errors(
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
    *,
    track: ProductionTrack,
    manifest: TrackExecutionManifest,
) -> list[str]:
    errors: list[str] = []
    production_by_id = {milestone.id: milestone for milestone in track.milestones}
    aggregate_evidence_categories: set[str] = set()
    for prior_id in writer.required_prior_milestones:
        prior = production_by_id.get(prior_id)
        if prior is None:
            errors.append(f"{entry.milestone_id}: required prior milestone {prior_id} is not present")
            continue
        if prior.state != "completed":
            errors.append(f"{entry.milestone_id}: required prior milestone {prior_id} is {prior.state}, expected completed")
        if prior.completion_quality != writer.required_prior_completion_quality:
            errors.append(
                f"{entry.milestone_id}: required prior milestone {prior_id} has completion_quality "
                f"{prior.completion_quality}, expected {writer.required_prior_completion_quality}"
            )
        if not prior.completion_audit:
            errors.append(f"{entry.milestone_id}: required prior milestone {prior_id} has no completion_audit closeout")
            continue
        closeout = manifest_path_reference(prior.completion_audit)
        if not closeout.exists():
            errors.append(f"{entry.milestone_id}: required prior milestone {prior_id} closeout is missing: {prior.completion_audit}")
            continue
        try:
            record = closeout_evidence_record(closeout)
        except ValueError as error:
            errors.append(f"{entry.milestone_id}: required prior milestone {prior_id} closeout evidence metadata is invalid: {error}")
            continue
        if manifest.full_automation_target and record is None:
            errors.append(f"{entry.milestone_id}: required prior milestone {prior_id} closeout is missing closeout_evidence metadata")
        if record is not None:
            aggregate_evidence_categories.update(normalize_evidence_category(category) for category in record.evidence_categories)

    if manifest.full_automation_target:
        required = {normalize_evidence_category(category) for category in writer.required_evidence_categories}
        missing = sorted(required - aggregate_evidence_categories)
        if missing:
            errors.append(f"{entry.milestone_id}: proof_aggregation_writer prior closeout evidence metadata missing categories: " + ", ".join(missing))

    prior_product_scopes: list[str] = []
    for prior_id in writer.required_prior_milestones:
        prior_entry = manifest.by_milestone_id.get(prior_id)
        if prior_entry is not None:
            prior_product_scopes.extend(product_implementation_scopes_for_entry(prior_entry))
    prior_paths = [normalized for normalized in (manifest_write_scope_path(scope) for scope in prior_product_scopes) if normalized is not None]
    for scope in implementation_writer_output_scopes(writer):
        writer_path = manifest_write_scope_path(scope)
        if writer_path is None:
            continue
        for prior_path in prior_paths:
            if path_within_scope(writer_path, prior_path) or path_within_scope(prior_path, writer_path):
                errors.append(f"{entry.milestone_id}: proof_aggregation_writer must not modify prior proof-slice product file {prior_path}")
    return errors


def first_current_manifest_entry(
    manifest: TrackExecutionManifest,
    track: ProductionTrack,
) -> tuple[TrackExecutionManifestMilestone, ProductionMilestone]:
    manifest_by_id = manifest.by_milestone_id
    ordered = ordered_track_milestones(track)
    for milestone in ordered:
        entry = manifest_by_id[milestone.id]
        if milestone.state != "completed":
            return entry, milestone
    last_milestone = ordered[-1]
    return manifest_by_id[last_milestone.id], last_milestone


def next_action_blockers(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    planning: ProductionPlanningState,
    track: ProductionTrack,
    roadmap: RoadmapState,
) -> tuple[str, list[str]]:
    production_by_id = {candidate.id: candidate for candidate in track.milestones}
    blockers: list[str] = []
    for dependency in entry.predecessor_dependencies:
        dependency_state = production_by_id.get(dependency).state if dependency in production_by_id else "missing"
        if dependency_state != "completed":
            blockers.append(f"{entry.milestone_id}: dependency {dependency} is {dependency_state}, expected completed")
    if entry.future_wr_candidate:
        blockers.append(f"{entry.milestone_id}: Track Expansion must create or link {entry.future_wr_candidate}")
        return "track_expansion_required", blockers
    if not entry.owning_wr:
        blockers.append(f"{entry.milestone_id}: missing owning WR")
        return "missing_wr_authority", blockers
    item = roadmap.by_id.get(entry.owning_wr)
    if item is None:
        blockers.append(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
        return "missing_wr_authority", blockers
    action = classify_plan_action(
        ProductionPlanContext(
            planning=planning,
            roadmap=roadmap,
            track=track,
            milestone=milestone,
            roadmap_item=item,
        )
    )
    if entry.milestone_type in {"docs_only", "design_only", "implementation", "hardening"} and action == "design_first":
        return action, blockers
    if action != "write_implementation_contract":
        blockers.append(f"{item.id}: workflow action is {action} (state={item.planning_state}, blocker={item.blocker_label})")
    return action, blockers


def implementation_authorization_note(
    entry: TrackExecutionManifestMilestone,
    workflow_action: str,
    blockers: list[str],
) -> str:
    if not entry.may_create_code:
        return "no - manifest milestone does not allow code creation"
    if blockers:
        return "no - blockers must be cleared first"
    if workflow_action == "runtime_closeout":
        return "no - implementation evidence exists; closeout requires run-track with --allow agent_closeout"
    if workflow_action != "write_implementation_contract":
        return f"no - workflow action is {workflow_action}"
    return "no - task production:next is read-only; product implementation requires an explicit run-track command with --allow product_code --allow product_implementation and a valid accepted plan"
