from __future__ import annotations

from hashlib import sha256
from pathlib import Path

import yaml

from production_plan import default_contract_path
from production_state import PRODUCTION_SOURCE, load_production_tracks
from roadmap_state import ROADMAP_SOURCE, REPO_ROOT, WorkflowError, load_roadmap, path_within_scope, repo_path, split_source_paths
from track_sources.manifest import (
    TRACK_EXECUTION_MANIFEST_ROOT,
    agent_design_contract_for_entry,
    load_track_execution_manifest,
    manifest_write_scope_path,
    product_forbidden_scopes_for_entry,
    product_implementation_scopes_for_entry,
    product_validation_commands_for_entry,
    source_digest_paths,
)
from track_sources.plan_metadata import (
    PlanContractMetadata,
    load_plan_contract_metadata,
    structured_plan_contract_path,
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
    "architecture_contract": "artifact",
    "architecture contract": "artifact",
    "architecture_contract_design": "artifact",
    "architecture contract design": "artifact",
    "governance": "artifact",
    "owner_map": "artifact",
    "owner map": "artifact",
    "schema": "artifact",
    "surface": "artifact",
    "truth_claims": "artifact",
    "truth claims": "artifact",
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


def evidence_requirement_from_contract(value) -> EvidenceRequirement:
    if not isinstance(value, dict):
        raise WorkflowError("plan.contract.yaml evidence_required entries must be mappings with resolver metadata")
    data = dict(value)
    kind = data.get("kind")
    if isinstance(kind, str):
        key = kind.strip().lower().replace("-", "_")
        data["kind"] = EVIDENCE_ALIASES.get(key) or EVIDENCE_ALIASES.get(kind.strip().lower()) or kind
    try:
        return EvidenceRequirement.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"invalid plan.contract.yaml evidence requirement: {error}") from error


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
    if entry.handoff_contract is not None and entry.agent_closeout_contract is not None and entry.agent_closeout_contract.completion_quality_allowed:
        return entry.agent_closeout_contract.completion_quality_allowed[0]
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


def planning_expansion_outputs(
    *,
    track_id: str,
    production_source: Path,
    roadmap_source: Path,
    manifest_source: Path,
    contract_pack_root: Path,
) -> tuple[list[str], list[str]]:
    _archive_source, deferred_source = split_source_paths(roadmap_source)
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    allowed = [
        source_path_for_action(production_source, production_source=production_source),
        source_path_for_action(manifest_source, production_source=production_source),
    ]
    deferred_output = source_path_for_action(deferred_source, production_source=production_source)
    if deferred_source.exists():
        allowed.append(deferred_output)
        new: list[str] = []
    else:
        new = [deferred_output]
    generated_allowed, generated_new = generated_outputs_for_sources(
        production_source=production_source,
        roadmap_source=roadmap_source,
    )
    allowed.extend(generated_allowed)
    new.extend(generated_new)
    allowed.extend(existing_contract_pack_outputs(track_id=track_id, contract_pack_root=contract_pack_root, production_source=production_source))
    return list(dict.fromkeys(allowed)), list(dict.fromkeys(new))


def existing_contract_pack_outputs(*, track_id: str, contract_pack_root: Path, production_source: Path) -> list[str]:
    outputs = [
        source_path_for_action(contract_pack_path(track_id, root=contract_pack_root), production_source=production_source)
    ]
    if not contract_pack_root.exists():
        return outputs
    outputs.extend(
        source_path_for_action(pack_path, production_source=production_source)
        for pack_path in sorted(contract_pack_root.glob("*.yaml"))
    )
    return list(dict.fromkeys(outputs))


