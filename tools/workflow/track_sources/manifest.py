from __future__ import annotations

import re
from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path
from typing import Literal

import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

from roadmap_state import (
    REPO_ROOT,
    WorkflowError,
    document_frontmatter_status,
    is_new_write_scope,
    normalize_repo_path,
    normalize_write_scope_path,
    normalized_write_scopes_with_generated_outputs,
    path_within_scope,
    repo_path,
)


TRACK_EXECUTION_MANIFEST_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/track-execution-manifests"
GENERATED_SCOPE_PREFIXES = ("generated:", "derived:")
ROADMAP_ID_PATTERN = re.compile(r"^WR-\d{3}$")
FUTURE_ROADMAP_ID_PATTERN = re.compile(r"^WR-TBD-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
MANIFEST_AUDIT_CATEGORY_ORDER = (
    "alignment errors",
    "missing gates",
    "invalid blocked fields",
    "invalid closeout path",
    "WR scope mismatch",
    "truth claim errors",
    "missing WR authority",
    "other manifest audit blockers",
)
FULL_AUTOMATION_EXECUTION_KINDS = {
    "design_contract",
    "implementation_proof",
    "proof_aggregation",
    "handoff_closeout",
    "extraction_gate",
}
FULL_TRACK_PERMISSION_SET = {
    "auto_safe",
    "agent_design",
    "agent_closeout",
    "product_code",
    "product_implementation",
}
FULL_AUTOMATION_PERMISSION_GRANTS = {
    "auto_safe": {"auto_safe"},
    "agent_design": {"agent_design"},
    "agent_closeout": {"agent_closeout"},
    "product_code": {"product_code"},
    "product_implementation": {"product_implementation"},
    "runtime_closeout": {"agent_closeout"},
    "handoff": {"agent_closeout"},
    "crate_creation": {"crate_creation"},
    "foundation_extraction": {"foundation_extraction"},
}
PROOF_AGGREGATION_REQUIRED_EVIDENCE_CATEGORIES = {
    "headless fixture",
    "diagnostics",
    "source-map proof",
    "runtime artifact evidence",
    "reproducibility evidence",
}
GENERIC_EVIDENCE_CATEGORY_ALIASES = {
    "runtime test": "runtime_test",
    "runtime_test": "runtime_test",
    "headless fixture": "fixture",
    "fixture": "fixture",
    "diagnostic": "diagnostics",
    "diagnostics": "diagnostics",
    "source-map proof": "source_maps",
    "source maps": "source_maps",
    "source_maps": "source_maps",
    "runtime artifact evidence": "artifact",
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
    "artifact": "artifact",
    "migration": "migration",
    "reproducibility evidence": "reproducibility",
    "reproducibility": "reproducibility",
    "visual": "visual",
    "handoff": "handoff",
}

ManifestMilestoneType = Literal["docs_only", "design_only", "implementation", "hardening", "closeout"]
ManifestExecutionKind = Literal[
    "design_contract",
    "implementation_proof",
    "proof_aggregation",
    "handoff_closeout",
    "extraction_gate",
]
ImplementationWriterStrategy = Literal[
    "no_writer",
    "template_writer",
    "patch_writer",
    "agent_writer",
    "proof_aggregation_writer",
    "verification_writer",
]
CloseoutStrategy = Literal[
    "bounded_contract_closeout",
    "runtime_proven_closeout",
    "handoff_closeout",
    "extraction_gate_closeout",
]
TruthClaimKind = Literal[
    "product_behavior",
    "architecture_contract",
    "proof_slice",
    "handoff",
    "extraction_gate",
]
TruthClaimLevel = Literal[
    "not_applicable",
    "bounded_contract",
    "runtime_proven",
    "proof_slice_runtime_proven",
    "architecture_runtime_proven",
    "perfectionist_verified",
]
TruthClaimStatus = Literal["satisfied", "blocked", "superseded"]
TruthEvidenceKind = Literal[
    "doc_exists",
    "doc_frontmatter_status",
    "rust_symbol_exists",
    "module_path_exists",
    "validation_command",
    "closeout_evidence_category",
]
AgentDesignAuthoringStrategy = Literal["template_contract_writer", "codex_contract_writer"]


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


class ManifestDesignDependency(StrictModel):
    path: str
    required_status: str = "active"
    reason: str

    @field_validator("path")
    @classmethod
    def validate_path(cls, value: str) -> str:
        normalized = normalize_repo_path(value)
        if not normalized:
            raise ValueError("design dependency path must not be empty")
        return normalized

    @field_validator("required_status", "reason")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("design dependency text fields must not be empty")
        return cleaned


class ManifestTruthEvidence(StrictModel):
    evidence_kind: TruthEvidenceKind
    path: str | None = None
    required_status: str | None = None
    symbol: str | None = None
    command: str | None = None
    category: str | None = None
    closeout_path: str | None = None
    reason: str

    @field_validator("path", "required_status", "symbol", "command", "category", "closeout_path")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str):
            value = str(value)
        cleaned = value.strip()
        return cleaned or None

    @field_validator("reason")
    @classmethod
    def validate_reason(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("truth evidence reason must not be empty")
        return cleaned

    @model_validator(mode="after")
    def validate_required_fields_for_kind(self) -> ManifestTruthEvidence:
        required_by_kind = {
            "doc_exists": ("path",),
            "doc_frontmatter_status": ("path", "required_status"),
            "rust_symbol_exists": ("path", "symbol"),
            "module_path_exists": ("path",),
            "validation_command": ("command",),
            "closeout_evidence_category": ("closeout_path", "category"),
        }
        missing = [field for field in required_by_kind[self.evidence_kind] if not getattr(self, field)]
        if missing:
            raise ValueError(f"truth evidence {self.evidence_kind} requires " + ", ".join(missing))
        return self


class ManifestTruthClaim(StrictModel):
    claim_id: str
    claim_kind: TruthClaimKind
    claim_level: TruthClaimLevel
    claim_status: TruthClaimStatus
    claim_statement: str
    required_docs: list[ManifestTruthEvidence] = Field(default_factory=list)
    required_code_contracts: list[ManifestTruthEvidence] = Field(default_factory=list)
    required_validations: list[ManifestTruthEvidence] = Field(default_factory=list)
    required_closeout_evidence: list[ManifestTruthEvidence] = Field(default_factory=list)
    known_gaps: list[str] = Field(default_factory=list)
    supersedes: list[str] = Field(default_factory=list)
    blocks_downstream: list[str] = Field(default_factory=list)

    @field_validator("claim_id", "claim_statement")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("truth claim text fields must not be empty")
        return cleaned

    @field_validator("known_gaps", "supersedes", "blocks_downstream")
    @classmethod
    def validate_text_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @model_validator(mode="after")
    def validate_claim_contract(self) -> ManifestTruthClaim:
        if self.claim_status == "blocked" and not self.known_gaps:
            raise ValueError("blocked truth claims must list known_gaps")
        if self.claim_status == "satisfied" and not (
            self.required_docs
            or self.required_code_contracts
            or self.required_validations
            or self.required_closeout_evidence
        ):
            raise ValueError("satisfied truth claims must declare evidence requirements")
        return self


class ManifestAgentDesignContract(StrictModel):
    source_documents: list[str]
    required_sections: list[str]
    required_decisions: list[str]
    acceptance_checklist: list[str]
    planning_write_scope: list[str] = Field(default_factory=list)
    allowed_write_scopes: list[str] = Field(default_factory=list)
    forbidden_scopes: list[str] = Field(default_factory=list)
    expected_output_paths: list[str] = Field(default_factory=list)
    validation_commands: list[str] = Field(default_factory=list)
    stop_conditions: list[str] = Field(default_factory=list)
    template_key: str | None = None
    generated_contract_marker: str | None = None
    generated_from_template_version: str | None = None
    authoring_strategy: AgentDesignAuthoringStrategy = "template_contract_writer"
    agent_prompt: str | None = None
    agent_context_files: list[str] = Field(default_factory=list)
    agent_required_outputs: list[str] = Field(default_factory=list)
    agent_diff_protocol_version: str | None = None
    agent_worktree_policy: str = "isolated_action_workspace"

    @field_validator("source_documents")
    @classmethod
    def validate_source_documents(cls, value: list[str]) -> list[str]:
        cleaned = [normalize_repo_path(item) for item in value if item.strip()]
        if not cleaned:
            raise ValueError("agent_design source_documents must not be empty")
        return cleaned

    @field_validator("required_sections", "required_decisions", "acceptance_checklist")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("agent_design list fields must not be empty")
        return cleaned

    @field_validator(
        "planning_write_scope",
        "allowed_write_scopes",
        "forbidden_scopes",
        "expected_output_paths",
        "validation_commands",
        "stop_conditions",
        "agent_context_files",
        "agent_required_outputs",
    )
    @classmethod
    def validate_optional_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("agent_prompt", "agent_diff_protocol_version")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        return cleaned or None

    @field_validator("agent_worktree_policy")
    @classmethod
    def validate_agent_worktree_policy(cls, value: str) -> str:
        cleaned = value.strip()
        if cleaned != "isolated_action_workspace":
            raise ValueError("agent_design agent_worktree_policy must be isolated_action_workspace")
        return cleaned


class ManifestAutoSafeContract(StrictModel):
    wr_candidate_policy: str
    wr_id_allocation_behavior: str
    milestone_to_wr_link_behavior: str
    manifest_wr_reference_behavior: str
    allowed_metadata_write_scopes: list[str]
    forbidden_scopes: list[str]
    validation_commands: list[str]
    stop_conditions: list[str]
    template_key: str
    generated_contract_marker: str
    generated_from_template_version: str

    @field_validator(
        "wr_candidate_policy",
        "wr_id_allocation_behavior",
        "milestone_to_wr_link_behavior",
        "manifest_wr_reference_behavior",
        "template_key",
        "generated_contract_marker",
        "generated_from_template_version",
    )
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("auto_safe_contract text fields must not be empty")
        return cleaned

    @field_validator("allowed_metadata_write_scopes", "forbidden_scopes", "validation_commands", "stop_conditions")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("auto_safe_contract list fields must not be empty")
        return cleaned


class ManifestProductCodeContract(StrictModel):
    required_active_wr_state: str
    required_accepted_implementation_plan: str
    exact_allowed_implementation_write_scopes: list[str]
    required_function_method_scope: list[str]
    forbidden_implementation_scopes: list[str]
    tests_to_add_change: list[str]
    runtime_evidence_required: list[str]
    validation_commands: list[str]
    rollback_compatibility_expectations: list[str]
    closeout_evidence: list[str]
    stop_conditions: list[str]
    template_key: str
    generated_contract_marker: str
    generated_from_template_version: str

    @field_validator(
        "required_active_wr_state",
        "required_accepted_implementation_plan",
        "template_key",
        "generated_contract_marker",
        "generated_from_template_version",
    )
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("product_code_contract text fields must not be empty")
        return cleaned

    @field_validator(
        "exact_allowed_implementation_write_scopes",
        "required_function_method_scope",
        "forbidden_implementation_scopes",
        "tests_to_add_change",
        "runtime_evidence_required",
        "validation_commands",
        "rollback_compatibility_expectations",
        "closeout_evidence",
        "stop_conditions",
    )
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("product_code_contract list fields must not be empty")
        return cleaned


class ManifestImplementationTemplate(StrictModel):
    file: str
    content: str

    @field_validator("file")
    @classmethod
    def validate_file(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("implementation_writer template file must not be empty")
        return cleaned


class ManifestImplementationPatch(StrictModel):
    file: str
    find: str
    replace: str

    @field_validator("file", "find")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("implementation_writer patch file/find fields must not be empty")
        return cleaned


class ManifestImplementationWriter(StrictModel):
    strategy: ImplementationWriterStrategy = "no_writer"
    aggregation_only: bool = False
    required_prior_milestones: list[str] = Field(default_factory=list)
    required_prior_completion_quality: str | None = None
    required_evidence_categories: list[str] = Field(default_factory=list)
    allowed_files: list[str] = Field(default_factory=list)
    allowed_write_scopes: list[str] = Field(default_factory=list)
    required_outputs: list[str] = Field(default_factory=list)
    forbidden_files: list[str] = Field(default_factory=list)
    forbidden_scopes: list[str] = Field(default_factory=list)
    forbidden_patterns: list[str] = Field(default_factory=list)
    new_file_policy: str = "existing_files_only"
    validation_commands: list[str] = Field(default_factory=list)
    closeout_path: str | None = None
    stop_conditions: list[str] = Field(default_factory=list)
    templates: list[ManifestImplementationTemplate] = Field(default_factory=list)
    patches: list[ManifestImplementationPatch] = Field(default_factory=list)
    agent_prompt: str | None = None
    agent_context_files: list[str] = Field(default_factory=list)
    agent_required_outputs: list[str] = Field(default_factory=list)
    agent_diff_protocol_version: str | None = None
    agent_worktree_policy: str = "isolated_action_workspace"

    @field_validator("new_file_policy")
    @classmethod
    def validate_new_file_policy(cls, value: str) -> str:
        cleaned = value.strip()
        allowed = {"existing_files_only", "explicit_new_scope_required"}
        if cleaned not in allowed:
            raise ValueError(f"implementation_writer new_file_policy must be one of {sorted(allowed)}")
        return cleaned

    @field_validator("agent_worktree_policy")
    @classmethod
    def validate_agent_worktree_policy(cls, value: str) -> str:
        cleaned = value.strip()
        if cleaned != "isolated_action_workspace":
            raise ValueError("implementation_writer agent_worktree_policy must be isolated_action_workspace")
        return cleaned

    @field_validator(
        "required_prior_milestones",
        "required_evidence_categories",
        "allowed_files",
        "allowed_write_scopes",
        "required_outputs",
        "forbidden_files",
        "forbidden_scopes",
        "forbidden_patterns",
        "validation_commands",
        "stop_conditions",
        "agent_context_files",
        "agent_required_outputs",
    )
    @classmethod
    def validate_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("required_prior_completion_quality", "closeout_path", "agent_prompt", "agent_diff_protocol_version")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        return cleaned or None


class ManifestCloseoutEvidenceRecord(StrictModel):
    milestone_id: str
    wr_id: str
    completion_quality: str
    evidence_categories: list[str]
    validation_commands: list[str]
    validation_results: list[str]
    files_changed: list[str]
    runtime_artifacts: list[str] = Field(default_factory=list)
    diagnostics: list[str] = Field(default_factory=list)
    source_maps: list[str] = Field(default_factory=list)
    known_gaps: list[str]
    closeout_path: str
    produced_at: str | None = None

    @field_validator("milestone_id", "wr_id", "completion_quality", "closeout_path", "produced_at", mode="before")
    @classmethod
    def validate_required_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str):
            value = str(value)
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("closeout evidence record text fields must not be empty")
        return cleaned

    @field_validator(
        "evidence_categories",
        "validation_commands",
        "validation_results",
        "files_changed",
        "runtime_artifacts",
        "diagnostics",
        "source_maps",
        "known_gaps",
    )
    @classmethod
    def validate_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @model_validator(mode="after")
    def validate_required_lists(self) -> ManifestCloseoutEvidenceRecord:
        for field_name in (
            "evidence_categories",
            "validation_commands",
            "validation_results",
            "files_changed",
        ):
            if not getattr(self, field_name):
                raise ValueError(f"closeout evidence record requires {field_name}")
        return self


class ManifestAgentCloseoutContract(StrictModel):
    evidence_files: list[str]
    validation_commands: list[str]
    completion_quality_allowed: list[str]
    closeout_path: str
    production_roadmap_state_updates: list[str]
    known_gap_reporting: list[str]
    next_action_update_rules: list[str]
    template_key: str
    generated_contract_marker: str
    generated_from_template_version: str

    @field_validator("closeout_path", "template_key", "generated_contract_marker", "generated_from_template_version")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("agent_closeout_contract text fields must not be empty")
        return cleaned

    @field_validator(
        "evidence_files",
        "validation_commands",
        "completion_quality_allowed",
        "production_roadmap_state_updates",
        "known_gap_reporting",
        "next_action_update_rules",
    )
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("agent_closeout_contract list fields must not be empty")
        return cleaned


class ManifestRuntimeCloseoutContract(StrictModel):
    runtime_test_evidence_required: list[str]
    validation_commands: list[str]
    completion_quality_allowed: list[str]
    closeout_path: str
    files_changed_report: list[str]
    known_gap_reporting: list[str]
    production_roadmap_state_updates: list[str]
    next_action_update_rules: list[str]
    evidence_manifest_path: str | None = None
    template_key: str
    generated_contract_marker: str
    generated_from_template_version: str

    @field_validator("closeout_path", "template_key", "generated_contract_marker", "generated_from_template_version")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("runtime_closeout_contract text fields must not be empty")
        return cleaned

    @field_validator(
        "runtime_test_evidence_required",
        "validation_commands",
        "completion_quality_allowed",
        "files_changed_report",
        "known_gap_reporting",
        "production_roadmap_state_updates",
        "next_action_update_rules",
    )
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("runtime_closeout_contract list fields must not be empty")
        return cleaned

    @field_validator("evidence_manifest_path")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        return cleaned or None


class ManifestHandoffContract(StrictModel):
    handoff_target: str
    proof_path_rules: list[str]
    forbidden_scopes: list[str]
    validation_commands: list[str]
    stop_conditions: list[str]
    template_key: str
    generated_contract_marker: str
    generated_from_template_version: str

    @field_validator("handoff_target", "template_key", "generated_contract_marker", "generated_from_template_version")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("handoff_contract text fields must not be empty")
        return cleaned

    @field_validator("proof_path_rules", "forbidden_scopes", "validation_commands", "stop_conditions")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("handoff_contract list fields must not be empty")
        return cleaned


class ManifestContractParameters(StrictModel):
    proof_slice_id: str | None = None
    proof_slice_title: str | None = None
    target_control_surface: str | None = None
    exact_allowed_implementation_write_scopes: list[str] = Field(default_factory=list)
    required_function_method_scope: list[str] = Field(default_factory=list)
    tests_to_add_change: list[str] = Field(default_factory=list)
    runtime_evidence_required: list[str] = Field(default_factory=list)
    validation_commands: list[str] = Field(default_factory=list)
    source_documents: list[str] = Field(default_factory=list)
    required_sections: list[str] = Field(default_factory=list)
    required_decisions: list[str] = Field(default_factory=list)
    acceptance_checklist: list[str] = Field(default_factory=list)
    closeout_evidence: list[str] = Field(default_factory=list)
    rollback_compatibility_expectations: list[str] = Field(default_factory=list)

    @field_validator("proof_slice_id", "proof_slice_title", "target_control_surface")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        return cleaned or None

    @field_validator(
        "exact_allowed_implementation_write_scopes",
        "required_function_method_scope",
        "tests_to_add_change",
        "runtime_evidence_required",
        "validation_commands",
        "source_documents",
        "required_sections",
        "required_decisions",
        "acceptance_checklist",
        "closeout_evidence",
        "rollback_compatibility_expectations",
    )
    @classmethod
    def validate_optional_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]


