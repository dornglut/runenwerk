from __future__ import annotations

from typing import Any, Literal
from pathlib import Path

import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator

from roadmap_state import WorkflowError, repo_path


class AgentSubActionMetadata(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)

    sub_action_id: str
    title: str
    prompt: str
    allowed_outputs: list[str] = Field(default_factory=list)
    new_outputs: list[str] = Field(default_factory=list)
    validation_commands: list[str | dict[str, Any]] = Field(default_factory=list)
    evidence_required: list[str | dict[str, Any]] = Field(default_factory=list)
    stop_conditions: list[str] = Field(default_factory=list)

    @field_validator("sub_action_id", "title", "prompt")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("agent_subactions text fields must not be empty")
        return cleaned

    @field_validator("allowed_outputs", "new_outputs", "stop_conditions")
    @classmethod
    def validate_text_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]


class PlanContractMetadata(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)

    version: int = 1
    status: Literal["draft", "accepted"] = "accepted"
    wr_id: str
    milestone_id: str
    execution_kind: str | None = None
    executor_kind: str
    authority_level: str
    permissions_required: list[str] = Field(default_factory=list)
    writer_strategy: str = "no_writer"
    allowed_outputs: list[str] = Field(default_factory=list)
    new_outputs: list[str] = Field(default_factory=list)
    forbidden_outputs: list[str] = Field(default_factory=list)
    forbidden_patterns: list[str] = Field(default_factory=list)
    template_outputs: dict[str, str] = Field(default_factory=dict)
    patches: list[dict[str, str]] = Field(default_factory=list)
    validation_commands: list[str | dict[str, Any]] = Field(default_factory=list)
    evidence_required: list[str | dict[str, Any]] = Field(default_factory=list)
    closeout_contract: dict = Field(default_factory=dict)
    truth_claim_updates: list[dict[str, Any]] = Field(default_factory=list)
    agent_subactions: list[AgentSubActionMetadata] = Field(default_factory=list)
    rollback_policy: str
    stop_conditions: list[str] = Field(default_factory=list)

    @field_validator("wr_id", "milestone_id", "executor_kind", "authority_level", "writer_strategy", "rollback_policy")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("plan contract text fields must not be empty")
        return cleaned

    @field_validator("execution_kind")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("execution_kind must not be empty when provided")
        return cleaned

    @field_validator(
        "permissions_required",
        "allowed_outputs",
        "new_outputs",
        "forbidden_outputs",
        "forbidden_patterns",
        "stop_conditions",
    )
    @classmethod
    def validate_text_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("template_outputs")
    @classmethod
    def validate_template_outputs(cls, value: dict[str, str]) -> dict[str, str]:
        return {key.strip(): content for key, content in value.items() if key.strip()}


def structured_plan_contract_path(plan_path: Path) -> Path:
    return plan_path.parent / "plan.contract.yaml"


def load_plan_contract_metadata(
    *,
    plan_path: Path,
    milestone_id: str,
    executor_kind: str,
    wr_id: str | None,
) -> PlanContractMetadata:
    sidecar_path = structured_plan_contract_path(plan_path)
    if not sidecar_path.exists():
        raise WorkflowError(
            f"{milestone_id}: structured implementation-plan authority is missing at {repo_path(sidecar_path)}"
        )
    data = yaml.safe_load(sidecar_path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(sidecar_path)} must contain a YAML mapping")
    try:
        contract = PlanContractMetadata.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(sidecar_path)} is not a valid plan contract sidecar: {error}") from error
    if contract.milestone_id != milestone_id:
        raise WorkflowError(f"{repo_path(sidecar_path)} milestone_id must be {milestone_id}")
    if contract.executor_kind != executor_kind:
        raise WorkflowError(f"{repo_path(sidecar_path)} executor_kind must be {executor_kind}")
    if wr_id and contract.wr_id != wr_id:
        raise WorkflowError(f"{repo_path(sidecar_path)} wr_id must be {wr_id}")
    return contract