def runtime_closeout_outputs(
    *,
    track_id: str,
    closeout_path: str,
    production_source: Path,
    roadmap_source: Path,
    manifest_source: Path,
    contract_pack_root: Path,
) -> tuple[list[str], list[str]]:
    archive_source, deferred_source = split_source_paths(roadmap_source)
    allowed = [
        source_path_for_action(production_source, production_source=production_source),
        source_path_for_action(roadmap_source, production_source=production_source),
        source_path_for_action(manifest_source, production_source=production_source),
    ]
    new: list[str] = []
    for source in (archive_source, deferred_source):
        output = source_path_for_action(source, production_source=production_source)
        if source.exists():
            allowed.append(output)
        else:
            new.append(output)
    generated_allowed, generated_new = generated_outputs_for_sources(
        production_source=production_source,
        roadmap_source=roadmap_source,
    )
    allowed.extend(generated_allowed)
    new.extend(generated_new)
    closeout_output = normalized_contract_output(closeout_path)
    if closeout_output is not None:
        closeout_file = source_root_for_action_paths(production_source) / closeout_output
        if closeout_file.exists():
            allowed.append(closeout_output)
        else:
            new.append(closeout_output)
    allowed.extend(existing_contract_pack_outputs(track_id=track_id, contract_pack_root=contract_pack_root, production_source=production_source))
    return list(dict.fromkeys(allowed)), list(dict.fromkeys(new))


def forbidden_outputs_for_entry(entry) -> list[str]:
    scopes = product_forbidden_scopes_for_entry(entry) if entry.milestone_type in {"implementation", "hardening"} else entry.forbidden_scope
    outputs = [normalized_contract_output(scope) for scope in scopes]
    return list(dict.fromkeys(output for output in outputs if output is not None))


def validation_commands_for_entry(entry) -> list[str]:
    if entry.future_wr_candidate and entry.auto_safe_contract is not None:
        return list(entry.auto_safe_contract.validation_commands)
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


def executor_kind_after_plan(entry) -> ExecutorKind:
    if entry.execution_kind == "implementation_proof":
        return "product_implementation"
    if entry.execution_kind == "proof_aggregation":
        return "proof_aggregation"
    if entry.execution_kind == "handoff_closeout":
        return "handoff_closeout"
    if entry.execution_kind == "extraction_gate":
        return "extraction_gate"
    if entry.execution_kind == "design_contract":
        return "design_authoring"
    raise WorkflowError(f"{entry.milestone_id}: unsupported execution_kind {entry.execution_kind}")


def load_structured_plan_contract(entry, plan_path: Path, executor_kind: ExecutorKind) -> PlanContractMetadata | None:
    if executor_kind == "planning_expansion":
        return None
    return load_plan_contract_metadata(
        plan_path=plan_path,
        milestone_id=entry.milestone_id,
        executor_kind=executor_kind,
        wr_id=entry.owning_wr,
    )


def normalize_sidecar_allowed_outputs(plan_contract: PlanContractMetadata, *, production_source: Path) -> list[str]:
    outputs: list[str] = []
    for scope in plan_contract.allowed_outputs:
        if scope.strip().startswith("new:"):
            raise WorkflowError(f"{plan_contract.milestone_id}: allowed_outputs must not use new: scope markers")
        outputs.extend(expand_scope_to_existing_files(scope, production_source=production_source))
    return list(dict.fromkeys(outputs))


def normalize_sidecar_new_outputs(plan_contract: PlanContractMetadata) -> list[str]:
    outputs: list[str] = []
    for scope in plan_contract.new_outputs:
        marked = scope if scope.strip().startswith("new:") else f"new: {scope}"
        normalized = normalized_contract_output(marked)
        if normalized is not None:
            outputs.append(normalized)
    return list(dict.fromkeys(outputs))


def normalize_sidecar_outputs(scopes: list[str]) -> list[str]:
    outputs = [normalized_contract_output(scope) for scope in scopes]
    return list(dict.fromkeys(output for output in outputs if output is not None))


def expand_scope_to_existing_files(scope: str, *, production_source: Path) -> list[str]:
    normalized = normalized_contract_output(scope)
    if normalized is None:
        return []
    source_root = production_source.resolve().parent
    repo_candidate = REPO_ROOT / normalized
    candidate = repo_candidate if repo_candidate.exists() else source_root / normalized
    if candidate.is_file():
        return [normalized]
    if candidate.is_dir():
        return [
            source_path_for_action(path, production_source=production_source)
            for path in sorted(candidate.rglob("*"))
            if path.is_file() and "__pycache__" not in path.parts and path.suffix != ".pyc"
        ]
    return [normalized]


