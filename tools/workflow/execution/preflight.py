from __future__ import annotations

import re
from pathlib import PurePosixPath

from execution.contracts import ActionContract, ContractPack


BROAD_OUTPUTS = {
    ".",
    "docs-site",
    "docs-site/src",
    "docs-site/src/content",
    "docs-site/src/content/docs",
    "domain",
    "domain/ui",
    "tools",
    "tools/workflow",
}


def validation_command_errors(action: ActionContract) -> list[str]:
    errors: list[str] = []
    for command in action.validation_commands:
        label = command.raw or " ".join(command.argv)
        if command.blocked_reason:
            errors.append(f"{action.action_id}: invalid validation command {label!r}: {command.blocked_reason}")
        if command.command_id == "blocked":
            errors.append(f"{action.action_id}: validation command is blocked: {label!r}")
        cwd = PurePosixPath(command.cwd)
        if cwd.is_absolute() or ".." in cwd.parts:
            errors.append(f"{action.action_id}: validation command cwd must be repo-relative and contained: {command.cwd}")
        for output in command.allowed_outputs:
            normalized = output.strip().strip("/")
            if normalized in BROAD_OUTPUTS or normalized.endswith("/"):
                errors.append(f"{action.action_id}: validation command allowed_output is too broad: {output}")
    return errors


def output_scope_errors(action: ActionContract) -> list[str]:
    errors: list[str] = []
    outputs = [*action.allowed_outputs, *action.new_outputs]
    for output in outputs:
        normalized = output.strip().strip("/")
        if not normalized:
            errors.append(f"{action.action_id}: output path must not be empty")
        if normalized in BROAD_OUTPUTS or normalized.endswith("/"):
            errors.append(f"{action.action_id}: output path is too broad: {output}")
        if normalized.startswith("foundation/meta"):
            errors.append(f"{action.action_id}: foundation/meta output is forbidden")
    if action.writer_strategy == "agent_writer" and not outputs:
        errors.append(f"{action.action_id}: agent_writer requires explicit allowed/new outputs")
    return errors


def writer_errors(action: ActionContract) -> list[str]:
    errors: list[str] = []
    writer_backed_executors = {
        "design_authoring",
        "product_implementation",
        "runtime_closeout",
        "handoff_closeout",
        "truth_claim_update",
    }
    if action.executor_kind in writer_backed_executors and action.writer_strategy == "no_writer":
        errors.append(f"{action.action_id}: {action.executor_kind} requires a declared writer strategy")
    if action.executor_kind in writer_backed_executors and not action.allowed_outputs and not action.new_outputs:
        errors.append(f"{action.action_id}: {action.executor_kind} requires exact allowed/new outputs")
    if action.executor_kind == "planning_expansion" and action.writer_strategy != "no_writer":
        errors.append(f"{action.action_id}: planning_expansion must not run a product/design writer")
    if (
        action.executor_kind != "planning_expansion"
        and action.execution_kind in {"implementation_proof", "proof_aggregation"}
        and action.writer_strategy == "no_writer"
    ):
        errors.append(f"{action.action_id}: implementation/proof action cannot use no_writer")
    if action.execution_kind == "proof_aggregation" and action.writer_strategy != "proof_aggregation_writer":
        errors.append(f"{action.action_id}: proof_aggregation requires proof_aggregation_writer")
    if action.executor_kind == "proof_aggregation" and action.writer_strategy != "proof_aggregation_writer":
        errors.append(f"{action.action_id}: proof_aggregation executor requires proof_aggregation_writer")
    if action.writer_strategy == "agent_writer" and "product_implementation" not in action.permissions_required:
        errors.append(f"{action.action_id}: agent_writer requires product_implementation permission")
    return errors


