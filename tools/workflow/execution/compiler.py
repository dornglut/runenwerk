from __future__ import annotations

from hashlib import sha256
from pathlib import Path

import yaml

from production_plan import default_contract_path
from production_state import PRODUCTION_SOURCE, load_production_tracks
from roadmap_state import ROADMAP_SOURCE, REPO_ROOT, WorkflowError, load_roadmap, repo_path
from track_execution_manifest import (
    TRACK_EXECUTION_MANIFEST_ROOT,
    load_track_execution_manifest,
    manifest_write_scope_path,
    product_forbidden_scopes_for_entry,
    product_implementation_scopes_for_entry,
    product_validation_commands_for_entry,
    source_digest_map,
)

from execution.contracts import (
    ActionContract,
    CloseoutContract,
    ContractPack,
    EvidenceRequirement,
    ExecutorKind,
    RollbackPolicy,
    TextPatch,
    now_utc_iso,
)


CONTRACT_PACK_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/execution-contract-packs"

EVIDENCE_ALIASES = {
    "runtime_test": "runtime_test",
    "runtime test": "runtime_test",
    "runtime/test": "runtime_test",
    "headless_fixture": "fixture",
    "headless fixture": "fixture",
    "fixture": "fixture",
    "fixtures": "fixture",
    "diagnostic": "diagnostics",
    "diagnostics": "diagnostics",
    "source_map": "source_maps",
    "source-map": "source_maps",
    "source-map proof": "source_maps",
    "source maps": "source_maps",
    "source_maps": "source_maps",
    "artifact": "artifact",
    "runtime artifact evidence": "artifact",
    "migration": "migration",
    "migrations": "migration",
    "reproducibility": "reproducibility",
    "reproducibility evidence": "reproducibility",
    "visual": "visual",
    "visual/render output proof": "visual",
    "handoff": "handoff",
    "closeout": "handoff",
    "closeout evidence": "handoff",
}


def contract_pack_path(track_id: str, *, root: Path = CONTRACT_PACK_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


def load_contract_pack(track_id: str, *, root: Path = CONTRACT_PACK_ROOT) -> ContractPack | None:
    path = contract_pack_path(track_id, root=root)
    if not path.exists():
        return None
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    try:
        pack = ContractPack.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid Execution Contract Pack: {error}") from error
    if pack.track_id != track_id:
        raise WorkflowError(f"{repo_path(path)} declares track_id={pack.track_id}, expected {track_id}")
    return pack


def write_contract_pack(pack: ContractPack, *, root: Path = CONTRACT_PACK_ROOT) -> Path:
    path = contract_pack_path(pack.track_id, root=root)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(pack.model_dump(mode="json", exclude_none=True), sort_keys=False, width=4096),
        encoding="utf-8",
        newline="\n",
    )
    return path


def normalized_contract_output(scope: str) -> str | None:
    if scope.startswith("generated:") or scope.startswith("derived:"):
        return None
    return manifest_write_scope_path(scope)


def evidence_requirements(categories: list[str]) -> list[EvidenceRequirement]:
    requirements: list[EvidenceRequirement] = []
    for category in categories:
        key = category.strip().lower().replace("-", "_")
        kind = EVIDENCE_ALIASES.get(key) or EVIDENCE_ALIASES.get(category.strip().lower())
        if kind is None:
            raise WorkflowError(f"unknown evidence category for Contract Pack: {category}")
        requirements.append(EvidenceRequirement(kind=kind, name=category))
    return requirements


def permissions_for_entry(entry, executor_kind: ExecutorKind) -> list[str]:
    if entry.future_wr_candidate:
        return ["auto_safe"]
    if executor_kind == "design_authoring":
        permissions = ["agent_design"]
    elif executor_kind == "product_implementation":
        permissions = ["product_code", "product_implementation"]
    elif executor_kind == "proof_aggregation":
        permissions = ["product_code", "product_implementation"]
    elif executor_kind in {"runtime_closeout", "handoff_closeout", "truth_claim_update"}:
        permissions = ["agent_closeout"]
    elif executor_kind == "extraction_gate":
        permissions = ["foundation_extraction"]
    else:
        permissions = list(dict.fromkeys(entry.permission_classes_required))
    if "crate_creation" in entry.permission_classes_required and "crate_creation" not in permissions:
        permissions.append("crate_creation")
    if "foundation_extraction" in entry.permission_classes_required and "foundation_extraction" not in permissions:
        permissions.append("foundation_extraction")
    return permissions


def writer_strategy_for_entry(entry) -> str:
    if entry.implementation_writer is None:
        return "no_writer"
    return entry.implementation_writer.strategy


def template_outputs_for_entry(entry) -> dict[str, str]:
    if entry.implementation_writer is None:
        return {}
    return {
        normalized: template.content
        for template in entry.implementation_writer.templates
        if (normalized := normalized_contract_output(template.file)) is not None
    }