def default_evidence_output(
    *,
    track_id: str,
    milestone_id: str,
    kind: str,
    name: str,
) -> str:
    safe_name = name.lower().replace(" ", "-").replace("/", "-")
    return (
        "docs-site/src/content/docs/reports/execution-evidence/"
        f"{track_id.lower()}/{milestone_id.lower()}/{kind}-{safe_name}.yaml"
    )


def source_root_for_action_paths(production_source: Path) -> Path:
    try:
        production_source.resolve().relative_to(REPO_ROOT.resolve())
        return REPO_ROOT
    except ValueError:
        pass
    standard_root = standard_repo_source_root(production_source)
    if standard_root is not None:
        return standard_root
    return production_source.resolve().parent


def standard_repo_source_root(production_source: Path) -> Path | None:
    suffix = Path("docs-site/src/content/docs/workspace/production-tracks.yaml").parts
    resolved = production_source.resolve()
    if tuple(resolved.parts[-len(suffix) :]) != suffix:
        return None
    root = resolved
    for _part in suffix:
        root = root.parent
    return root


def evidence_requirements_satisfied(requirements: list[EvidenceRequirement], *, production_source: Path) -> bool:
    root = source_root_for_action_paths(production_source)
    for requirement in requirements:
        if requirement.required and not requirement.paths:
            return False
        for evidence_path in requirement.paths:
            if not (root / evidence_path).exists():
                return False
    return True


def generated_outputs_for_sources(*, production_source: Path, roadmap_source: Path) -> tuple[list[str], list[str]]:
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    source_root = source_root_for_action_paths(production_source)
    outputs = [
        source_root / planning.render.production_index,
        source_root / planning.render.milestone_register,
        source_root / planning.render.track_roadmap,
        source_root / planning.render.full_track_roadmap,
        source_root / roadmap.render.decision_register,
        source_root / roadmap.render.dependency_roadmap,
        source_root / roadmap.render.current_candidates_roadmap,
        source_root / roadmap.render.triage,
        source_root / roadmap.render.archive_register,
        source_root / roadmap.render.deferred_register,
    ]
    allowed: list[str] = []
    new: list[str] = []
    for path in outputs:
        output = source_path_for_action(path, production_source=production_source)
        if path.exists():
            allowed.append(output)
        else:
            new.append(output)
    return allowed, new


def runtime_test_validation_command_ids(validation_commands: list[str]) -> list[str]:
    ids: list[str] = []
    for raw in validation_commands:
        stripped = raw.strip()
        if stripped.startswith("cargo test"):
            ids.append("cargo:test")
        elif stripped.startswith("uv run pytest"):
            ids.append("uv:pytest")
        elif stripped.startswith("task test") or stripped.startswith("task ci:"):
            ids.append(f"task:{stripped.split()[1]}")
    return list(dict.fromkeys(ids))


def evidence_subject_paths_for_entry(entry, *, production_source: Path) -> list[str]:
    subject_paths: list[str] = []
    for scope in entry.write_scope:
        output = normalized_contract_output(scope)
        if output is None:
            continue
        if output.startswith("docs-site/src/content/docs/reports/execution-evidence/"):
            continue
        if output.endswith("/closeout.md") or output.endswith("/plan.md") or output.endswith("/plan.contract.yaml"):
            continue
        if output.startswith("docs-site/src/content/docs/workspace/"):
            continue
        subject_paths.extend(expand_scope_to_existing_files(scope, production_source=production_source))
    return list(dict.fromkeys(subject_paths))


def sidecar_evidence_entries(entry, *, track_id: str, production_source: Path) -> list[dict]:
    entries: list[dict] = []
    subject_paths = evidence_subject_paths_for_entry(entry, production_source=production_source)
    for category in entry.required_evidence_categories:
        key = category.strip().lower().replace("-", "_")
        kind = EVIDENCE_ALIASES.get(key) or EVIDENCE_ALIASES.get(category.strip().lower())
        if kind is None:
            raise WorkflowError(f"unknown evidence category for Contract Pack: {category}")
        evidence = {
            "kind": kind,
            "name": category,
            "paths": [
                default_evidence_output(
                    track_id=track_id,
                    milestone_id=entry.milestone_id,
                    kind=kind,
                    name=category,
                )
            ],
        }
        if kind == "runtime_test":
            command_ids = runtime_test_validation_command_ids(list(validation_commands_for_entry(entry)))
            if not command_ids:
                command_ids = ["missing:runtime_test_validation"]
            evidence["validation_command_ids"] = command_ids
        else:
            if not subject_paths:
                raise WorkflowError(f"{entry.milestone_id}: {kind} evidence requires exact subject paths")
            evidence["subject_paths"] = subject_paths
        entries.append(evidence)
    return entries