def crate_creation_errors(action: ActionContract) -> list[str]:
    errors: list[str] = []
    crate_outputs = [path for path in action.new_outputs if path.endswith("Cargo.toml")]
    if crate_outputs and "crate_creation" not in action.permissions_required:
        errors.append(f"{action.action_id}: new Cargo.toml output requires crate_creation permission")
    if "crate_creation" in action.permissions_required and not crate_outputs:
        errors.append(f"{action.action_id}: crate_creation permission requires exact new Cargo.toml output")
    return errors


def forbidden_pattern_errors(action: ActionContract) -> list[str]:
    errors: list[str] = []
    for pattern in action.forbidden_patterns:
        try:
            re.compile(pattern)
        except re.error as error:
            errors.append(f"{action.action_id}: invalid forbidden pattern {pattern!r}: {error}")
    return errors


def evidence_errors(action: ActionContract) -> list[str]:
    errors: list[str] = []
    if (
        action.executor_kind in {"product_implementation", "proof_aggregation"}
        and action.execution_kind in {"implementation_proof", "proof_aggregation"}
        and not action.evidence_required
    ):
        errors.append(f"{action.action_id}: runtime/proof action requires machine-readable evidence requirements")
    if action.execution_kind == "proof_aggregation" and not action.required_prior_milestones:
        errors.append(f"{action.action_id}: proof_aggregation requires required_prior_milestones")
    if action.execution_kind == "proof_aggregation" and action.required_prior_completion_quality != "runtime_proven":
        errors.append(f"{action.action_id}: proof_aggregation requires prior runtime_proven completion quality")
    if action.closeout_contract.completion_quality in {"runtime_proven", "architecture_runtime_proven"}:
        if not action.closeout_contract.evidence_required:
            errors.append(f"{action.action_id}: runtime/architecture closeout requires evidence metadata")
    for requirement in action.evidence_required:
        if not requirement.paths:
            errors.append(f"{action.action_id}: {requirement.kind} evidence requires declared evidence output path")
        for path in requirement.paths:
            if path not in action.allowed_outputs and path not in action.new_outputs:
                errors.append(f"{action.action_id}: evidence output is not declared in action outputs: {path}")
        if requirement.kind == "runtime_test" and not requirement.validation_command_ids:
            errors.append(f"{action.action_id}: runtime_test evidence requires validation_command_ids")
    return errors


def preflight_action(action: ActionContract) -> list[str]:
    errors: list[str] = []
    errors.extend(validation_command_errors(action))
    errors.extend(output_scope_errors(action))
    errors.extend(writer_errors(action))
    errors.extend(crate_creation_errors(action))
    errors.extend(forbidden_pattern_errors(action))
    errors.extend(evidence_errors(action))
    if "foundation_extraction" in action.permissions_required:
        errors.append(f"{action.action_id}: foundation_extraction is a strategic human gate")
    if action.executor_kind == "extraction_gate":
        errors.append(f"{action.action_id}: extraction_gate is a strategic human gate")
    return errors


def preflight_actions(pack: ContractPack, *, actions: list[ActionContract], allow: set[str] | None = None) -> list[str]:
    errors: list[str] = []
    if not pack.source_digests:
        errors.append(f"{pack.track_id}: Contract Pack source_digests must not be empty")
    for action in actions:
        errors.extend(preflight_action(action))
        if allow is not None:
            missing = sorted(set(action.permissions_required) - allow)
            if missing:
                errors.append(f"{action.action_id}: ungranted permissions: {', '.join(missing)}")
    return errors


def preflight_pack(pack: ContractPack, *, allow: set[str] | None = None) -> list[str]:
    return preflight_actions(pack, actions=list(pack.actions), allow=allow)


def preflight_for_mode(pack: ContractPack, *, mode: str, allow: set[str] | None = None) -> list[str]:
    if mode == "full-track":
        return preflight_pack(pack, allow=allow)
    if mode == "single-action":
        return preflight_actions(pack, actions=list(pack.actions[:1]), allow=allow)
    return [f"{pack.track_id}: unsupported execution mode for preflight: {mode}"]