def patches_for_entry(entry) -> list[TextPatch]:
    if entry.implementation_writer is None:
        return []
    patches: list[TextPatch] = []
    for patch in entry.implementation_writer.patches:
        normalized = normalized_contract_output(patch.file)
        if normalized is None:
            continue
        patches.append(TextPatch(path=normalized, find=patch.find, replace=patch.replace))
    return patches


def closeout_quality_for_entry(entry) -> str:
    if entry.runtime_closeout_contract is not None and entry.runtime_closeout_contract.completion_quality_allowed:
        return entry.runtime_closeout_contract.completion_quality_allowed[0]
    if entry.handoff_contract is not None:
        return "bounded_contract"
    if entry.agent_closeout_contract is not None and entry.milestone_type in {"docs_only", "design_only", "closeout"}:
        return "bounded_contract"
    if entry.milestone_type in {"implementation", "hardening"}:
        return "runtime_proven"
    return "not_applicable"


def allowed_outputs_for_entry(entry) -> tuple[list[str], list[str]]:
    scopes = product_implementation_scopes_for_entry(entry) if entry.milestone_type in {"implementation", "hardening"} else entry.write_scope
    allowed: list[str] = []
    new: list[str] = []
    for scope in scopes:
        output = normalized_contract_output(scope)
        if output is None:
            continue
        if scope.strip().startswith("new:"):
            new.append(output)
        else:
            allowed.append(output)
    return list(dict.fromkeys(allowed)), list(dict.fromkeys(new))


def forbidden_outputs_for_entry(entry) -> list[str]:
    scopes = product_forbidden_scopes_for_entry(entry) if entry.milestone_type in {"implementation", "hardening"} else entry.forbidden_scope
    outputs = [normalized_contract_output(scope) for scope in scopes]
    return list(dict.fromkeys(output for output in outputs if output is not None))


def validation_commands_for_entry(entry) -> list[str]:
    if entry.milestone_type in {"implementation", "hardening"}:
        return product_validation_commands_for_entry(entry)
    return list(entry.validation_commands)


def executor_kind_for_entry(entry) -> ExecutorKind:
    if entry.future_wr_candidate:
        return "planning_expansion"
    if entry.execution_kind == "design_contract":
        return "design_authoring"
    if entry.execution_kind == "implementation_proof":
        return "product_implementation"
    if entry.execution_kind == "proof_aggregation":
        return "proof_aggregation"
    if entry.execution_kind == "handoff_closeout":
        return "handoff_closeout"
    if entry.execution_kind == "extraction_gate":
        return "extraction_gate"
    raise WorkflowError(f"{entry.milestone_id}: unsupported execution_kind {entry.execution_kind}")


def structured_plan_contract_path(plan_path: Path) -> Path:
    return plan_path.with_suffix(".execution.yaml")


def load_structured_plan_contract(entry, plan_path: Path, executor_kind: ExecutorKind) -> dict | None:
    if executor_kind not in {"product_implementation", "proof_aggregation", "runtime_closeout", "handoff_closeout", "truth_claim_update"}:
        return None
    sidecar_path = structured_plan_contract_path(plan_path)
    if not sidecar_path.exists():
        raise WorkflowError(
            f"{entry.milestone_id}: structured execution plan metadata is missing at {repo_path(sidecar_path)}"
        )
    data = yaml.safe_load(sidecar_path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(sidecar_path)} must contain a YAML mapping")
    if data.get("milestone_id") != entry.milestone_id:
        raise WorkflowError(f"{repo_path(sidecar_path)} milestone_id must be {entry.milestone_id}")
    if data.get("executor_kind") != executor_kind:
        raise WorkflowError(f"{repo_path(sidecar_path)} executor_kind must be {executor_kind}")
    if data.get("wr_id") and data["wr_id"] != entry.owning_wr:
        raise WorkflowError(f"{repo_path(sidecar_path)} wr_id must be {entry.owning_wr}")
    return data