def text_patches_from_contract(plan_contract: PlanContractMetadata) -> list[TextPatch]:
    patches: list[TextPatch] = []
    for patch in plan_contract.patches:
        file = patch.get("file") or patch.get("path")
        if not isinstance(file, str):
            raise WorkflowError(f"{plan_contract.milestone_id}: patch entries require file or path")
        normalized = normalized_contract_output(file)
        if normalized is None:
            raise WorkflowError(f"{plan_contract.milestone_id}: patch file is not an executable output path: {file}")
        try:
            patches.append(TextPatch(path=normalized, find=patch["find"], replace=patch["replace"]))
        except KeyError as error:
            raise WorkflowError(f"{plan_contract.milestone_id}: patch entries require find and replace") from error
    return patches


def template_outputs_from_contract(plan_contract: PlanContractMetadata) -> dict[str, str]:
    outputs: dict[str, str] = {}
    for path, content in plan_contract.template_outputs.items():
        normalized = normalized_contract_output(path)
        if normalized is None:
            raise WorkflowError(f"{plan_contract.milestone_id}: template output is not an executable output path: {path}")
        outputs[normalized] = content
    return outputs


def evidence_requirements_from_contract(plan_contract: PlanContractMetadata) -> list[EvidenceRequirement]:
    if not plan_contract.evidence_required:
        raise WorkflowError(f"{plan_contract.milestone_id}: plan.contract.yaml evidence_required must not be empty")
    return [evidence_requirement_from_contract(item) for item in plan_contract.evidence_required]


def validation_commands_from_contract(plan_contract: PlanContractMetadata) -> list:
    if not plan_contract.validation_commands:
        raise WorkflowError(f"{plan_contract.milestone_id}: plan.contract.yaml validation_commands must not be empty")
    return list(plan_contract.validation_commands)


def closeout_contract_from_contract(plan_contract: PlanContractMetadata, evidence_required: list[EvidenceRequirement]) -> CloseoutContract:
    if not plan_contract.closeout_contract:
        raise WorkflowError(f"{plan_contract.milestone_id}: plan.contract.yaml closeout_contract must not be empty")
    data = dict(plan_contract.closeout_contract)
    if "evidence_required" not in data:
        raise WorkflowError(f"{plan_contract.milestone_id}: closeout_contract.evidence_required must be explicit")
    try:
        return CloseoutContract.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{plan_contract.milestone_id}: invalid closeout_contract: {error}") from error


def plan_authoring_markdown(*, entry, wr_id: str, future_executor_kind: ExecutorKind) -> str:
    return "\n".join(
        [
            "---",
            f"title: {entry.title} Implementation Plan",
            "status: active",
            "type: implementation-plan",
            f"wr: {wr_id}",
            f"milestone: {entry.milestone_id}",
            "---",
            "",
            "# Implementation Plan",
            "",
            "This plan was generated by the clean Track Execution Harness design_authoring action.",
            "Executable authority lives in `plan.contract.yaml`; this prose is a human-readable companion.",
            "",
            "## Scope",
            "",
            f"- Track: `{entry.milestone_id}`",
            f"- Executor: `{future_executor_kind}`",
            "- Product-domain code, MaterialProgram, crate creation, placeholder folders, and foundation/meta extraction remain forbidden unless explicitly authorized by this contract.",
            "",
            "## Validation",
            "",
            *[f"- `{command}`" for command in entry.validation_commands],
            "",
            "## Stop Conditions",
            "",
            *[f"- {condition}" for condition in entry.stop_conditions],
            "",
        ]
    )