class TrackExecutionManifestMilestone(StrictModel):
    milestone_id: str
    title: str
    stage: str
    authority_level: str
    milestone_type: ManifestMilestoneType
    owning_wr: str | None = None
    future_wr_candidate: str | None = None
    predecessor_dependencies: list[str] = Field(default_factory=list)
    write_scope: list[str]
    forbidden_scope: list[str]
    required_contracts: list[str]
    validation_commands: list[str]
    evidence_gates: list[str]
    expected_closeout_path: str
    stop_conditions: list[str]
    next_legal_action: str
    may_create_code: bool
    may_create_crates: bool
    may_modify_production_behavior: bool
    execution_kind: ManifestExecutionKind | None = None
    milestone_kind: str | None = None
    permission_classes_required: list[str] = Field(default_factory=list)
    implementation_proof_kind: str | None = None
    closeout_kind: str | None = None
    required_evidence_categories: list[str] = Field(default_factory=list)
    template_key: str | None = None
    generated_contract_marker: str | None = None
    generated_from_template_version: str | None = None
    contract_parameters: ManifestContractParameters | None = None
    auto_safe_contract: ManifestAutoSafeContract | None = None
    agent_design: ManifestAgentDesignContract | None = None
    agent_design_contract: ManifestAgentDesignContract | None = None
    product_code_contract: ManifestProductCodeContract | None = None
    implementation_writer: ManifestImplementationWriter | None = None
    closeout_strategy: CloseoutStrategy | None = None
    agent_closeout_contract: ManifestAgentCloseoutContract | None = None
    runtime_closeout_contract: ManifestRuntimeCloseoutContract | None = None
    handoff_contract: ManifestHandoffContract | None = None

    @field_validator("milestone_id", "title", "stage", "authority_level", "expected_closeout_path", "next_legal_action")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("manifest milestone text fields must not be empty")
        return cleaned

    @field_validator("write_scope", "forbidden_scope", "required_contracts", "validation_commands", "evidence_gates", "stop_conditions")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("manifest milestone list fields must not be empty")
        return cleaned

    @field_validator("owning_wr")
    @classmethod
    def validate_owning_wr(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        if not ROADMAP_ID_PATTERN.fullmatch(cleaned):
            raise ValueError("owning_wr must match WR-000")
        return cleaned

    @field_validator("future_wr_candidate")
    @classmethod
    def validate_future_wr_candidate(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        if not FUTURE_ROADMAP_ID_PATTERN.fullmatch(cleaned):
            raise ValueError("future_wr_candidate must match WR-TBD-NAME")
        return cleaned

    @model_validator(mode="after")
    def validate_wr_authority(self) -> TrackExecutionManifestMilestone:
        if bool(self.owning_wr) == bool(self.future_wr_candidate):
            raise ValueError("manifest milestones must have exactly one owning_wr or future_wr_candidate")
        return self

    @field_validator("permission_classes_required", "required_evidence_categories")
    @classmethod
    def validate_optional_text_list(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]


class TrackExecutionManifest(StrictModel):
    version: int
    track_id: str
    authority_level: str
    accepted_design_dependencies: list[ManifestDesignDependency]
    global_forbidden_scope: list[str]
    global_validation_commands: list[str]
    global_stop_conditions: list[str]
    next_legal_action: str
    ai_executable: bool = False
    full_automation_target: bool = False
    truth_claims: list[ManifestTruthClaim] = Field(default_factory=list)
    milestones: list[TrackExecutionManifestMilestone]

    @field_validator("track_id", "authority_level", "next_legal_action")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("manifest text fields must not be empty")
        return cleaned

    @field_validator("global_forbidden_scope", "global_validation_commands", "global_stop_conditions")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("manifest list fields must not be empty")
        return cleaned

    @model_validator(mode="after")
    def validate_unique_milestones(self) -> TrackExecutionManifest:
        milestone_ids = [milestone.milestone_id for milestone in self.milestones]
        duplicates = sorted({milestone_id for milestone_id in milestone_ids if milestone_ids.count(milestone_id) > 1})
        if duplicates:
            raise ValueError(f"duplicate manifest milestone ids: {', '.join(duplicates)}")
        return self

    @property
    def by_milestone_id(self) -> dict[str, TrackExecutionManifestMilestone]:
        return {milestone.milestone_id: milestone for milestone in self.milestones}


@dataclass(frozen=True)
class LoadedTrackExecutionManifest:
    manifest: TrackExecutionManifest
    path: Path


def manifest_source_path(track_id: str, root: Path = TRACK_EXECUTION_MANIFEST_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


def load_track_execution_manifest(
    track_id: str,
    *,
    root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> LoadedTrackExecutionManifest | None:
    path = manifest_source_path(track_id, root=root)
    if not path.exists():
        return None
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as error:
        raise WorkflowError(f"{repo_path(path)} contains malformed YAML: {error}") from error
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    try:
        manifest = TrackExecutionManifest.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid Track Execution Manifest: {error}") from error
    if manifest.track_id != track_id:
        raise WorkflowError(f"{repo_path(path)} declares track_id={manifest.track_id}, expected {track_id}")
    return LoadedTrackExecutionManifest(manifest=manifest, path=path)


def sha256_file(path: Path) -> str:
    if not path.exists():
        raise WorkflowError(f"cannot digest missing source file: {repo_path(path)}")
    return sha256(path.read_bytes()).hexdigest()


def manifest_design_dependency_errors(
    manifest: TrackExecutionManifest,
    *,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    errors: list[str] = []
    for dependency in manifest.accepted_design_dependencies:
        candidate = repo_root / dependency.path
        if not candidate.exists():
            errors.append(f"{manifest.track_id}: manifest design dependency missing {dependency.path} ({dependency.reason})")
            continue
        status = document_frontmatter_status(candidate)
        if status is None:
            errors.append(
                f"{manifest.track_id}: manifest design dependency {dependency.path} has no frontmatter status "
                f"({dependency.reason})"
            )
        elif status.lower() != dependency.required_status.lower():
            errors.append(
                f"{manifest.track_id}: manifest design dependency {dependency.path} status {status!r} "
                f"does not match required {dependency.required_status!r} ({dependency.reason})"
            )
    return errors


def source_digest_paths(
    loaded: LoadedTrackExecutionManifest,
    *,
    production_source: Path,
    roadmap_source: Path,
) -> list[Path]:
    paths = [loaded.path, production_source, roadmap_source]
    for dependency in loaded.manifest.accepted_design_dependencies:
        paths.append(REPO_ROOT / dependency.path)
    unique: list[Path] = []
    seen: set[str] = set()
    for path in paths:
        key = repo_path(path)
        if key in seen:
            continue
        seen.add(key)
        unique.append(path)
    return unique


def source_digest_map(
    loaded: LoadedTrackExecutionManifest,
    *,
    production_source: Path,
    roadmap_source: Path,
) -> dict[str, str]:
    return {
        repo_path(path): sha256_file(path)
        for path in source_digest_paths(
            loaded,
            production_source=production_source,
            roadmap_source=roadmap_source,
        )
    }


def is_generated_or_derived_scope(scope: str) -> bool:
    return scope.strip().lower().startswith(GENERATED_SCOPE_PREFIXES)


def mentions_generated_or_derived_scope(scope: str) -> bool:
    cleaned = scope.strip().lower()
    return "generated" in cleaned or "derived" in cleaned


def manifest_write_scope_path(scope: str) -> str | None:
    normalized = normalize_write_scope_path(scope)
    if not normalized or normalized.startswith("blocked:") or " " in normalized:
        return None
    if "/" not in normalized and not (REPO_ROOT / normalized).is_file():
        return None
    return normalized


def agent_design_contract_for_entry(entry: TrackExecutionManifestMilestone) -> ManifestAgentDesignContract | None:
    return entry.agent_design or entry.agent_design_contract


def product_implementation_scopes_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.exact_allowed_implementation_write_scopes)
    if entry.contract_parameters and entry.contract_parameters.exact_allowed_implementation_write_scopes:
        return list(entry.contract_parameters.exact_allowed_implementation_write_scopes)
    return list(entry.write_scope)


def product_validation_commands_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.validation_commands)
    if entry.contract_parameters and entry.contract_parameters.validation_commands:
        return list(entry.contract_parameters.validation_commands)
    return list(entry.validation_commands)


def product_forbidden_scopes_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return [*entry.forbidden_scope, *entry.product_code_contract.forbidden_implementation_scopes]
    return list(entry.forbidden_scope)


def implementation_writer_allowed_scopes(writer: ManifestImplementationWriter) -> list[str]:
    return [*writer.allowed_files, *writer.allowed_write_scopes]


def implementation_writer_forbidden_scopes(writer: ManifestImplementationWriter) -> list[str]:
    return [*writer.forbidden_files, *writer.forbidden_scopes]


def implementation_writer_output_scopes(writer: ManifestImplementationWriter) -> list[str]:
    return [*implementation_writer_allowed_scopes(writer), *writer.required_outputs]


def normalize_evidence_category(category: str) -> str:
    cleaned = category.strip().lower().replace("-", "_").replace(" ", "_")
    alias_key = category.strip().lower()
    return GENERIC_EVIDENCE_CATEGORY_ALIASES.get(alias_key, GENERIC_EVIDENCE_CATEGORY_ALIASES.get(cleaned, cleaned))


def scope_is_covered_by_wr(scope: str, wr_scopes: list[str]) -> bool:
    normalized = manifest_write_scope_path(scope)
    if normalized is None:
        return True
    normalized_wr_scopes = normalized_write_scopes_with_generated_outputs(wr_scopes)
    return any(path_within_scope(normalized, wr_scope) for wr_scope in normalized_wr_scopes)


def truth_claim_summary_lines(manifest: TrackExecutionManifest) -> list[str]:
    if not manifest.truth_claims:
        return ["Truth claims: missing"]
    lines = ["Truth claims:"]
    for claim in manifest.truth_claims:
        downstream = f"; blocks {', '.join(claim.blocks_downstream)}" if claim.blocks_downstream else ""
        lines.append(
            f"- {claim.claim_id}: {claim.claim_status} {claim.claim_kind} "
            f"at {claim.claim_level}{downstream}"
        )
    return lines


def new_scope_is_marked(path: str, scopes: list[str]) -> bool:
    normalized = normalize_repo_path(path)
    return any(is_new_write_scope(scope) and normalize_write_scope_path(scope) == normalized for scope in scopes)
