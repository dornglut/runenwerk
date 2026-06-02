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
    AgentSubActionMetadata,
    PlanContractMetadata,
    load_plan_contract_metadata,
    structured_plan_contract_path,
)
from truth.certificates import digest_path

from execution.contracts import (
    ActionContract,
    AgentSubActionContract,
    CloseoutContract,
    ContractPack,
    EvidenceRequirement,
    ExecutorKind,
    RollbackPolicy,
    TextPatch,
    now_utc_iso,
    validation_command_from_string,
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


def truth_certificate_output(track_id: str, claim_id: str) -> str:
    return f"docs-site/src/content/docs/reports/truth-certificates/{track_id.lower()}/{claim_id}.yaml"


def validation_command_argv(value) -> list[str]:
    if isinstance(value, str):
        parsed = validation_command_from_string(value)
        argv = parsed.get("argv") if isinstance(parsed, dict) else None
        return argv if isinstance(argv, list) else []
    if isinstance(value, dict):
        argv = value.get("argv")
        return argv if isinstance(argv, list) else []
    if hasattr(value, "argv"):
        return list(value.argv)
    return []


def with_truth_certificate_outputs(validation_commands, *, track_id: str) -> list:
    normalized = []
    for command in validation_commands:
        argv = validation_command_argv(command)
        output: str | None = None
        if (
            len(argv) == 7
            and argv[0] == "task"
            and argv[1] in {"truth:certify", "truth:verify"}
            and argv[2] == "--"
            and argv[3] == "--track"
            and argv[4] == track_id
            and argv[5] == "--claim"
            and isinstance(argv[6], str)
            and argv[6].strip()
        ):
            output = truth_certificate_output(track_id, argv[6].strip())
        if output is None:
            normalized.append(command)
            continue
        if isinstance(command, str):
            data = validation_command_from_string(command)
        elif hasattr(command, "model_dump"):
            data = command.model_dump(mode="json")
        elif isinstance(command, dict):
            data = dict(command)
        else:
            data = validation_command_from_string(str(command))
        allowed_outputs = list(data.get("allowed_outputs") or [])
        if output not in allowed_outputs:
            allowed_outputs.append(output)
        data["allowed_outputs"] = allowed_outputs
        normalized.append(data)
    return normalized


def with_render_command_outputs(
    validation_commands,
    *,
    production_source: Path,
    roadmap_source: Path,
) -> list:
    generated_allowed, _generated_new = generated_outputs_for_sources(
        production_source=production_source,
        roadmap_source=roadmap_source,
    )
    normalized = []
    for command in validation_commands:
        argv = validation_command_argv(command)
        if not (len(argv) == 2 and argv[0] == "task" and argv[1] in {"production:render", "roadmap:render"}):
            normalized.append(command)
            continue
        if isinstance(command, str):
            data = validation_command_from_string(command)
        elif hasattr(command, "model_dump"):
            data = command.model_dump(mode="json")
        elif isinstance(command, dict):
            data = dict(command)
        else:
            data = validation_command_from_string(str(command))
        allowed_outputs = list(data.get("allowed_outputs") or [])
        for output in generated_allowed:
            if output not in allowed_outputs:
                allowed_outputs.append(output)
        data["allowed_outputs"] = allowed_outputs
        normalized.append(data)
    return normalized


def normalize_validation_command_outputs(
    validation_commands,
    *,
    track_id: str,
    production_source: Path,
    roadmap_source: Path,
) -> list:
    commands = with_truth_certificate_outputs(validation_commands, track_id=track_id)
    return with_render_command_outputs(
        commands,
        production_source=production_source,
        roadmap_source=roadmap_source,
    )


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


def classify_existing_outputs(
    outputs: list[str],
    *,
    production_source: Path,
) -> tuple[list[str], list[str]]:
    source_root = source_root_for_action_paths(production_source)
    allowed: list[str] = []
    new: list[str] = []
    for output in outputs:
        target = source_root / output
        if target.exists():
            allowed.append(output)
        else:
            new.append(output)
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


def normalize_subaction_allowed_outputs(
    subaction: AgentSubActionMetadata,
    *,
    production_source: Path,
) -> list[str]:
    outputs: list[str] = []
    for scope in subaction.allowed_outputs:
        if scope.strip().startswith("new:"):
            raise WorkflowError(
                f"{subaction.sub_action_id}: agent_subactions.allowed_outputs must not use new: scope markers"
            )
        outputs.extend(expand_scope_to_existing_files(scope, production_source=production_source))
    return list(dict.fromkeys(outputs))


def normalize_subaction_new_outputs(subaction: AgentSubActionMetadata) -> list[str]:
    outputs: list[str] = []
    for scope in subaction.new_outputs:
        marked = scope if scope.strip().startswith("new:") else f"new: {scope}"
        normalized = normalized_contract_output(marked)
        if normalized is not None:
            outputs.append(normalized)
    return list(dict.fromkeys(outputs))


def validation_commands_from_subaction(
    plan_contract: PlanContractMetadata,
    subaction: AgentSubActionMetadata,
) -> list:
    if not subaction.validation_commands:
        raise WorkflowError(
            f"{plan_contract.milestone_id}: agent_subaction {subaction.sub_action_id} validation_commands must not be empty"
        )
    return list(subaction.validation_commands)


def evidence_requirements_from_subaction(
    plan_contract: PlanContractMetadata,
    subaction: AgentSubActionMetadata,
) -> list[EvidenceRequirement]:
    if not subaction.evidence_required:
        raise WorkflowError(
            f"{plan_contract.milestone_id}: agent_subaction {subaction.sub_action_id} evidence_required must not be empty"
        )
    return [evidence_requirement_from_contract(item) for item in subaction.evidence_required]


def stop_conditions_from_subaction(
    plan_contract: PlanContractMetadata,
    subaction: AgentSubActionMetadata,
) -> list[str]:
    if not subaction.stop_conditions:
        raise WorkflowError(
            f"{plan_contract.milestone_id}: agent_subaction {subaction.sub_action_id} stop_conditions must not be empty"
        )
    return list(subaction.stop_conditions)


def subaction_scope_errors(
    *,
    entry,
    plan_contract: PlanContractMetadata,
    subaction: AgentSubActionMetadata,
    allowed_outputs: list[str],
    new_outputs: list[str],
    production_source: Path,
) -> list[str]:
    errors = sidecar_scope_errors(
        entry=entry,
        plan_contract=plan_contract,
        allowed_outputs=allowed_outputs,
        new_outputs=new_outputs,
    )
    if not allowed_outputs and not new_outputs:
        errors.append(f"{entry.milestone_id}: agent_subaction {subaction.sub_action_id} must declare exact outputs")
    parent_outputs = set(
        normalize_sidecar_allowed_outputs(plan_contract, production_source=production_source)
        + normalize_sidecar_new_outputs(plan_contract)
    )
    for output in [*allowed_outputs, *new_outputs]:
        if output not in parent_outputs:
            errors.append(
                f"{entry.milestone_id}: agent_subaction {subaction.sub_action_id} "
                f"output is outside plan.contract.yaml outputs: {output}"
            )
    return errors


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


def evidence_record_satisfied(record_path: Path, requirement: EvidenceRequirement, *, root: Path) -> bool:
    try:
        data = yaml.safe_load(record_path.read_text(encoding="utf-8")) or {}
    except (OSError, yaml.YAMLError):
        return False
    if not isinstance(data, dict):
        return False
    if data.get("status") != "passed":
        return False
    if data.get("evidence_kind") != requirement.kind:
        return False

    subject_paths = [str(path).strip() for path in (data.get("subject_paths") or []) if str(path).strip()]
    required_subjects = {path.strip() for path in requirement.subject_paths if path.strip()}
    if required_subjects and not required_subjects.issubset(set(subject_paths)):
        return False
    subject_digests = data.get("subject_digests") or {}
    if not isinstance(subject_digests, dict):
        return False
    for subject in subject_paths:
        subject_path = root / subject
        if not subject_path.exists():
            return False
        if subject_digests.get(subject) != digest_path(subject_path):
            return False

    provenance = data.get("validation_provenance") or []
    if not isinstance(provenance, list) or not provenance:
        return False
    for item in provenance:
        if not isinstance(item, dict):
            return False
        if item.get("returncode") != 0:
            return False
        if not item.get("command_id") or not item.get("argv") or not item.get("run_action_id"):
            return False
        if not item.get("validation_result_digest"):
            return False
        ledger_path = item.get("run_ledger_path")
        if not isinstance(ledger_path, str) or not ledger_path.strip() or not (root / ledger_path).exists():
            return False
        provenance_subject_digests = item.get("subject_digests") or {}
        if not isinstance(provenance_subject_digests, dict):
            return False
        for subject, recorded_digest in provenance_subject_digests.items():
            subject_path = root / str(subject)
            if not subject_path.exists() or recorded_digest != digest_path(subject_path):
                return False
    return True


def evidence_requirements_satisfied(requirements: list[EvidenceRequirement], *, production_source: Path) -> bool:
    root = source_root_for_action_paths(production_source)
    for requirement in requirements:
        if requirement.required and not requirement.paths:
            return False
        for evidence_path in requirement.paths:
            record_path = root / evidence_path
            if not record_path.exists():
                return False
            if not evidence_record_satisfied(record_path, requirement, root=root):
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
    invalid_validation_commands = [
        (command, parsed.get("blocked_reason"))
        for command in validation_commands_for_entry(entry)
        if (parsed := validation_command_from_string(command)).get("command_id") == "blocked"
    ]
    if invalid_validation_commands:
        details = "; ".join(f"{command!r}: {reason}" for command, reason in invalid_validation_commands)
        raise WorkflowError(f"{entry.milestone_id}: cannot generate plan.contract.yaml with invalid validation command(s): {details}")
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
        "status": "draft",
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
    for subaction in plan_contract.agent_subactions:
        for requirement in subaction.evidence_required:
            paths = requirement.get("paths") if isinstance(requirement, dict) else []
            for raw_path in paths or []:
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


def sidecar_authority_errors(
    *,
    entry,
    roadmap_item,
    plan_contract: PlanContractMetadata,
) -> list[str]:
    errors: list[str] = []
    requested_permissions = set(plan_contract.permissions_required)
    declared_permissions = set(entry.permission_classes_required)
    product_permissions = {"product_code", "product_implementation"}
    if requested_permissions & product_permissions:
        if not entry.may_create_code:
            errors.append(f"{entry.milestone_id}: plan.contract.yaml requests product code but manifest may_create_code is false")
        if not entry.may_modify_production_behavior:
            errors.append(
                f"{entry.milestone_id}: plan.contract.yaml requests product behavior changes but manifest may_modify_production_behavior is false"
            )
        missing_permissions = sorted((requested_permissions & product_permissions) - declared_permissions)
        if missing_permissions:
            errors.append(
                f"{entry.milestone_id}: plan.contract.yaml requests permissions outside manifest permission_classes_required: "
                + ", ".join(missing_permissions)
            )
        if roadmap_item.planning_state != "current_candidate":
            errors.append(
                f"{entry.milestone_id}: product implementation requires owning WR {roadmap_item.id} "
                f"to be current_candidate; got {roadmap_item.planning_state}"
            )
        if roadmap_item.blocker > 2:
            errors.append(
                f"{entry.milestone_id}: product implementation requires owning WR {roadmap_item.id} "
                f"to be B2 or lower; got B{roadmap_item.blocker}"
            )
    if "crate_creation" in requested_permissions and not entry.may_create_crates:
        errors.append(f"{entry.milestone_id}: plan.contract.yaml requests crate_creation but manifest may_create_crates is false")
    if "foundation_extraction" in requested_permissions:
        errors.append(f"{entry.milestone_id}: plan.contract.yaml must not request foundation_extraction")
    non_strategic_permissions = requested_permissions - {"crate_creation", "foundation_extraction"}
    extra_declared = sorted(non_strategic_permissions - declared_permissions - {"agent_design", "agent_closeout"})
    if extra_declared and not (set(extra_declared) <= product_permissions and product_permissions <= declared_permissions):
        errors.append(
            f"{entry.milestone_id}: plan.contract.yaml requests undeclared permissions: " + ", ".join(extra_declared)
        )
    return errors


def draft_plan_action(
    *,
    entry,
    plan_path: Path,
    sidecar_path: Path,
    production_source: Path,
) -> dict:
    plan_output = source_path_for_action(plan_path, production_source=production_source)
    sidecar_output = source_path_for_action(sidecar_path, production_source=production_source)
    return {
        "executor_kind": "design_authoring",
        "allowed_outputs": [plan_output, sidecar_output],
        "new_outputs": [],
        "forbidden_outputs": forbidden_outputs_for_entry(entry),
        "forbidden_patterns": list(entry.implementation_writer.forbidden_patterns)
        if entry.implementation_writer is not None
        else [],
        "writer_strategy": "verification_writer",
        "template_outputs": {},
        "patches": [],
        "validation_commands": [
            "task production:validate",
            "task roadmap:validate",
            "task docs:validate",
            "task planning:validate",
        ],
        "action_evidence_required": [],
        "closeout_contract": CloseoutContract(
            path=entry.expected_closeout_path,
            completion_quality="not_applicable",
            evidence_required=[],
        ),
        "permissions_required": ["agent_design"],
        "rollback_policy": RollbackPolicy(policy="reject import and leave repository unchanged on validation, scope, or digest failure"),
        "stop_conditions": [
            "plan.contract.yaml is draft authority and cannot execute product code",
            "accept only after exact scopes, evidence, validation commands, WR state, and manifest permissions align",
        ],
    }


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
                sidecar_path = structured_plan_contract_path(plan_path)
                plan_contract = load_structured_plan_contract(entry, plan_path, executor_kind)
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
            if plan_contract.status == "draft":
                if not entry.owning_wr:
                    raise WorkflowError(f"{entry.milestone_id}: draft plan.contract.yaml requires owning_wr")
                draft_action = draft_plan_action(
                    entry=entry,
                    plan_path=plan_path,
                    sidecar_path=sidecar_path,
                    production_source=production_source,
                )
                executor_kind = draft_action["executor_kind"]
                allowed_outputs = draft_action["allowed_outputs"]
                new_outputs = draft_action["new_outputs"]
                forbidden_outputs = draft_action["forbidden_outputs"]
                forbidden_patterns = draft_action["forbidden_patterns"]
                writer_strategy = draft_action["writer_strategy"]
                template_outputs = draft_action["template_outputs"]
                patches = draft_action["patches"]
                validation_commands = draft_action["validation_commands"]
                action_evidence_required = draft_action["action_evidence_required"]
                closeout_contract = draft_action["closeout_contract"]
                permissions_required = draft_action["permissions_required"]
                rollback_policy = draft_action["rollback_policy"]
                stop_conditions = draft_action["stop_conditions"]
                truth_claim_updates = []
                plan_contract = None
            else:
                if roadmap_item is None:
                    raise WorkflowError(f"{entry.milestone_id}: accepted plan.contract.yaml requires an owning WR")
                authority_errors = sidecar_authority_errors(
                    entry=entry,
                    roadmap_item=roadmap_item,
                    plan_contract=plan_contract,
                )
                if authority_errors:
                    raise WorkflowError("\n".join(authority_errors))
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
                and closeout_contract.completion_quality
                in {
                    "bounded_contract",
                    "runtime_proven",
                    "proof_slice_runtime_proven",
                    "architecture_runtime_proven",
                    "perfectionist_verified",
                }
                and evidence_requirements_satisfied(action_evidence_required, production_source=production_source)
            ):
                sidecar_runtime_closeout_outputs = [
                    output
                    for output in [*allowed_outputs, *new_outputs]
                    if output.startswith("docs-site/src/content/docs/reports/truth-certificates/")
                ]
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
                certificate_allowed, certificate_new = classify_existing_outputs(
                    sidecar_runtime_closeout_outputs,
                    production_source=production_source,
                )
                allowed_outputs = list(dict.fromkeys([*allowed_outputs, *certificate_allowed]))
                new_outputs = list(dict.fromkeys([*new_outputs, *certificate_new]))
                validation_commands = list(entry.runtime_closeout_contract.validation_commands) if entry.runtime_closeout_contract else [
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
        base_action_kwargs = {
            "track_id": track_id,
            "milestone_id": entry.milestone_id,
            "wr_id": wr_id,
            "execution_kind": entry.execution_kind,
            "executor_kind": executor_kind,
            "authority_level": plan_contract.authority_level if plan_contract is not None else entry.authority_level,
            "permissions_required": permissions_required,
            "forbidden_outputs": forbidden_outputs,
            "forbidden_patterns": forbidden_patterns,
            "writer_strategy": writer_strategy,
            "template_outputs": template_outputs,
            "patches": patches,
            "rollback_policy": rollback_policy,
            "required_prior_milestones": list(entry.implementation_writer.required_prior_milestones)
            if entry.implementation_writer is not None
            else [],
            "required_prior_completion_quality": entry.implementation_writer.required_prior_completion_quality
            if entry.implementation_writer is not None
            else None,
            "truth_claim_updates": truth_claim_updates if plan_contract is not None else [],
            "production_source_path": source_path_for_action(production_source, production_source=production_source),
            "roadmap_source_path": source_path_for_action(roadmap_source, production_source=production_source),
            "manifest_source_path": source_path_for_action(loaded.path, production_source=production_source),
        }
        if plan_contract is not None and plan_contract.agent_subactions and writer_strategy == "agent_writer":
            emitted_subactions = 0
            parent_outputs = set(allowed_outputs + new_outputs)
            for subaction in plan_contract.agent_subactions:
                scoped_subaction_allowed_outputs = normalize_subaction_allowed_outputs(
                    subaction,
                    production_source=production_source,
                )
                scoped_subaction_new_outputs = normalize_subaction_new_outputs(subaction)
                subaction_validation_commands = validation_commands_from_subaction(plan_contract, subaction)
                subaction_evidence_required = evidence_requirements_from_subaction(plan_contract, subaction)
                subaction_stop_conditions = stop_conditions_from_subaction(plan_contract, subaction)
                subaction_scope_missing = sorted(
                    output
                    for output in [*scoped_subaction_allowed_outputs, *scoped_subaction_new_outputs]
                    if output not in parent_outputs
                )
                if subaction_scope_missing:
                    raise WorkflowError(
                        f"{entry.milestone_id}: agent_subaction {subaction.sub_action_id} "
                        "outputs are outside parent plan.contract.yaml authority: "
                        + ", ".join(subaction_scope_missing)
                    )
                errors = subaction_scope_errors(
                    entry=entry,
                    plan_contract=plan_contract,
                    subaction=subaction,
                    allowed_outputs=scoped_subaction_allowed_outputs,
                    new_outputs=scoped_subaction_new_outputs,
                    production_source=production_source,
                )
                if errors:
                    raise WorkflowError("\n".join(errors))
                if evidence_requirements_satisfied(subaction_evidence_required, production_source=production_source):
                    continue
                subaction_allowed_outputs = list(
                    dict.fromkeys([*scoped_subaction_allowed_outputs, *derived_pack_outputs])
                )
                subaction_permissions = list(permissions_required)
                if "crate_creation" in subaction_permissions and not any(
                    output.endswith("Cargo.toml") for output in scoped_subaction_new_outputs
                ):
                    subaction_permissions = [
                        permission for permission in subaction_permissions if permission != "crate_creation"
                    ]
                subaction_validation_commands = normalize_validation_command_outputs(
                    subaction_validation_commands,
                    track_id=track_id,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                )
                subaction_kwargs = dict(base_action_kwargs)
                subaction_kwargs["permissions_required"] = subaction_permissions
                subaction_action_id = f"{action_id}:sub:{subaction.sub_action_id}"
                actions.append(
                    ActionContract(
                        action_id=subaction_action_id,
                        allowed_outputs=subaction_allowed_outputs,
                        new_outputs=scoped_subaction_new_outputs,
                        validation_commands=subaction_validation_commands,
                        evidence_required=subaction_evidence_required,
                        closeout_contract=CloseoutContract(
                            path=closeout_contract.path,
                            completion_quality=closeout_contract.completion_quality,
                            evidence_required=subaction_evidence_required,
                        ),
                        stop_conditions=subaction_stop_conditions,
                        parent_action_id=action_id,
                        agent_subaction=AgentSubActionContract(
                            sub_action_id=subaction.sub_action_id,
                            title=subaction.title,
                            prompt=subaction.prompt,
                        ),
                        **subaction_kwargs,
                    )
                )
                emitted_subactions += 1
            if emitted_subactions:
                continue
        validation_commands = normalize_validation_command_outputs(
            validation_commands,
            track_id=track_id,
            production_source=production_source,
            roadmap_source=roadmap_source,
        )
        actions.append(
            ActionContract(
                action_id=action_id,
                allowed_outputs=allowed_outputs,
                new_outputs=new_outputs,
                validation_commands=validation_commands,
                evidence_required=action_evidence_required,
                closeout_contract=closeout_contract,
                stop_conditions=stop_conditions,
                **base_action_kwargs,
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