def plan_authoring_sidecar(
    *,
    track_id: str,
    entry,
    wr_id: str,
    future_executor_kind: ExecutorKind,
    production_source: Path,
) -> str:
    evidence = sidecar_evidence_entries(entry, track_id=track_id, production_source=production_source)
    evidence_paths = [path for item in evidence for path in item.get("paths", [])]
    expanded_allowed: list[str] = []
    expanded_new: list[str] = []
    for scope in entry.write_scope:
        normalized = normalized_contract_output(scope)
        if normalized is None:
            continue
        if scope.strip().startswith("new:"):
            expanded_new.append(normalized)
        else:
            expanded_allowed.extend(expand_scope_to_existing_files(scope, production_source=production_source))
    manifest_writer_strategy = writer_strategy_for_entry(entry)
    if manifest_writer_strategy != "no_writer":
        writer_strategy = manifest_writer_strategy
    elif future_executor_kind == "proof_aggregation":
        writer_strategy = "proof_aggregation_writer"
    elif future_executor_kind == "product_implementation":
        writer_strategy = "agent_writer"
    elif future_executor_kind == "handoff_closeout":
        writer_strategy = "verification_writer"
    elif future_executor_kind == "design_authoring" and (
        (agent_design_contract_for_entry(entry) is not None)
        and agent_design_contract_for_entry(entry).authoring_strategy == "codex_contract_writer"
    ):
        writer_strategy = "agent_writer"
    else:
        writer_strategy = "verification_writer"
    data = {
        "version": 1,
        "wr_id": wr_id,
        "milestone_id": entry.milestone_id,
        "execution_kind": entry.execution_kind,
        "executor_kind": future_executor_kind,
        "authority_level": entry.authority_level,
        "permissions_required": permissions_for_entry(entry, future_executor_kind),
        "writer_strategy": writer_strategy,
        "allowed_outputs": list(dict.fromkeys(expanded_allowed)),
        "new_outputs": list(dict.fromkeys([*expanded_new, *evidence_paths])),
        "forbidden_outputs": forbidden_outputs_for_entry(entry),
        "forbidden_patterns": list(entry.implementation_writer.forbidden_patterns)
        if entry.implementation_writer is not None
        else [],
        "validation_commands": list(validation_commands_for_entry(entry)),
        "evidence_required": evidence,
        "closeout_contract": {
            "path": entry.expected_closeout_path,
            "completion_quality": closeout_quality_for_entry(entry),
            "evidence_required": evidence,
        },
        "rollback_policy": "reject import and leave repository unchanged on validation, scope, or digest failure",
        "stop_conditions": list(entry.stop_conditions),
    }
    return yaml.safe_dump(data, sort_keys=False, width=4096)


def sidecar_scope_errors(
    *,
    entry,
    plan_contract: PlanContractMetadata,
    allowed_outputs: list[str],
    new_outputs: list[str],
) -> list[str]:
    manifest_outputs = {
        normalized
        for scope in entry.write_scope
        if (normalized := normalized_contract_output(scope)) is not None
    }
    if entry.runtime_closeout_contract is not None:
        if entry.runtime_closeout_contract.evidence_manifest_path:
            evidence_root = normalized_contract_output(entry.runtime_closeout_contract.evidence_manifest_path)
            if evidence_root is not None:
                manifest_outputs.add(evidence_root)
    if entry.handoff_contract is not None:
        handoff_path = normalized_contract_output(entry.expected_closeout_path)
        if handoff_path is not None:
            manifest_outputs.add(handoff_path)
    for requirement in plan_contract.evidence_required:
        if isinstance(requirement, dict):
            paths = requirement.get("paths") or []
        else:
            paths = []
        for raw_path in paths:
            evidence_path = normalized_contract_output(str(raw_path))
            if evidence_path is not None and evidence_path.startswith(
                "docs-site/src/content/docs/reports/execution-evidence/"
            ):
                manifest_outputs.add(evidence_path)
    sidecar_outputs = set(allowed_outputs + new_outputs)
    errors: list[str] = []
    missing = sorted(
        output
        for output in sidecar_outputs
        if not any(path_within_scope(output, scope) for scope in manifest_outputs)
    )
    if missing:
        errors.append(
            f"{entry.milestone_id}: structured plan outputs are outside manifest/WR scope: {', '.join(missing)}"
        )
    return errors