def compile_contract_pack(
    track_id: str,
    *,
    production_source: Path = PRODUCTION_SOURCE,
    roadmap_source: Path = ROADMAP_SOURCE,
    manifest_root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> ContractPack:
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    track = next((candidate for candidate in planning.tracks if candidate.id == track_id), None)
    if track is None:
        raise WorkflowError(f"{track_id}: not present in production tracks source")
    loaded = load_track_execution_manifest(track_id, root=manifest_root)
    if loaded is None:
        raise WorkflowError(f"{track_id}: missing Track Execution Manifest")

    actions: list[ActionContract] = []
    authority_paths: list[Path] = []
    for milestone in track.milestones:
        if milestone.state == "completed":
            continue
        entry = loaded.manifest.by_milestone_id.get(milestone.id)
        if entry is None:
            raise WorkflowError(f"{track_id}: manifest missing milestone {milestone.id}")
        wr_id = entry.owning_wr or entry.future_wr_candidate
        if not wr_id:
            raise WorkflowError(f"{entry.milestone_id}: Contract Pack action requires owning_wr or future_wr_candidate")
        executor_kind = executor_kind_for_entry(entry)
        if entry.future_wr_candidate and executor_kind != "planning_expansion":
            raise WorkflowError(
                f"{entry.milestone_id}: future_wr_candidate may compile only to planning_expansion"
            )
        plan_contract: dict | None = None
        if entry.owning_wr:
            roadmap_item = roadmap.by_id.get(entry.owning_wr)
            if roadmap_item is None:
                raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is missing from roadmap")
            plan_path = default_contract_path(roadmap_item)
            if not plan_path.exists():
                raise WorkflowError(f"{entry.milestone_id}: implementation/design plan is missing at {repo_path(plan_path)}")
            authority_paths.append(plan_path)
            plan_contract = load_structured_plan_contract(entry, plan_path, executor_kind)
            sidecar_path = structured_plan_contract_path(plan_path)
            if sidecar_path.exists():
                authority_paths.append(sidecar_path)
        if executor_kind in {"product_implementation", "proof_aggregation"} and entry.runtime_closeout_contract is None:
            raise WorkflowError(f"{entry.milestone_id}: runtime_closeout_contract is required for runtime proof actions")
        allowed_outputs, new_outputs = allowed_outputs_for_entry(entry)
        if plan_contract is not None:
            sidecar_allowed = [
                output
                for output in (normalized_contract_output(scope) for scope in list(plan_contract.get("allowed_outputs", [])))
                if output is not None
            ]
            sidecar_new = [
                output
                for output in (normalized_contract_output(f"new: {scope}") for scope in list(plan_contract.get("new_outputs", [])))
                if output is not None
            ]
            sidecar_outputs = set(sidecar_allowed + sidecar_new)
            manifest_outputs = set(allowed_outputs + new_outputs)
            missing = sorted(sidecar_outputs - manifest_outputs)
            if missing:
                raise WorkflowError(
                    f"{entry.milestone_id}: structured plan outputs are outside manifest/WR scope: {', '.join(missing)}"
                )
        action_id = f"{track_id}:{entry.milestone_id}:{wr_id}"
        actions.append(
            ActionContract(
                action_id=action_id,
                track_id=track_id,
                milestone_id=entry.milestone_id,
                wr_id=wr_id,
                execution_kind=entry.execution_kind,
                executor_kind=executor_kind,
                authority_level=entry.authority_level,
                permissions_required=permissions_for_entry(entry, executor_kind),
                allowed_outputs=allowed_outputs,
                new_outputs=new_outputs,
                forbidden_outputs=forbidden_outputs_for_entry(entry),
                forbidden_patterns=list(entry.implementation_writer.forbidden_patterns)
                if entry.implementation_writer is not None
                else [],
                writer_strategy=writer_strategy_for_entry(entry),
                template_outputs=template_outputs_for_entry(entry),
                patches=patches_for_entry(entry),
                validation_commands=validation_commands_for_entry(entry),
                evidence_required=evidence_requirements(entry.required_evidence_categories),
                closeout_contract=CloseoutContract(
                    path=entry.expected_closeout_path,
                    completion_quality=closeout_quality_for_entry(entry),
                    evidence_required=evidence_requirements(entry.required_evidence_categories),
                ),
                rollback_policy=RollbackPolicy(policy="reject import and leave repository unchanged on validation, scope, or digest failure"),
                stop_conditions=list(entry.stop_conditions),
                required_prior_milestones=list(entry.implementation_writer.required_prior_milestones)
                if entry.implementation_writer is not None
                else [],
                required_prior_completion_quality=entry.implementation_writer.required_prior_completion_quality
                if entry.implementation_writer is not None
                else None,
            )
        )

    return ContractPack(
        track_id=track_id,
        generated_at=now_utc_iso(),
        source_digests={
            **source_digest_map(loaded, production_source=production_source, roadmap_source=roadmap_source),
            **authority_source_digest_map(authority_paths),
            **harness_source_digest_map(),
        },
        actions=actions,
    )


def first_action(pack: ContractPack) -> ActionContract | None:
    return pack.actions[0] if pack.actions else None


def harness_source_digest_map() -> dict[str, str]:
    root = REPO_ROOT / "tools/workflow/execution"
    return {
        repo_path(path): sha256(path.read_bytes()).hexdigest()
        for path in sorted(root.glob("*.py"))
    }


def authority_source_digest_map(paths: list[Path]) -> dict[str, str]:
    unique: dict[str, Path] = {}
    for path in paths:
        unique[repo_path(path)] = path
    return {
        source: sha256(path.read_bytes()).hexdigest()
        for source, path in sorted(unique.items())
        if path.exists()
    }
