from __future__ import annotations

import shlex
from datetime import UTC, datetime
from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator


ExecutionKind = Literal[
    "design_contract",
    "implementation_proof",
    "proof_aggregation",
    "handoff_closeout",
    "extraction_gate",
]

ExecutorKind = Literal[
    "planning_expansion",
    "design_authoring",
    "product_implementation",
    "proof_aggregation",
    "runtime_closeout",
    "handoff_closeout",
    "truth_claim_update",
    "extraction_gate",
]

Permission = Literal[
    "auto_safe",
    "agent_design",
    "agent_closeout",
    "product_code",
    "product_implementation",
    "runtime_closeout",
    "handoff",
    "crate_creation",
    "foundation_extraction",
]

WriterStrategy = Literal[
    "no_writer",
    "template_writer",
    "patch_writer",
    "proof_aggregation_writer",
    "agent_writer",
    "verification_writer",
]

EvidenceKind = Literal[
    "runtime_test",
    "fixture",
    "diagnostics",
    "source_maps",
    "artifact",
    "migration",
    "reproducibility",
    "visual",
    "handoff",
]

CompletionQuality = Literal[
    "not_applicable",
    "bounded_contract",
    "runtime_proven",
    "proof_slice_runtime_proven",
    "architecture_runtime_proven",
    "perfectionist_verified",
]

SHELL_META_CHARS = (";", "&&", "||", "|", ">", "<", "`", "$", "\n", "\r")
NON_EXECUTABLE_MARKERS = (
    "run relevant tests",
    "relevant tests",
    "manual validation",
    "validate manually",
    "tbd",
    "to be decided",
    "named by",
)
SAFE_TASK_NAMES = {
    "docs:validate",
    "planning:validate",
    "production:check",
    "production:render",
    "production:validate",
    "roadmap:check",
    "roadmap:render",
    "roadmap:validate",
    "ui:dependencies",
    "workflow:test",
}
SAFE_TASK_NAMES_WITH_ARGS = {
    "truth:audit",
    "truth:certify",
    "truth:post-completion-audit",
    "truth:verify",
}


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_assignment=True)


def clean_text(value: str) -> str:
    cleaned = value.strip()
    if not cleaned:
        raise ValueError("text fields must not be empty")
    return cleaned