def compile_contract_pack(
    track_id: str,
    *,
    production_source: Path = PRODUCTION_SOURCE,
    roadmap_source: Path = ROADMAP_SOURCE,
    manifest_root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
    contract_pack_root: Path = CONTRACT_PACK_ROOT,
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
        missing_plan_path: Path | None = None
        missing_sidecar_path: Path | None = None
        future_executor_kind: ExecutorKind | None = None
        plan_contract: PlanContractMetadata | None = None
        if entry.owning_wr:
            roadmap_item = roadmap.by_id.get(entry.owning_wr)
            if roadmap_item is None:
                raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is missing from roadmap")
            plan_path = default_contract_path(roadmap_item)
            if not plan_path.exists():
                missing_plan_path = plan_path
                missing_sidecar_path = structured_plan_contract_path(plan_path)
                future_executor_kind = executor_kind_after_plan(entry)
                executor_kind = "design_authoring"
            else:
                authority_paths.append(plan_path)
                plan_contract = load_structured_plan_contract(entry, plan_path, executor_kind)
                sidecar_path = structured_plan_contract_path(plan_path)
                if sidecar_path.exists():
                    authority_paths.append(sidecar_path)
        if executor_kind in {"product_implementation", "proof_aggregation"} and entry.runtime_closeout_contract is None:
            raise WorkflowError(f"{entry.milestone_id}: runtime_closeout_contract is required for runtime proof actions")
        if executor_kind == "planning_expansion":
            allowed_outputs, new_outputs = planning_expansion_outputs(
                track_id=track_id,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source=loaded.path,
                contract_pack_root=contract_pack_root,
            )
            writer_strategy = "no_writer"
            template_outputs = {}
            patches = []
        elif missing_plan_path is not None and missing_sidecar_path is not None and future_executor_kind is not None:
            allowed_outputs = []
            new_outputs = [
                source_path_for_action(missing_plan_path, production_source=production_source),
                source_path_for_action(missing_sidecar_path, production_source=production_source),
            ]
        else:
            allowed_outputs, new_outputs = allowed_outputs_for_entry(entry)
        forbidden_outputs = forbidden_outputs_for_entry(entry)
        forbidden_patterns = list(entry.implementation_writer.forbidden_patterns) if entry.implementation_writer is not None else []
        writer_strategy = writer_strategy_for_entry(entry)
        template_outputs = template_outputs_for_entry(entry)
        patches = patches_for_entry(entry)
        validation_commands = validation_commands_for_entry(entry)
        if executor_kind == "planning_expansion":
            writer_strategy = "no_writer"
            template_outputs = {}
            patches = []
            forbidden_patterns = []
        action_evidence_required = (
            []
            if executor_kind == "planning_expansion"
            else [
                evidence_requirement_from_contract(item)
                for item in sidecar_evidence_entries(entry, track_id=track_id, production_source=production_source)
            ]
        )
        closeout_contract = CloseoutContract(
            path=entry.expected_closeout_path,
            completion_quality="not_applicable" if executor_kind == "planning_expansion" else closeout_quality_for_entry(entry),
            evidence_required=[] if executor_kind == "planning_expansion" else action_evidence_required,
        )
        permissions_required = permissions_for_entry(entry, executor_kind)
        rollback_policy = RollbackPolicy(policy="reject import and leave repository unchanged on validation, scope, or digest failure")
        stop_conditions = list(entry.stop_conditions)
        if missing_plan_path is not None and missing_sidecar_path is not None and future_executor_kind is not None:
            plan_output = source_path_for_action(missing_plan_path, production_source=production_source)
            sidecar_output = source_path_for_action(missing_sidecar_path, production_source=production_source)
            writer_strategy = "template_writer"
            template_outputs = {
                plan_output: plan_authoring_markdown(
                    entry=entry,
                    wr_id=entry.owning_wr,
                    future_executor_kind=future_executor_kind,
                ),
                sidecar_output: plan_authoring_sidecar(
                    track_id=track_id,
                    entry=entry,
                    wr_id=entry.owning_wr,
                    future_executor_kind=future_executor_kind,
                    production_source=production_source,
                ),
            }
            patches = []
            validation_commands = [
                "task production:validate",
                "task roadmap:validate",
                "task docs:validate",
                "task planning:validate",
            ]
            action_evidence_required = []
            closeout_contract = CloseoutContract(
                path=entry.expected_closeout_path,
                completion_quality="not_applicable",
                evidence_required=[],
            )
            permissions_required = ["agent_design"]
            rollback_policy = RollbackPolicy(policy="reject import and leave repository unchanged on validation, scope, or digest failure")
            stop_conditions = [
                "stop after creating the implementation plan and plan.contract.yaml sidecar",
                "stop before product implementation until the generated contract validates",
            ]
        if plan_contract is not None:
            if plan_contract.execution_kind is not None and plan_contract.execution_kind != entry.execution_kind:
                raise WorkflowError(f"{entry.milestone_id}: plan.contract.yaml execution_kind must be {entry.execution_kind}")
            allowed_outputs = normalize_sidecar_allowed_outputs(plan_contract, production_source=production_source)
            new_outputs = normalize_sidecar_new_outputs(plan_contract)
            forbidden_outputs = normalize_sidecar_outputs(plan_contract.forbidden_outputs)
            forbidden_patterns = list(plan_contract.forbidden_patterns)
            writer_strategy = plan_contract.writer_strategy
            template_outputs = template_outputs_from_contract(plan_contract)
            patches = text_patches_from_contract(plan_contract)
            validation_commands = validation_commands_from_contract(plan_contract)
            action_evidence_required = evidence_requirements_from_contract(plan_contract)
            closeout_contract = closeout_contract_from_contract(plan_contract, action_evidence_required)
            permissions_required = list(plan_contract.permissions_required)
            for strategic_permission in ("crate_creation", "foundation_extraction"):
                if strategic_permission in entry.permission_classes_required and strategic_permission not in permissions_required:
                    permissions_required.append(strategic_permission)
            rollback_policy = RollbackPolicy(policy=plan_contract.rollback_policy)
            stop_conditions = list(plan_contract.stop_conditions)
            truth_claim_updates = list(plan_contract.truth_claim_updates)
            errors = sidecar_scope_errors(
                entry=entry,
                plan_contract=plan_contract,
                allowed_outputs=allowed_outputs,
                new_outputs=new_outputs,
            )
            if errors:
                raise WorkflowError("\n".join(errors))
            if (
                executor_kind in {"design_authoring", "product_implementation", "proof_aggregation"}
                and closeout_contract.completion_quality in {"bounded_contract", "runtime_proven", "architecture_runtime_proven"}
                and evidence_requirements_satisfied(action_evidence_required, production_source=production_source)
            ):
                executor_kind = "runtime_closeout"
                writer_strategy = "template_writer"
                template_outputs = {}
                patches = []
                allowed_outputs, new_outputs = runtime_closeout_outputs(
                    track_id=track_id,
                    closeout_path=closeout_contract.path,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    manifest_source=loaded.path,
                    contract_pack_root=contract_pack_root,
                )
                validation_commands = [
                    "task production:render",
                    "task production:validate",
                    "task production:check",
                    "task roadmap:render",
                    "task roadmap:validate",
                    "task roadmap:check",
                    "task docs:validate",
                    "task planning:validate",
                ]
                permissions_required = ["agent_closeout"]
                action_evidence_required = []
            if executor_kind == "handoff_closeout":
                closeout_allowed, closeout_new = runtime_closeout_outputs(
                    track_id=track_id,
                    closeout_path=closeout_contract.path,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    manifest_source=loaded.path,
                    contract_pack_root=contract_pack_root,
                )
                allowed_outputs = list(dict.fromkeys([*allowed_outputs, *closeout_allowed]))
                new_outputs = list(dict.fromkeys([*new_outputs, *closeout_new]))
        derived_pack_outputs = existing_contract_pack_outputs(
            track_id=track_id,
            contract_pack_root=contract_pack_root,
            production_source=production_source,
        )
        allowed_outputs = list(dict.fromkeys([*allowed_outputs, *derived_pack_outputs]))
        action_id = f"{track_id}:{entry.milestone_id}:{wr_id}"
        actions.append(
            ActionContract(
                action_id=action_id,
                track_id=track_id,
                milestone_id=entry.milestone_id,
                wr_id=wr_id,
                execution_kind=entry.execution_kind,
                executor_kind=executor_kind,
                authority_level=plan_contract.authority_level if plan_contract is not None else entry.authority_level,
                permissions_required=permissions_required,
                allowed_outputs=allowed_outputs,
                new_outputs=new_outputs,
                forbidden_outputs=forbidden_outputs,
                forbidden_patterns=forbidden_patterns,
                writer_strategy=writer_strategy,
                template_outputs=template_outputs,
                patches=patches,
                validation_commands=validation_commands,
                evidence_required=action_evidence_required,
                closeout_contract=closeout_contract,
                rollback_policy=rollback_policy,
                stop_conditions=stop_conditions,
                required_prior_milestones=list(entry.implementation_writer.required_prior_milestones)
                if entry.implementation_writer is not None
                else [],
                required_prior_completion_quality=entry.implementation_writer.required_prior_completion_quality
                if entry.implementation_writer is not None
                else None,
                truth_claim_updates=truth_claim_updates if plan_contract is not None else [],
                production_source_path=source_path_for_action(production_source, production_source=production_source),
                roadmap_source_path=source_path_for_action(roadmap_source, production_source=production_source),
                manifest_source_path=source_path_for_action(loaded.path, production_source=production_source),
            )
        )

    return ContractPack(
        track_id=track_id,
        generated_at=now_utc_iso(),
        source_digests={
            **source_digest_map_for_pack(loaded, production_source=production_source, roadmap_source=roadmap_source),
            **authority_source_digest_map(authority_paths),
            **harness_source_digest_map(),
        },
        actions=actions,
    )


def first_action(pack: ContractPack) -> ActionContract | None:
    return pack.actions[0] if pack.actions else None


def harness_source_digest_map() -> dict[str, str]:
    roots = [
        REPO_ROOT / "tools/workflow/execution",
        REPO_ROOT / "tools/workflow/track_sources",
    ]
    files = [
        REPO_ROOT / "tools/workflow/production_track_cli.py",
        REPO_ROOT / "tools/workflow/production_goal.py",
        REPO_ROOT / "tools/workflow/production_state.py",
        REPO_ROOT / "tools/workflow/roadmap_state.py",
    ]
    digests = {
        repo_path(path): sha256(path.read_bytes()).hexdigest()
        for root in roots
        for path in sorted(root.glob("*.py"))
    }
    digests.update(
        {
            repo_path(path): sha256(path.read_bytes()).hexdigest()
            for path in files
            if path.exists()
        }
    )
    return digests


def authority_source_digest_map(paths: list[Path]) -> dict[str, str]:
    unique: dict[str, Path] = {}
    for path in paths:
        unique[repo_path(path)] = path
    return {
        source: sha256(path.read_bytes()).hexdigest()
        for source, path in sorted(unique.items())
        if path.exists()
    }


def source_digest_map_for_pack(
    loaded,
    *,
    production_source: Path,
    roadmap_source: Path,
) -> dict[str, str]:
    return {
        source_path_for_digest(path, production_source=production_source): sha256(path.read_bytes()).hexdigest()
        for path in source_digest_paths(
            loaded,
            production_source=production_source,
            roadmap_source=roadmap_source,
        )
        if path.exists()
    }


def source_path_for_action(path: Path, *, production_source: Path) -> str:
    resolved = path.resolve()
    try:
        return resolved.relative_to(REPO_ROOT.resolve()).as_posix()
    except ValueError:
        pass
    try:
        return resolved.relative_to(source_root_for_action_paths(production_source).resolve()).as_posix()
    except ValueError:
        return resolved.as_posix()


def source_path_for_digest(path: Path, *, production_source: Path) -> str:
    resolved = path.resolve()
    try:
        return resolved.relative_to(REPO_ROOT.resolve()).as_posix()
    except ValueError:
        pass
    standard_root = standard_repo_source_root(production_source)
    if standard_root is not None:
        try:
            return resolved.relative_to(standard_root.resolve()).as_posix()
        except ValueError:
            pass
    return resolved.as_posix()