class EvidenceRequirement(StrictModel):
    kind: EvidenceKind
    name: str
    required: bool = True
    paths: list[str] = Field(default_factory=list)
    subject_paths: list[str] = Field(default_factory=list)
    validation_command_ids: list[str] = Field(default_factory=list)

    @field_validator("name")
    @classmethod
    def validate_name(cls, value: str) -> str:
        return clean_text(value)

    @field_validator("paths", "subject_paths", "validation_command_ids")
    @classmethod
    def validate_paths(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @model_validator(mode="after")
    def validate_resolver_contract(self) -> EvidenceRequirement:
        if not self.paths:
            raise ValueError(f"{self.name}: evidence requirement must declare an output record path")
        if self.kind == "runtime_test":
            if not self.validation_command_ids:
                raise ValueError(f"{self.name}: runtime_test evidence requires validation_command_ids")
            return self
        if not self.subject_paths:
            raise ValueError(f"{self.name}: {self.kind} evidence requires subject_paths")
        return self


class CloseoutContract(StrictModel):
    path: str
    completion_quality: CompletionQuality
    evidence_required: list[EvidenceRequirement] = Field(default_factory=list)

    @field_validator("path")
    @classmethod
    def validate_path(cls, value: str) -> str:
        return clean_text(value)


class RollbackPolicy(StrictModel):
    policy: str

    @field_validator("policy")
    @classmethod
    def validate_policy(cls, value: str) -> str:
        return clean_text(value)


class TextPatch(StrictModel):
    path: str
    find: str
    replace: str

    @field_validator("path", "find")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        return clean_text(value)


class AgentSubActionContract(StrictModel):
    sub_action_id: str
    title: str
    prompt: str

    @field_validator("sub_action_id", "title", "prompt")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        return clean_text(value)


class ValidationCommand(StrictModel):
    command_id: str
    argv: list[str]
    cwd: str = "."
    allowed_outputs: list[str] = Field(default_factory=list)
    timeout_seconds: int = 600
    raw: str | None = None
    blocked_reason: str | None = None

    @field_validator("command_id", "cwd")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        return clean_text(value)

    @field_validator("argv", "allowed_outputs")
    @classmethod
    def validate_text_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("timeout_seconds")
    @classmethod
    def validate_timeout(cls, value: int) -> int:
        if value <= 0:
            raise ValueError("timeout_seconds must be positive")
        return value

    @field_validator("blocked_reason")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        return clean_text(value) if value is not None else None

    @model_validator(mode="after")
    def validate_command(self) -> ValidationCommand:
        if not self.argv:
            raise ValueError(f"{self.command_id}: argv must not be empty")
        if self.command_id != "blocked":
            expected = validation_command_id(self.argv)
            if expected is None:
                raise ValueError(
                    f"{self.command_id}: argv is not in the safe command registry: {' '.join(self.argv)}"
                )
            if expected != self.command_id:
                raise ValueError(
                    f"{self.command_id}: command_id must match argv-derived id {expected}"
                )
        return self


def blocked_validation_command(raw: str, reason: str) -> dict:
    return {
        "command_id": "blocked",
        "argv": ["blocked"],
        "cwd": ".",
        "raw": raw,
        "blocked_reason": reason,
    }


def validation_command_id(argv: list[str]) -> str | None:
    executable = argv[0]
    if argv == ["uv", "run", "pytest", "tools/workflow/test_workflow.py", "-q"]:
        return None
    if executable == "task" and len(argv) >= 2 and argv[1] in SAFE_TASK_NAMES_WITH_ARGS:
        return truth_task_command_id(argv)
    if executable == "task" and len(argv) == 2 and argv[1] in SAFE_TASK_NAMES:
        return f"task:{argv[1]}"
    if executable == "cargo" and argv[1:] in (
        ["fmt"],
        ["fmt", "--all"],
        ["fmt", "--all", "--check"],
        ["fmt", "--all", "--", "--check"],
    ):
        return "cargo:fmt"
    if executable == "cargo" and len(argv) >= 2 and argv[1] in {"test", "check", "clippy"}:
        return f"cargo:{argv[1]}"
    if executable == "uv" and len(argv) >= 3 and argv[1] == "run":
        if argv[2] == "pytest":
            return "uv:pytest"
        return None
    if executable in {"python", "python3"}:
        if len(argv) >= 3 and argv[1] == "-m" and argv[2] == "pytest":
            return f"{executable}:pytest"
        if len(argv) >= 2 and argv[1] == "--version":
            return f"{executable}:version"
        return None
    if executable == "pytest":
        return "pytest"
    if executable in {"npm", "pnpm"} and len(argv) == 2 and argv[1] == "test":
        return f"{executable}:{argv[1]}"
    if executable == "bun" and len(argv) >= 2 and argv[1] == "test":
        return "bun:test"
    if executable == "npx" and len(argv) >= 2 and argv[1] in {"vitest", "playwright"}:
        return f"npx:{argv[1]}"
    if executable == "git" and argv in (["git", "diff", "--check"], ["git", "status", "--short"]):
        return "git:" + ":".join(argv[1:])
    return None


def truth_task_command_id(argv: list[str]) -> str | None:
    task_name = argv[1]
    if task_name in {"truth:audit", "truth:post-completion-audit"}:
        if len(argv) == 5 and argv[2] == "--" and argv[3] == "--track" and argv[4].strip():
            return f"task:{task_name}"
        return None
    if task_name in {"truth:certify", "truth:verify"}:
        if (
            len(argv) == 7
            and argv[2] == "--"
            and argv[3] == "--track"
            and argv[4].strip()
            and argv[5] == "--claim"
            and argv[6].strip()
        ):
            return f"task:{task_name}"
        return None
    return None


def validation_command_from_string(command: str) -> dict:
    raw = command.strip()
    if not raw:
        return blocked_validation_command(command, "validation command is empty")
    normalized = raw.lower()
    if any(marker in normalized for marker in NON_EXECUTABLE_MARKERS):
        return blocked_validation_command(raw, "validation command is prose/non-executable")
    if any(marker in raw for marker in SHELL_META_CHARS):
        return blocked_validation_command(raw, "validation command contains shell metacharacters")
    try:
        argv = shlex.split(raw)
    except ValueError as error:
        return blocked_validation_command(raw, f"validation command cannot be parsed: {error}")
    if not argv:
        return blocked_validation_command(raw, "validation command is empty")
    command_id = validation_command_id(argv)
    if command_id is None:
        return blocked_validation_command(raw, "validation command is not in the safe command registry")
    return {
        "command_id": command_id,
        "argv": argv,
        "cwd": ".",
        "raw": raw,
    }


def coerce_validation_command(value) -> dict | ValidationCommand:
    if isinstance(value, ValidationCommand):
        return value
    if isinstance(value, str):
        return validation_command_from_string(value)
    if isinstance(value, dict):
        return value
    return blocked_validation_command(str(value), "validation command must be a string or mapping")


def coerce_evidence_requirement(value) -> dict | EvidenceRequirement:
    if isinstance(value, EvidenceRequirement):
        return value
    if isinstance(value, str):
        return {"kind": value, "name": value}
    if isinstance(value, dict):
        return value
    return {"kind": "runtime_test", "name": str(value)}


class ActionContract(StrictModel):
    action_id: str
    track_id: str
    milestone_id: str
    wr_id: str
    execution_kind: ExecutionKind
    executor_kind: ExecutorKind
    authority_level: str
    permissions_required: list[Permission]
    allowed_outputs: list[str] = Field(default_factory=list)
    new_outputs: list[str] = Field(default_factory=list)
    forbidden_outputs: list[str] = Field(default_factory=list)
    forbidden_patterns: list[str] = Field(default_factory=list)
    writer_strategy: WriterStrategy = "no_writer"
    template_outputs: dict[str, str] = Field(default_factory=dict)
    patches: list[TextPatch] = Field(default_factory=list)
    validation_commands: list[ValidationCommand]
    evidence_required: list[EvidenceRequirement] = Field(default_factory=list)
    closeout_contract: CloseoutContract
    rollback_policy: RollbackPolicy
    stop_conditions: list[str]
    required_prior_milestones: list[str] = Field(default_factory=list)
    required_prior_completion_quality: CompletionQuality | None = None
    truth_claim_updates: list[dict[str, Any]] = Field(default_factory=list)
    parent_action_id: str | None = None
    agent_subaction: AgentSubActionContract | None = None
    production_source_path: str = "docs-site/src/content/docs/workspace/production-tracks.yaml"
    roadmap_source_path: str = "docs-site/src/content/docs/workspace/roadmap-items.yaml"
    manifest_source_path: str = ""

    @field_validator(
        "action_id",
        "track_id",
        "milestone_id",
        "wr_id",
        "authority_level",
    )
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        return clean_text(value)

    @field_validator(
        "allowed_outputs",
        "new_outputs",
        "forbidden_outputs",
        "forbidden_patterns",
        "stop_conditions",
        "required_prior_milestones",
    )
    @classmethod
    def validate_text_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("production_source_path", "roadmap_source_path", "manifest_source_path")
    @classmethod
    def validate_optional_source_path(cls, value: str) -> str:
        return value.strip()

    @field_validator("parent_action_id")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        return clean_text(value) if value is not None else None

    @field_validator("validation_commands", mode="before")
    @classmethod
    def validate_validation_commands(cls, value) -> list[dict | ValidationCommand]:
        if not isinstance(value, list):
            return [blocked_validation_command(str(value), "validation_commands must be a list")]
        return [coerce_validation_command(item) for item in value]

    @field_validator("evidence_required", mode="before")
    @classmethod
    def validate_evidence_requirements(cls, value) -> list[dict | EvidenceRequirement]:
        if not isinstance(value, list):
            return [{"kind": "runtime_test", "name": str(value)}]
        return [coerce_evidence_requirement(item) for item in value]

    @model_validator(mode="after")
    def validate_action_contract(self) -> ActionContract:
        if not self.permissions_required:
            raise ValueError(f"{self.action_id}: permissions_required must not be empty")
        if not self.validation_commands:
            raise ValueError(f"{self.action_id}: validation_commands must not be empty")
        if not self.stop_conditions:
            raise ValueError(f"{self.action_id}: stop_conditions must not be empty")
        if self.executor_kind == "extraction_gate" and "foundation_extraction" not in self.permissions_required:
            raise ValueError(f"{self.action_id}: extraction_gate executor requires foundation_extraction permission")
        return self


class ContractPack(StrictModel):
    version: int = 1
    track_id: str
    generated_at: str
    source_digests: dict[str, str]
    actions: list[ActionContract]

    @field_validator("track_id", "generated_at")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        return clean_text(value)

    @model_validator(mode="after")
    def validate_pack(self) -> ContractPack:
        if self.version != 1:
            raise ValueError("only Contract Pack version 1 is supported")
        if not self.source_digests:
            raise ValueError(f"{self.track_id}: source_digests must not be empty")
        action_ids = [action.action_id for action in self.actions]
        duplicates = sorted({action_id for action_id in action_ids if action_ids.count(action_id) > 1})
        if duplicates:
            raise ValueError(f"{self.track_id}: duplicate action_id values: {', '.join(duplicates)}")
        for action in self.actions:
            if action.track_id != self.track_id:
                raise ValueError(f"{action.action_id}: action track_id does not match pack track_id")
        return self


def now_utc_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")
