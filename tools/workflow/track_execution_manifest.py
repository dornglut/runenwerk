#!/usr/bin/env python3
"""
Machine-readable Track Execution Manifest contracts.

File: tools/workflow/track_execution_manifest.py
Module: track_execution_manifest
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from datetime import date, datetime, timezone
from hashlib import sha256
from pathlib import Path
from typing import Literal

import typer
import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator
from rich.console import Console

from production_plan import (
    ProductionPlanContext,
    classify_plan_action,
    default_contract_path,
    render_readiness_report,
    resolve_plan_context,
)
from production_state import PRODUCTION_SOURCE, ProductionMilestone, ProductionPlanningState, ProductionTrack, load_production_tracks
from roadmap_state import (
    REPO_ROOT,
    ROADMAP_SOURCE,
    RoadmapItem,
    RoadmapState,
    WorkflowError,
    combine_roadmap_data,
    document_frontmatter_status,
    load_yaml,
    load_roadmap,
    is_new_write_scope,
    normalize_repo_path,
    normalize_write_scope_path,
    normalized_write_scopes_with_generated_outputs,
    path_within_scope,
    repo_path,
    split_source_paths,
)


TRACK_EXECUTION_MANIFEST_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/track-execution-manifests"
TRACK_EXECUTION_LOCK_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/track-execution-locks"
EXECUTION_CONTRACT_PACK_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/execution-contract-packs"
TRACK_EXECUTION_RUN_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/track-execution-runs"
TRACK_EXECUTION_LOCKED_WORKFLOW_SOURCES = (
    REPO_ROOT / "tools/workflow/track_execution_manifest.py",
    REPO_ROOT / "tools/workflow/production_goal.py",
    REPO_ROOT / "tools/workflow/production_state.py",
    REPO_ROOT / "tools/workflow/roadmap_state.py",
)
ROADMAP_ID_PATTERN = re.compile(r"^WR-\d{3}$")
FUTURE_ROADMAP_ID_PATTERN = re.compile(r"^WR-TBD-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
GENERATED_SCOPE_PREFIXES = ("generated:", "derived:")
CONTRACT_TEMPLATE_VERSION = "v1"
CONTRACT_GENERATED_MARKER = "generated_by_production_complete_track_contracts"
CONTRACT_TEMPLATE_KEYS = {
    "docs_design": "docs/design milestone template",
    "implementation_runtime_proof": "implementation/runtime-proof milestone template",
    "final_handoff_closeout": "final handoff/closeout milestone template",
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
ManifestRunMode = Literal["single-step", "bounded-segment", "full-track", "agent-track"]
MANIFEST_RUNNER_PERMISSIONS = {
    "auto_safe",
    "agent_design",
    "agent_closeout",
    "product_code",
    "product_implementation",
    "crate_creation",
    "foundation_extraction",
}

console = Console()
app = typer.Typer(no_args_is_help=True, help="Plan, inspect, and audit Track Execution Manifest sources.")

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

PROOF_AGGREGATION_REQUIRED_EVIDENCE_CATEGORIES = {
    "headless fixture",
    "diagnostics",
    "source-map proof",
    "runtime artifact evidence",
    "reproducibility evidence",
}

GENERIC_EVIDENCE_CATEGORIES = {
    "runtime_test",
    "fixture",
    "diagnostics",
    "source_maps",
    "artifact",
    "migration",
    "reproducibility",
    "visual",
    "handoff",
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
    "artifact": "artifact",
    "migration": "migration",
    "reproducibility evidence": "reproducibility",
    "reproducibility": "reproducibility",
    "visual": "visual",
    "handoff": "handoff",
}

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

AGENT_TRACK_PERMISSION_SET = {
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

STRATEGIC_HUMAN_GATES = {
    "adr_acceptance",
    "locked_design_direction_change",
    "foundation_meta_extraction",
    "crate_creation_unless_pre_authorized",
    "external_plugin_security_boundary",
    "second_domain_extraction",
    "release_perfectionist_verified_certification",
}

STRATEGIC_GATE_PERMISSION_MAP = {
    "foundation_extraction": "foundation_meta_extraction",
    "crate_creation": "crate_creation_unless_pre_authorized",
}

WRITER_STRATEGIES = {
    "no_writer",
    "template_writer",
    "patch_writer",
    "agent_writer",
    "proof_aggregation_writer",
}

CLOSEOUT_STRATEGIES = {
    "bounded_contract_closeout",
    "runtime_proven_closeout",
    "handoff_closeout",
    "extraction_gate_closeout",
}


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", frozen=True)


class IndentedSafeDumper(yaml.SafeDumper):
    def increase_indent(self, flow: bool = False, indentless: bool = False):
        return super().increase_indent(flow, False)


class TrackExecutionLock(StrictModel):
    version: int = 1
    track_id: str
    ai_executable: bool
    locked_by: str
    locked_at: str
    manifest_digest: str
    source_digests: dict[str, str]
    granted_permissions: list[str]
    denied_permissions: list[str]
    strategic_human_gates: list[str]
    invalidation_rules: list[str]

    @field_validator("track_id", "locked_by", "locked_at", "manifest_digest")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("track execution lock text fields must not be empty")
        return cleaned

    @field_validator("granted_permissions", "strategic_human_gates", "invalidation_rules")
    @classmethod
    def validate_non_empty_list(cls, value: list[str]) -> list[str]:
        cleaned = [item.strip() for item in value if item.strip()]
        if not cleaned:
            raise ValueError("track execution lock list fields must not be empty")
        return cleaned

    @field_validator("denied_permissions")
    @classmethod
    def validate_denied_permissions(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("source_digests")
    @classmethod
    def validate_source_digests(cls, value: dict[str, str]) -> dict[str, str]:
        cleaned = {path.strip().replace("\\", "/"): digest.strip() for path, digest in value.items() if path.strip()}
        if not cleaned:
            raise ValueError("track execution lock source_digests must not be empty")
        for path, digest in cleaned.items():
            if not re.fullmatch(r"[0-9a-f]{64}", digest):
                raise ValueError(f"track execution lock digest for {path} must be sha256 hex")
        return cleaned

    @model_validator(mode="after")
    def validate_permissions_and_gates(self) -> TrackExecutionLock:
        unknown_permissions = sorted((set(self.granted_permissions) | set(self.denied_permissions)) - MANIFEST_RUNNER_PERMISSIONS)
        if unknown_permissions:
            raise ValueError("track execution lock has unknown permissions: " + ", ".join(unknown_permissions))
        overlaps = sorted(set(self.granted_permissions) & set(self.denied_permissions))
        if overlaps:
            raise ValueError("track execution lock grants and denies the same permissions: " + ", ".join(overlaps))
        unknown_gates = sorted(set(self.strategic_human_gates) - STRATEGIC_HUMAN_GATES)
        if unknown_gates:
            raise ValueError("track execution lock has unknown strategic human gates: " + ", ".join(unknown_gates))
        return self


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
        missing = [
            field_name
            for field_name in required_by_kind[self.evidence_kind]
            if not getattr(self, field_name)
        ]
        if missing:
            raise ValueError(
                f"truth evidence {self.evidence_kind} requires " + ", ".join(missing)
            )
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


AgentDesignAuthoringStrategy = Literal["template_contract_writer", "codex_contract_writer"]


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

    @field_validator("planning_write_scope")
    @classmethod
    def validate_planning_write_scope(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator(
        "allowed_write_scopes",
        "forbidden_scopes",
        "expected_output_paths",
        "validation_commands",
        "stop_conditions",
    )
    @classmethod
    def validate_optional_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]

    @field_validator("agent_context_files", "agent_required_outputs")
    @classmethod
    def validate_agent_lists(cls, value: list[str]) -> list[str]:
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
        allowed = {"isolated_action_workspace"}
        if cleaned not in allowed:
            raise ValueError(f"agent_design agent_worktree_policy must be one of {sorted(allowed)}")
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
        allowed = {"isolated_action_workspace"}
        if cleaned not in allowed:
            raise ValueError(f"implementation_writer agent_worktree_policy must be one of {sorted(allowed)}")
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

    @field_validator("milestone_id", "wr_id", "completion_quality", "closeout_path")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
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
            "known_gaps",
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

    @field_validator("evidence_manifest_path")
    @classmethod
    def validate_optional_text(cls, value: str | None) -> str | None:
        if value is None:
            return None
        cleaned = value.strip()
        return cleaned or None

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

    @field_validator(
        "milestone_id",
        "title",
        "stage",
        "authority_level",
        "expected_closeout_path",
        "next_legal_action",
    )
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("manifest milestone text fields must not be empty")
        return cleaned

    @field_validator(
        "write_scope",
        "forbidden_scope",
        "required_contracts",
        "validation_commands",
        "evidence_gates",
        "stop_conditions",
    )
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


@dataclass(frozen=True)
class LoadedTrackExecutionLock:
    lock: TrackExecutionLock
    path: Path


def manifest_source_path(track_id: str, root: Path = TRACK_EXECUTION_MANIFEST_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


def lock_source_path(track_id: str, root: Path = TRACK_EXECUTION_LOCK_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


def find_track(planning: ProductionPlanningState, track_id: str) -> ProductionTrack:
    for track in planning.tracks:
        if track.id == track_id:
            return track
    raise WorkflowError(f"{track_id}: not present in production tracks source")


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


def load_track_execution_manifest(
    track_id: str,
    *,
    root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> LoadedTrackExecutionManifest | None:
    path = manifest_source_path(track_id, root=root)
    if not path.exists():
        return None
    try:
        with path.open("r", encoding="utf-8") as source:
            data = yaml.safe_load(source)
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


def load_track_execution_lock(
    track_id: str,
    *,
    root: Path = TRACK_EXECUTION_LOCK_ROOT,
) -> LoadedTrackExecutionLock | None:
    path = lock_source_path(track_id, root=root)
    if not path.exists():
        return None
    try:
        with path.open("r", encoding="utf-8") as source:
            data = yaml.safe_load(source)
    except yaml.YAMLError as error:
        raise WorkflowError(f"{repo_path(path)} contains malformed YAML: {error}") from error
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    try:
        lock = TrackExecutionLock.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid Track Execution Lock: {error}") from error
    if lock.track_id != track_id:
        raise WorkflowError(f"{repo_path(path)} declares track_id={lock.track_id}, expected {track_id}")
    return LoadedTrackExecutionLock(lock=lock, path=path)


def sha256_file(path: Path) -> str:
    if not path.exists():
        raise WorkflowError(f"cannot digest missing source file: {repo_path(path)}")
    return sha256(path.read_bytes()).hexdigest()


def source_digest_paths(
    loaded: LoadedTrackExecutionManifest,
    *,
    production_source: Path,
    roadmap_source: Path,
) -> list[Path]:
    paths: list[Path] = [production_source, roadmap_source]
    archive_source, deferred_source = split_source_paths(roadmap_source)
    paths.extend(path for path in (archive_source, deferred_source) if path.exists())
    paths.append(loaded.path)
    paths.extend(TRACK_EXECUTION_LOCKED_WORKFLOW_SOURCES)
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


def build_track_execution_lock_data(
    loaded: LoadedTrackExecutionManifest,
    *,
    production_source: Path,
    roadmap_source: Path,
    locked_by: str,
    granted_permissions: list[str],
    denied_permissions: list[str],
    strategic_human_gates: list[str] | None = None,
) -> dict:
    digests = source_digest_map(loaded, production_source=production_source, roadmap_source=roadmap_source)
    manifest_digest = digests.get(repo_path(loaded.path), sha256_file(loaded.path))
    return {
        "version": 1,
        "track_id": loaded.manifest.track_id,
        "ai_executable": True,
        "locked_by": locked_by,
        "locked_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "manifest_digest": manifest_digest,
        "source_digests": digests,
        "granted_permissions": granted_permissions,
        "denied_permissions": denied_permissions,
        "strategic_human_gates": strategic_human_gates
        or [
            "adr_acceptance",
            "locked_design_direction_change",
            "foundation_meta_extraction",
            "crate_creation_unless_pre_authorized",
            "external_plugin_security_boundary",
            "second_domain_extraction",
            "release_perfectionist_verified_certification",
        ],
        "invalidation_rules": [
            "invalidate when manifest_digest changes before a new full-track run starts",
            "invalidate when production, roadmap, or accepted design source digests change before a new full-track run starts",
            "invalidate when requested permissions exceed granted_permissions or intersect denied_permissions",
            "invalidate when a remaining milestone crosses a listed strategic_human_gate",
        ],
    }


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
        expected_type_errors = manifest_type_errors(entry, milestone_kind=milestone.kind)
        errors.extend(f"{entry.milestone_id}: {error}" for error in expected_type_errors)
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


def is_generated_or_derived_scope(scope: str) -> bool:
    cleaned = scope.strip().lower()
    return cleaned.startswith(GENERATED_SCOPE_PREFIXES)


def mentions_generated_or_derived_scope(scope: str) -> bool:
    cleaned = scope.strip().lower()
    return "generated" in cleaned or "derived" in cleaned


def manifest_write_scope_path(scope: str) -> str | None:
    normalized = normalize_write_scope_path(scope)
    if not normalized or normalized.startswith("blocked:") or " " in normalized:
        return None
    if "/" not in normalized:
        return None
    return normalized


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
        if not path.exists():
            return [f"{label} requires document {evidence.path} ({evidence.reason})"]
        return []
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
        if not path.exists():
            return [f"{label} requires module path {evidence.path} ({evidence.reason})"]
        return []
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
        return validation_command_errors(
            [evidence.command],
            label=f"{label} validation",
            product_code_eligible=True,
        )
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
    satisfied_by_kind_and_level = {
        (claim.claim_kind, claim.claim_level)
        for claim in manifest.truth_claims
        if claim.claim_status == "satisfied"
    }
    blocked_by_kind_and_level = {
        (claim.claim_kind, claim.claim_level)
        for claim in manifest.truth_claims
        if claim.claim_status == "blocked"
    }
    if track.target_completion_quality == "proof_slice_runtime_proven" and (
        "proof_slice",
        "proof_slice_runtime_proven",
    ) not in satisfied_by_kind_and_level:
        errors.append(
            f"{track.id}: target_completion_quality proof_slice_runtime_proven requires a satisfied "
            "proof_slice truth claim at proof_slice_runtime_proven"
        )
    if track.target_completion_quality == "architecture_runtime_proven":
        architecture_key = ("architecture_contract", "architecture_runtime_proven")
        if architecture_key not in satisfied_by_kind_and_level:
            if track.state == "completed":
                errors.append(
                    f"{track.id}: target_completion_quality architecture_runtime_proven requires a satisfied "
                    "architecture_contract truth claim at architecture_runtime_proven"
                )
            elif architecture_key not in blocked_by_kind_and_level:
                errors.append(
                    f"{track.id}: active target_completion_quality architecture_runtime_proven requires either a "
                    "satisfied or blocked architecture_contract truth claim at architecture_runtime_proven"
                )
            else:
                overclaim_texts = [track.strategic_goal, *track.success_criteria]
                for text in overclaim_texts:
                    normalized = text.lower()
                    if (
                        "architecture is implemented" in normalized
                        or "architecture exists" in normalized
                        or "architecture is proven" in normalized
                        or "unblocks materialprogram" in normalized
                    ):
                        errors.append(
                            f"{track.id}: production wording claims architecture truth while the "
                            f"architecture_runtime_proven truth claim is blocked: {text}"
                        )
    if track.target_completion_quality == "proof_slice_runtime_proven":
        overclaim_texts = [track.strategic_goal, *track.success_criteria]
        for text in overclaim_texts:
            normalized = text.lower()
            proof_slice_language = "proof-slice" in normalized or "proof slice" in normalized or "bounded" in normalized
            if ("is proven" in normalized or "proves " in normalized or normalized.startswith("prove ")) and not proof_slice_language:
                errors.append(
                    f"{track.id}: production wording claims stronger truth than proof_slice_runtime_proven: {text}"
                )
            if "enables the materialprogram" in normalized or "enables materialprogram" in normalized:
                errors.append(
                    f"{track.id}: production wording enables MaterialProgram despite corrected truth claims: {text}"
                )
    return errors


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
    if "manifest write_scope" in error:
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


def manifest_entry_execution_kind(entry: TrackExecutionManifestMilestone) -> str | None:
    return entry.execution_kind


def execution_kind_compatibility_errors(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
) -> list[str]:
    execution_kind = manifest_entry_execution_kind(entry)
    if execution_kind is None:
        return [f"{entry.milestone_id}: execution_kind is required for full automation"]
    expected = {
        "design_contract": ({"docs_only", "design_only"}, {"design"}),
        "implementation_proof": ({"implementation"}, {"implementation"}),
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
    execution_kind = manifest_entry_execution_kind(entry)
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
        errors.append(
            f"{entry.milestone_id}: permission_classes_required missing "
            + ", ".join(missing_declared)
        )
    if allow is not None:
        ungranted = sorted(
            permission
            for permission in required_permissions
            if not full_automation_permission_granted(permission, allow)
        )
        if ungranted:
            errors.append(
                f"{entry.milestone_id}: full automation requires ungranted permissions "
                + ", ".join(ungranted)
            )
    if "foundation_extraction" in declared_permissions:
        errors.append(f"{entry.milestone_id}: full automation must not require foundation_extraction")
    if entry.may_create_crates or "crate_creation" in declared_permissions:
        errors.append(f"{entry.milestone_id}: full automation must not require crate_creation unless a separate gate allows it")

    errors.extend(full_automation_contract_errors(entry, execution_kind=execution_kind or ""))
    errors.extend(full_automation_scope_errors(entry))
    errors.extend(
        validation_command_errors(
            entry.validation_commands,
            label=f"{entry.milestone_id}: manifest validation_commands",
            product_code_eligible=True,
        )
    )
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
    execution_kind = manifest_entry_execution_kind(entry)
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
        errors.extend(
            exact_scope_list_errors(
                contract.allowed_write_scopes,
                label=f"{entry.milestone_id}: agent_design_contract allowed_write_scopes",
            )
        )
        errors.extend(
            validation_command_errors(
                contract.validation_commands,
                label=f"{entry.milestone_id}: agent_design_contract",
                product_code_eligible=True,
            )
        )
    if entry.product_code_contract is not None:
        product_scopes = entry.product_code_contract.exact_allowed_implementation_write_scopes
        errors.extend(
            exact_scope_list_errors(
                product_scopes,
                label=f"{entry.milestone_id}: product_code_contract exact_allowed_implementation_write_scopes",
            )
        )
        errors.extend(new_file_scope_errors(entry.milestone_id, product_scopes, label="product_code_contract"))
        errors.extend(
            validation_command_errors(
                entry.product_code_contract.validation_commands,
                label=f"{entry.milestone_id}: product_code_contract",
                product_code_eligible=True,
            )
        )
        if not entry.product_code_contract.forbidden_implementation_scopes:
            errors.append(f"{entry.milestone_id}: product_code_contract must declare forbidden implementation scopes")
    if entry.implementation_writer is not None and entry.implementation_writer.strategy != "no_writer":
        writer_scopes = implementation_writer_allowed_scopes(entry.implementation_writer)
        errors.extend(
            exact_scope_list_errors(
                writer_scopes,
                label=f"{entry.milestone_id}: implementation_writer allowed scope",
            )
        )
        errors.extend(new_file_scope_errors(entry.milestone_id, writer_scopes, label="implementation_writer"))
        if not implementation_writer_forbidden_scopes(entry.implementation_writer):
            errors.append(f"{entry.milestone_id}: implementation_writer must declare forbidden scopes")
        errors.extend(
            validation_command_errors(
                entry.implementation_writer.validation_commands,
                label=f"{entry.milestone_id}: implementation_writer",
                product_code_eligible=True,
            )
        )
    if entry.runtime_closeout_contract is not None:
        if not entry.runtime_closeout_contract.runtime_test_evidence_required:
            errors.append(f"{entry.milestone_id}: runtime_closeout_contract must declare runtime evidence")
        if entry.runtime_closeout_contract.closeout_path != entry.expected_closeout_path:
            errors.append(f"{entry.milestone_id}: runtime_closeout_contract closeout_path must match expected_closeout_path")
        errors.extend(
            validation_command_errors(
                entry.runtime_closeout_contract.validation_commands,
                label=f"{entry.milestone_id}: runtime_closeout_contract",
                product_code_eligible=True,
            )
        )
    if entry.agent_closeout_contract is not None:
        if entry.agent_closeout_contract.closeout_path != entry.expected_closeout_path:
            errors.append(f"{entry.milestone_id}: agent_closeout_contract closeout_path must match expected_closeout_path")
        errors.extend(
            validation_command_errors(
                entry.agent_closeout_contract.validation_commands,
                label=f"{entry.milestone_id}: agent_closeout_contract",
                product_code_eligible=True,
            )
        )
    if entry.handoff_contract is not None:
        errors.extend(
            validation_command_errors(
                entry.handoff_contract.validation_commands,
                label=f"{entry.milestone_id}: handoff_contract",
                product_code_eligible=True,
            )
        )
    return errors


def new_file_scope_errors(milestone_id: str, scopes: list[str], *, label: str) -> list[str]:
    errors: list[str] = []
    for scope in scopes:
        if is_generated_or_derived_scope(scope):
            continue
        if is_new_write_scope(scope):
            continue
        cleaned = scope.strip()
        raw_candidate = Path(cleaned)
        if raw_candidate.is_absolute():
            if not write_scope_path_requires_new_marker(raw_candidate):
                continue
            normalized = normalize_repo_path(cleaned)
            errors.append(f"{milestone_id}: {label} new file scope must be marked with 'new:': {normalized}")
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            continue
        path = REPO_ROOT / normalized
        if not write_scope_path_requires_new_marker(path):
            continue
        errors.append(f"{milestone_id}: {label} new file scope must be marked with 'new:': {normalized}")
    return errors


def write_scope_path_requires_new_marker(path: Path) -> bool:
    try:
        path.resolve().relative_to(REPO_ROOT.resolve())
    except ValueError:
        return not path.exists()
    return not git_tracks_path(path)


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


def full_automation_preflight_or_raise(
    loaded: LoadedTrackExecutionManifest,
    *,
    track: ProductionTrack,
    roadmap: RoadmapState,
    allow: set[str] | None = None,
) -> None:
    errors = full_automation_preflight_errors(loaded, track=track, roadmap=roadmap, allow=allow)
    if errors:
        raise WorkflowError("\n".join(full_automation_blocker_lines(errors)))


def track_execution_lock_errors(
    loaded_manifest: LoadedTrackExecutionManifest,
    loaded_lock: LoadedTrackExecutionLock | None,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    deny: set[str],
    track: ProductionTrack | None = None,
) -> list[str]:
    if loaded_lock is None:
        expected_path = lock_source_path(loaded_manifest.manifest.track_id)
        return [f"{loaded_manifest.manifest.track_id}: full-track execution requires Track Execution Lock at {repo_path(expected_path)}"]
    lock = loaded_lock.lock
    errors: list[str] = []
    if lock.track_id != loaded_manifest.manifest.track_id:
        errors.append(
            f"{repo_path(loaded_lock.path)}: lock track_id {lock.track_id} does not match manifest track_id {loaded_manifest.manifest.track_id}"
        )
    if not lock.ai_executable:
        errors.append(f"{lock.track_id}: execution_lock.ai_executable must be true for full-track execution")
    if not loaded_manifest.manifest.ai_executable:
        errors.append(f"{lock.track_id}: manifest ai_executable must be true for full-track execution")

    try:
        current_digests = source_digest_map(
            loaded_manifest,
            production_source=production_source,
            roadmap_source=roadmap_source,
        )
    except WorkflowError as error:
        errors.append(str(error))
        current_digests = {}
    current_manifest_digest = current_digests.get(repo_path(loaded_manifest.path))
    if current_manifest_digest is not None and lock.manifest_digest != current_manifest_digest:
        errors.append(
            f"{lock.track_id}: execution lock manifest_digest is stale "
            f"(lock={lock.manifest_digest}, current={current_manifest_digest})"
        )
    for path, digest in current_digests.items():
        locked_digest = lock.source_digests.get(path)
        if locked_digest is None:
            errors.append(f"{lock.track_id}: execution lock missing source digest for {path}")
        elif locked_digest != digest:
            errors.append(
                f"{lock.track_id}: execution lock source digest is stale for {path} "
                f"(lock={locked_digest}, current={digest})"
            )
    for path in sorted(set(lock.source_digests) - set(current_digests)):
        errors.append(f"{lock.track_id}: execution lock references source that is no longer part of the manifest input set: {path}")

    requested = set(allow)
    ungranted = sorted(requested - set(lock.granted_permissions))
    if ungranted:
        errors.append(f"{lock.track_id}: requested full-track permissions exceed execution lock grants: {', '.join(ungranted)}")
    denied_requested = sorted(requested & set(lock.denied_permissions))
    if denied_requested:
        errors.append(f"{lock.track_id}: requested full-track permissions are denied by execution lock: {', '.join(denied_requested)}")
    deny_conflicts = sorted(set(deny) & set(lock.granted_permissions))
    if deny_conflicts:
        errors.append(f"{lock.track_id}: command denies permissions granted by execution lock: {', '.join(deny_conflicts)}")

    if track is not None:
        for entry, _milestone, _index in remaining_manifest_entries(loaded_manifest.manifest, track):
            for permission in entry.permission_classes_required:
                gate = STRATEGIC_GATE_PERMISSION_MAP.get(permission)
                if gate and gate in lock.strategic_human_gates and permission not in lock.granted_permissions:
                    errors.append(
                        f"{entry.milestone_id}: strategic human gate {gate} blocks permission {permission}"
                    )
    return errors


def track_execution_lock_or_raise(
    loaded_manifest: LoadedTrackExecutionManifest,
    loaded_lock: LoadedTrackExecutionLock | None,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    deny: set[str],
    track: ProductionTrack | None = None,
) -> None:
    errors = track_execution_lock_errors(
        loaded_manifest,
        loaded_lock,
        production_source=production_source,
        roadmap_source=roadmap_source,
        allow=allow,
        deny=deny,
        track=track,
    )
    if errors:
        raise WorkflowError("\n".join(["Track Execution Lock blockers:", *[f"- {error}" for error in errors]]))


def agent_track_product_lock_or_raise(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    lock_source_root: Path,
    allow: set[str],
    deny: set[str],
) -> None:
    full_automation_preflight_or_raise(
        context.loaded,
        track=context.track,
        roadmap=context.roadmap,
        allow=allow,
    )
    loaded_lock = load_track_execution_lock(context.track.id, root=lock_source_root)
    track_execution_lock_or_raise(
        context.loaded,
        loaded_lock,
        production_source=production_source,
        roadmap_source=roadmap_source,
        allow=allow,
        deny=deny,
        track=context.track,
    )


def try_refresh_agent_track_lock(
    *,
    track_id: str,
    production_source: Path,
    roadmap_source: Path,
    manifest_source_root: Path,
    lock_source_root: Path,
    allow: set[str],
    deny: set[str],
) -> Path | None:
    context = resolve_manifest_command_context(
        track_id,
        production_source=production_source,
        roadmap_source=roadmap_source,
        manifest_source_root=manifest_source_root,
    )
    if full_automation_preflight_errors(context.loaded, track=context.track, roadmap=context.roadmap, allow=allow):
        return None
    data = context.loaded.manifest.model_dump(exclude_none=True, mode="json")
    changed = False
    if not data.get("ai_executable"):
        data["ai_executable"] = True
        changed = True
    if not data.get("full_automation_target"):
        data["full_automation_target"] = True
        changed = True
    if changed:
        manifest = TrackExecutionManifest.model_validate(data)
        write_yaml_mapping(context.loaded.path, manifest.model_dump(exclude_none=True, mode="json"))
        context = resolve_manifest_command_context(
            track_id,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        full_automation_preflight_or_raise(
            context.loaded,
            track=context.track,
            roadmap=context.roadmap,
            allow=allow,
        )
    lock_data = build_track_execution_lock_data(
        context.loaded,
        production_source=production_source,
        roadmap_source=roadmap_source,
        locked_by="agent-track",
        granted_permissions=sorted(allow),
        denied_permissions=sorted(deny),
    )
    lock = TrackExecutionLock.model_validate(lock_data)
    path = lock_source_path(track_id, root=lock_source_root)
    write_yaml_mapping(path, lock.model_dump(mode="json"))
    return path


def full_automation_blocker_lines(errors: list[str]) -> list[str]:
    return ["Full automation readiness blockers:", *[f"- {error}" for error in errors]]


def print_full_automation_blockers(errors: list[str]) -> None:
    lines = full_automation_blocker_lines(errors)
    console.print(f"[red]{lines[0]}[/red]")
    for line in lines[1:]:
        console.print(line)


def should_run_full_automation_preflight(
    *,
    mode: ManifestRunMode,
    preflight_only: bool,
) -> bool:
    if preflight_only:
        return True
    if mode == "full-track":
        return True
    return False


def print_manifest_audit_blockers(errors: list[str]) -> None:
    lines = manifest_audit_blocker_lines(errors)
    console.print(f"[red]{lines[0]}[/red]")
    for line in lines[1:]:
        console.print(line)


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
        values = getattr(entry, field_name)
        for value in values:
            if value.startswith("blocked:"):
                errors.append(f"{entry.milestone_id}: {field_name} remains blocked: {value}")
    if entry.expected_closeout_path.startswith("blocked:"):
        errors.append(f"{entry.milestone_id}: expected_closeout_path remains blocked")
    if not entry.expected_closeout_path.endswith(".md"):
        errors.append(f"{entry.milestone_id}: expected_closeout_path must point at a Markdown closeout/report")
    return errors


def audit_manifest_action_contracts(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track: ProductionTrack,
    manifest: TrackExecutionManifest,
) -> list[str]:
    if milestone.state == "completed":
        return []
    errors: list[str] = []
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
        if entry.product_code_contract is not None:
            contract_scope_errors = exact_scope_list_errors(
                entry.product_code_contract.exact_allowed_implementation_write_scopes,
                label=f"{entry.milestone_id}: product_code_contract exact_allowed_implementation_write_scopes",
            )
            errors.extend(contract_scope_errors)
            errors.extend(
                validation_command_errors(
                    entry.product_code_contract.validation_commands,
                    label=f"{entry.milestone_id}: product_code_contract",
                    product_code_eligible=True,
                )
            )
        if entry.runtime_closeout_contract is not None:
            errors.extend(
                validation_command_errors(
                    entry.runtime_closeout_contract.validation_commands,
                    label=f"{entry.milestone_id}: runtime_closeout_contract",
                    product_code_eligible=True,
                )
            )
    if entry.milestone_type == "closeout":
        if entry.agent_closeout_contract is None:
            errors.append(f"{entry.milestone_id}: remaining closeout milestone needs agent_closeout_contract")
        if entry.handoff_contract is None:
            errors.append(f"{entry.milestone_id}: remaining closeout milestone needs handoff_contract")
    return errors


def agent_design_contract_errors(
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
) -> list[str]:
    errors: list[str] = []
    errors.extend(
        validation_command_errors(
            contract.validation_commands,
            label=f"{entry.milestone_id}: agent_design_contract",
            product_code_eligible=True,
        )
    )
    if contract.authoring_strategy == "codex_contract_writer":
        output_scopes = contract.expected_output_paths or contract.agent_required_outputs
        if not output_scopes:
            errors.append(f"{entry.milestone_id}: codex_contract_writer requires expected_output_paths")
        if not contract.agent_prompt:
            errors.append(f"{entry.milestone_id}: codex_contract_writer requires agent_prompt")
        if not contract.agent_diff_protocol_version:
            errors.append(f"{entry.milestone_id}: codex_contract_writer requires agent_diff_protocol_version")
        if contract.agent_worktree_policy != "isolated_action_workspace":
            errors.append(f"{entry.milestone_id}: codex_contract_writer must use isolated_action_workspace")
        errors.extend(
            exact_scope_list_errors(
                output_scopes,
                label=f"{entry.milestone_id}: codex_contract_writer output scope",
            )
        )
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
    missing_writer_commands = [
        command
        for command in writer.validation_commands
        if command not in product_validation_commands_for_entry(entry)
    ]
    if missing_writer_commands:
        errors.append(
            f"{entry.milestone_id}: implementation_writer validation commands are not covered by product_code_contract: "
            + ", ".join(missing_writer_commands)
        )
    errors.extend(
        exact_scope_list_errors(
            implementation_writer_allowed_scopes(writer),
            label=f"{entry.milestone_id}: implementation_writer allowed scope",
        )
    )
    if writer.strategy == "agent_writer":
        if writer.agent_worktree_policy != "isolated_action_workspace":
            errors.append(f"{entry.milestone_id}: agent_writer must use isolated_action_workspace")
        if not writer.agent_diff_protocol_version:
            errors.append(f"{entry.milestone_id}: agent_writer requires agent_diff_protocol_version")
        if not writer.agent_prompt:
            errors.append(f"{entry.milestone_id}: agent_writer requires a bounded agent_prompt")
        if writer.templates or writer.patches:
            errors.append(f"{entry.milestone_id}: agent_writer cannot also declare templates or patches")
    if writer.strategy == "proof_aggregation_writer":
        errors.extend(proof_aggregation_writer_contract_errors(entry, writer, track=track, manifest=manifest))
    return errors


def normalize_evidence_category(category: str) -> str:
    cleaned = category.strip().lower().replace("-", "_").replace(" ", "_")
    alias_key = category.strip().lower()
    return GENERIC_EVIDENCE_CATEGORY_ALIASES.get(alias_key, GENERIC_EVIDENCE_CATEGORY_ALIASES.get(cleaned, cleaned))


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
        errors.append(
            f"{entry.milestone_id}: proof_aggregation_writer requires required_prior_completion_quality: runtime_proven"
        )
    required_writer_categories = {normalize_evidence_category(category) for category in writer.required_evidence_categories}
    missing_categories = sorted(
        normalize_evidence_category(category)
        for category in PROOF_AGGREGATION_REQUIRED_EVIDENCE_CATEGORIES
        if normalize_evidence_category(category) not in required_writer_categories
    )
    if missing_categories:
        errors.append(
            f"{entry.milestone_id}: proof_aggregation_writer missing required evidence categories: "
            + ", ".join(missing_categories)
        )
    if writer.closeout_path != entry.expected_closeout_path:
        errors.append(
            f"{entry.milestone_id}: proof_aggregation_writer closeout_path must match expected_closeout_path"
        )
    if not [*writer.forbidden_files, *writer.forbidden_scopes]:
        errors.append(f"{entry.milestone_id}: proof_aggregation_writer must declare forbidden files or scopes")

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
            errors.append(
                f"{entry.milestone_id}: required prior milestone {prior_id} closeout is missing: {prior.completion_audit}"
            )
        else:
            status = document_frontmatter_status(closeout)
            if status is None:
                errors.append(
                    f"{entry.milestone_id}: required prior milestone {prior_id} closeout has no frontmatter status"
                )
            elif status.lower() != "completed":
                errors.append(
                    f"{entry.milestone_id}: required prior milestone {prior_id} closeout status {status!r}, expected completed"
                )
            if manifest.full_automation_target:
                try:
                    evidence_record = closeout_evidence_record(closeout)
                except ValueError as error:
                    errors.append(
                        f"{entry.milestone_id}: required prior milestone {prior_id} closeout evidence metadata is invalid: {error}"
                    )
                else:
                    if evidence_record is None:
                        errors.append(
                            f"{entry.milestone_id}: required prior milestone {prior_id} closeout is missing closeout_evidence metadata"
                        )
                    else:
                        if evidence_record.milestone_id != prior_id:
                            errors.append(
                                f"{entry.milestone_id}: required prior milestone {prior_id} closeout evidence "
                                f"has milestone_id {evidence_record.milestone_id}"
                            )
                        if prior.roadmap_links and evidence_record.wr_id not in prior.roadmap_links:
                            errors.append(
                                f"{entry.milestone_id}: required prior milestone {prior_id} closeout evidence "
                                f"wr_id {evidence_record.wr_id} is not one of {prior.roadmap_links}"
                            )
                        if evidence_record.completion_quality != writer.required_prior_completion_quality:
                            errors.append(
                                f"{entry.milestone_id}: required prior milestone {prior_id} closeout evidence "
                                f"completion_quality {evidence_record.completion_quality}, expected "
                                f"{writer.required_prior_completion_quality}"
                            )
                        normalized_closeout = normalize_repo_path(prior.completion_audit)
                        if normalize_repo_path(evidence_record.closeout_path) != normalized_closeout:
                            errors.append(
                                f"{entry.milestone_id}: required prior milestone {prior_id} closeout evidence "
                                f"closeout_path must match {normalized_closeout}"
                            )
                        aggregate_evidence_categories.update(
                            normalize_evidence_category(category)
                            for category in evidence_record.evidence_categories
                        )

    if manifest.full_automation_target:
        normalized_required_record_categories = {
            normalize_evidence_category(category)
            for category in writer.required_evidence_categories
        }
        missing_record_categories = sorted(
            normalized_required_record_categories - aggregate_evidence_categories
        )
        if missing_record_categories:
            errors.append(
                f"{entry.milestone_id}: proof_aggregation_writer prior closeout evidence metadata missing categories: "
                + ", ".join(missing_record_categories)
            )

    # Do not let aggregation writer output paths patch earlier proof-slice product files.
    prior_product_scopes = []
    writer_scopes = implementation_writer_output_scopes(writer)
    for prior_id in writer.required_prior_milestones:
        prior_entry = manifest.by_milestone_id.get(prior_id)
        if prior_entry is not None:
            prior_product_scopes.extend(product_implementation_scopes_for_entry(prior_entry))
    if prior_product_scopes:
        prior_paths = [
            normalized
            for normalized in (manifest_write_scope_path(scope) for scope in prior_product_scopes)
            if normalized is not None
        ]
        for scope in writer_scopes:
            writer_path = manifest_write_scope_path(scope)
            if writer_path is None:
                continue
            for prior_path in prior_paths:
                if path_within_scope(writer_path, prior_path) or path_within_scope(prior_path, writer_path):
                    errors.append(
                        f"{entry.milestone_id}: proof_aggregation_writer must not modify prior proof-slice product file {prior_path}"
                    )
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
        if normalized in broad_roots or len(normalized.split("/")) < 3:
            errors.append(f"{label} is too broad: {scope}")
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
    if entry.milestone_type == "closeout" and action == "design_first" and agent_closeout_pending(entry):
        return action, blockers
    if (
        entry.milestone_type in {"implementation", "hardening"}
        and action == "write_implementation_contract"
        and product_implementation_completed(entry)
    ):
        evidence_errors = runtime_evidence_errors(entry)
        closeout_path = REPO_ROOT / normalize_repo_path(entry.expected_closeout_path)
        if evidence_errors:
            blockers.extend(evidence_errors)
            return "runtime_evidence_missing", blockers
        if not closeout_path.exists():
            return "runtime_closeout", blockers
    if action != "write_implementation_contract":
        blockers.append(f"{item.id}: workflow action is {action} (state={item.planning_state}, blocker={item.blocker_label})")
    return action, blockers


def product_implementation_completed(entry: TrackExecutionManifestMilestone) -> bool:
    return "product_implementation completed" in entry.next_legal_action.lower()


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


def future_wr_candidate_for_milestone(milestone: ProductionMilestone) -> str:
    suffix = milestone.id.removeprefix("PM-")
    return f"WR-TBD-{suffix}"


def manifest_type_for_milestone(milestone: ProductionMilestone) -> ManifestMilestoneType:
    if milestone.kind == "implementation":
        return "implementation"
    if milestone.kind == "hardening":
        return "hardening"
    if milestone.kind == "release":
        return "closeout"
    return "design_only"


def build_manifest_scaffold(track: ProductionTrack, roadmap: RoadmapState) -> TrackExecutionManifest:
    design_dependencies: dict[str, ManifestDesignDependency] = {}
    for milestone in track.milestones:
        for gate in milestone.design_gates:
            design_dependencies[gate.path] = ManifestDesignDependency(
                path=gate.path,
                required_status=gate.required_status,
                reason=gate.reason,
            )
    if not design_dependencies:
        design_dependencies["blocked: define accepted design dependency"] = ManifestDesignDependency(
            path="blocked: define accepted design dependency",
            required_status="active",
            reason="Track manifest scaffold requires an accepted design dependency before goal execution.",
        )

    global_validation_commands = [
        "task production:render",
        "task production:validate",
        "task production:check",
        "task roadmap:render",
        "task roadmap:validate",
        "task roadmap:check",
        "task docs:validate",
        "task planning:validate",
    ]
    milestones: list[TrackExecutionManifestMilestone] = []
    for milestone in ordered_track_milestones(track):
        if len(milestone.roadmap_links) > 1:
            raise WorkflowError(f"{milestone.id}: cannot scaffold manifest for multiple roadmap links")
        owning_wr = milestone.roadmap_links[0] if milestone.roadmap_links else None
        future_wr = None if owning_wr else future_wr_candidate_for_milestone(milestone)
        roadmap_item: RoadmapItem | None = roadmap.by_id.get(owning_wr) if owning_wr else None
        write_scope = roadmap_item.write_scopes if roadmap_item and roadmap_item.write_scopes else [
            f"blocked: define exact write scope for {milestone.id}"
        ]
        validation_commands = roadmap_item.validations if roadmap_item and roadmap_item.validations else global_validation_commands
        milestones.append(
            TrackExecutionManifestMilestone(
                milestone_id=milestone.id,
                title=milestone.title,
                stage=f"blocked: assign stage for {milestone.id}",
                authority_level="planning_and_sequencing_only",
                milestone_type=manifest_type_for_milestone(milestone),
                owning_wr=owning_wr,
                future_wr_candidate=future_wr,
                predecessor_dependencies=milestone.dependencies,
                write_scope=write_scope,
                forbidden_scope=["no implementation from this manifest alone", "no crate creation without separate authority"],
                required_contracts=[f"blocked: define required contract for {milestone.id}"],
                validation_commands=validation_commands,
                evidence_gates=[f"blocked: define evidence gate for {milestone.id}"],
                expected_closeout_path=(
                    "docs-site/src/content/docs/reports/closeouts/"
                    f"{milestone.id.lower()}-{slugify(milestone.title)}/closeout.md"
                ),
                stop_conditions=[
                    "stop if validation fails",
                    "stop if WR authority is missing",
                    "stop before implementation unless a production plan authorizes a bounded slice",
                ],
                next_legal_action=(
                    f"Use owning WR {owning_wr} to plan the next bounded action."
                    if owning_wr
                    else f"Track Expansion must create or link {future_wr}."
                ),
                may_create_code=False,
                may_create_crates=False,
                may_modify_production_behavior=False,
            )
        )
    return TrackExecutionManifest(
        version=1,
        track_id=track.id,
        authority_level="planning_and_sequencing_only",
        accepted_design_dependencies=list(design_dependencies.values()),
        global_forbidden_scope=[
            "no product code from this manifest alone",
            "no new crates from this manifest alone",
            "no production behavior changes from this manifest alone",
        ],
        global_validation_commands=global_validation_commands,
        global_stop_conditions=[
            "stop if manifest data conflicts with production track or roadmap state",
            "stop if a milestone lacks WR authority",
            "stop after one legal action and rerun task ai:goal",
        ],
        next_legal_action="blocked: review generated scaffold and replace blocked fields before relying on full-track execution",
        milestones=milestones,
    )


def default_contract_template_key(entry: TrackExecutionManifestMilestone) -> str:
    if entry.milestone_type in {"implementation", "hardening"}:
        return "implementation_runtime_proof"
    if entry.milestone_type == "closeout":
        return "final_handoff_closeout"
    return "docs_design"


def contract_template_key(entry: TrackExecutionManifestMilestone) -> str:
    key = entry.template_key or default_contract_template_key(entry)
    if key not in CONTRACT_TEMPLATE_KEYS:
        raise WorkflowError(f"{entry.milestone_id}: missing contract template {key!r}")
    return key


def default_source_documents(manifest: TrackExecutionManifest, entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.contract_parameters and entry.contract_parameters.source_documents:
        return list(entry.contract_parameters.source_documents)
    return [dependency.path for dependency in manifest.accepted_design_dependencies]


def exact_contract_implementation_scopes(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.contract_parameters and entry.contract_parameters.exact_allowed_implementation_write_scopes:
        return list(entry.contract_parameters.exact_allowed_implementation_write_scopes)
    exact_scopes = product_implementation_scopes_for_entry(entry)
    if exact_scope_list_errors(exact_scopes, label=f"{entry.milestone_id}: product_code_contract exact scopes"):
        raise WorkflowError(
            f"{entry.milestone_id}: product_code_contract cannot be generated safely without exact implementation write scopes"
        )
    return exact_scopes


def default_contract_params_list(
    entry: TrackExecutionManifestMilestone,
    field_name: str,
    fallback: list[str],
) -> list[str]:
    params = entry.contract_parameters
    if params is None:
        return fallback
    value = getattr(params, field_name)
    return list(value) if value else fallback


def planning_scope_for_contract(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track_id: str,
    manifest_path: Path,
    production_source: Path,
    roadmap_source: Path,
) -> list[str]:
    wr_id = entry.owning_wr or entry.future_wr_candidate or "WR-TBD"
    _, deferred_source = split_source_paths(roadmap_source)
    return exact_auto_safe_write_scope(
        entry,
        milestone,
        track_id=track_id,
        wr_id=wr_id,
        production_source=production_source,
        manifest_source=manifest_path,
        roadmap_source=roadmap_source,
        deferred_source=deferred_source,
    )


def generated_contract_fields(template_key: str) -> dict[str, str]:
    return {
        "template_key": template_key,
        "generated_contract_marker": CONTRACT_GENERATED_MARKER,
        "generated_from_template_version": CONTRACT_TEMPLATE_VERSION,
    }


def build_auto_safe_contract_data(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track_id: str,
    manifest_path: Path,
    production_source: Path,
    roadmap_source: Path,
) -> dict:
    template_key = contract_template_key(entry)
    wr_candidate = entry.future_wr_candidate or entry.owning_wr or "owning WR"
    return {
        "wr_candidate_policy": f"Use {wr_candidate} only as manifest planning input; allocate a concrete WR id if the candidate is future-only.",
        "wr_id_allocation_behavior": "Allocate the next numeric WR id from combined roadmap state; never reuse archived or deferred ids.",
        "milestone_to_wr_link_behavior": "Update exactly this production milestone roadmap_links field to the allocated WR.",
        "manifest_wr_reference_behavior": "Replace future_wr_candidate with owning_wr for this milestone and leave other milestones untouched.",
        "allowed_metadata_write_scopes": planning_scope_for_contract(
            entry,
            milestone,
            track_id=track_id,
            manifest_path=manifest_path,
            production_source=production_source,
            roadmap_source=roadmap_source,
        ),
        "forbidden_scopes": list(entry.forbidden_scope),
        "validation_commands": auto_safe_validation_commands(),
        "stop_conditions": [
            "stop if production, roadmap, or manifest alignment fails",
            "stop if dependency milestones are incomplete",
            "stop before plan/design/product-code work",
        ],
        **generated_contract_fields(template_key),
    }


def build_agent_design_contract_data(
    manifest: TrackExecutionManifest,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    manifest_path: Path,
    production_source: Path,
    roadmap_source: Path,
) -> dict:
    template_key = contract_template_key(entry)
    sections = default_contract_params_list(
        entry,
        "required_sections",
        [
            f"{entry.title} bounded implementation plan",
            "Exact files/modules allowed",
            "Forbidden scope",
            "Tests to add/change",
            "Validation commands",
            "Closeout evidence",
            "Compatibility / rollback expectations",
            "Stop conditions",
        ],
    )
    decisions = default_contract_params_list(
        entry,
        "required_decisions",
        [
            f"{entry.milestone_id} remains bounded to {entry.stage}.",
            "No product/runtime code is changed by agent_design.",
            "Product code remains blocked until an accepted implementation plan and explicit product_code permission exist.",
        ],
    )
    acceptance = default_contract_params_list(
        entry,
        "acceptance_checklist",
        [
            "The implementation plan names exact write scopes and forbidden scopes.",
            "The implementation plan lists validation commands, closeout evidence, rollback/compatibility expectations, and stop conditions.",
            "No product code, crates, downstream implementation, or foundation/meta extraction occurs.",
        ],
    )
    expected_outputs = [
        implementation_plan_path(entry.owning_wr or entry.future_wr_candidate or "WR-TBD", milestone),
    ]
    return {
        "source_documents": default_source_documents(manifest, entry),
        "required_sections": sections,
        "required_decisions": decisions,
        "acceptance_checklist": acceptance,
        "planning_write_scope": planning_scope_for_contract(
            entry,
            milestone,
            track_id=manifest.track_id,
            manifest_path=manifest_path,
            production_source=production_source,
            roadmap_source=roadmap_source,
        ),
        "allowed_write_scopes": planning_scope_for_contract(
            entry,
            milestone,
            track_id=manifest.track_id,
            manifest_path=manifest_path,
            production_source=production_source,
            roadmap_source=roadmap_source,
        ),
        "forbidden_scopes": list(entry.forbidden_scope),
        "expected_output_paths": expected_outputs,
        "validation_commands": default_contract_params_list(
            entry,
            "validation_commands",
            list(entry.validation_commands),
        ),
        "stop_conditions": [
            "stop before product/runtime code",
            "stop if exact implementation write scopes are missing or ambiguous",
            "stop if validation fails",
        ],
        **generated_contract_fields(template_key),
    }


def build_product_code_contract_data(entry: TrackExecutionManifestMilestone) -> dict:
    template_key = contract_template_key(entry)
    exact_scopes = exact_contract_implementation_scopes(entry)
    writer_commands = (
        list(entry.implementation_writer.validation_commands)
        if entry.implementation_writer is not None and entry.implementation_writer.strategy != "no_writer"
        else []
    )
    validation_commands = writer_commands or list(entry.validation_commands)
    return {
        "required_active_wr_state": "owning WR must be current_candidate with blocker B2 or lower",
        "required_accepted_implementation_plan": "accepted implementation plan must exist at the owning WR plan path with active/accepted/completed frontmatter status",
        "exact_allowed_implementation_write_scopes": exact_scopes,
        "required_function_method_scope": default_contract_params_list(
            entry,
            "required_function_method_scope",
            [f"bounded {entry.title} functions/methods named by the accepted implementation plan"],
        ),
        "forbidden_implementation_scopes": list(entry.forbidden_scope),
        "tests_to_add_change": default_contract_params_list(
            entry,
            "tests_to_add_change",
            [f"focused {entry.stage} tests named by the accepted implementation plan"],
        ),
        "runtime_evidence_required": default_contract_params_list(
            entry,
            "runtime_evidence_required",
            [f"{entry.stage} runtime/test evidence for {entry.title}"],
        ),
        "validation_commands": default_contract_params_list(
            entry,
            "validation_commands",
            validation_commands,
        ),
        "rollback_compatibility_expectations": default_contract_params_list(
            entry,
            "rollback_compatibility_expectations",
            ["Rollback is limited to the exact allowed implementation write scopes."],
        ),
        "closeout_evidence": default_contract_params_list(
            entry,
            "closeout_evidence",
            [entry.expected_closeout_path],
        ),
        "stop_conditions": [
            "stop if product_code permission is not explicitly granted",
            "stop if the active WR or accepted implementation plan is missing",
            "stop if validation fails",
            "stop after one implementation WR unless the runner recomputes and all closeout gates pass",
        ],
        **generated_contract_fields(template_key),
    }


def build_runtime_closeout_contract_data(entry: TrackExecutionManifestMilestone) -> dict:
    template_key = contract_template_key(entry)
    return {
        "runtime_test_evidence_required": default_contract_params_list(
            entry,
            "runtime_evidence_required",
            [f"{entry.stage} runtime/test validation evidence"],
        ),
        "validation_commands": default_contract_params_list(
            entry,
            "validation_commands",
            list(entry.validation_commands),
        ),
        "completion_quality_allowed": ["runtime_proven"],
        "closeout_path": entry.expected_closeout_path,
        "files_changed_report": product_implementation_scopes_for_entry(entry),
        "known_gap_reporting": [
            f"{entry.milestone_id} is runtime_proven only for its bounded manifest and product_code_contract scopes.",
            "Later milestones require their own WRs, plans, validation, and closeout evidence.",
        ],
        "production_roadmap_state_updates": [
            "set production milestone state to completed",
            "archive owning WR as completed",
            "record completion_quality runtime_proven",
            "record closeout path as completion audit/evidence",
        ],
        "next_action_update_rules": [
            "advance manifest next legal action to the next milestone",
            "do not authorize crate creation, downstream implementation, or foundation/meta extraction",
        ],
        **generated_contract_fields(template_key),
    }


def build_agent_closeout_contract_data(entry: TrackExecutionManifestMilestone) -> dict:
    template_key = contract_template_key(entry)
    quality = "runtime_proven" if entry.milestone_type == "closeout" else "bounded_contract"
    return {
        "evidence_files": default_contract_params_list(
            entry,
            "closeout_evidence",
            [entry.expected_closeout_path],
        ),
        "validation_commands": list(entry.validation_commands),
        "completion_quality_allowed": [quality],
        "closeout_path": entry.expected_closeout_path,
        "production_roadmap_state_updates": [
            "set production milestone state to completed only after evidence is valid",
            "archive or update owning WR only through the closeout workflow",
            f"record completion_quality {quality}",
        ],
        "known_gap_reporting": [
            "closeout must report unresolved gaps honestly",
            "deferred work must not be described as completed",
        ],
        "next_action_update_rules": [
            "advance manifest next legal action to the next legal milestone action",
            "do not authorize product code from closeout metadata alone",
        ],
        **generated_contract_fields(template_key),
    }


def build_handoff_contract_data(entry: TrackExecutionManifestMilestone) -> dict:
    template_key = contract_template_key(entry)
    return {
        "handoff_target": f"{entry.title} handoff target",
        "proof_path_rules": [
            "create or link the next proof planning path only after this track proof is closed",
            "do not start downstream implementation from this handoff closeout",
            "do not substitute a different proof path without a separately accepted design or ADR",
        ],
        "forbidden_scopes": list(entry.forbidden_scope),
        "validation_commands": list(entry.validation_commands),
        "stop_conditions": [
            "stop if any required proof evidence is missing",
            "stop if handoff would imply shared foundation/meta extraction",
            "stop if handoff would start downstream implementation",
        ],
        **generated_contract_fields(template_key),
    }


def contract_permission_classes(entry: TrackExecutionManifestMilestone) -> list[str]:
    permissions: list[str] = []
    if entry.future_wr_candidate:
        permissions.append("auto_safe")
    if entry.milestone_type in {"docs_only", "design_only", "implementation", "hardening"}:
        permissions.append("agent_design")
    if entry.milestone_type in {"docs_only", "design_only", "closeout"}:
        permissions.append("agent_closeout")
    if entry.milestone_type in {"implementation", "hardening"}:
        permissions.extend(["product_code", "product_implementation", "runtime_closeout"])
    if entry.milestone_type == "closeout":
        permissions.append("handoff")
    return list(dict.fromkeys(permissions))


def default_execution_kind_for_milestone(
    milestone: ProductionMilestone,
    entry: TrackExecutionManifestMilestone,
) -> ManifestExecutionKind | None:
    if milestone.kind == "design":
        return "design_contract"
    if milestone.kind == "implementation":
        return "implementation_proof"
    if milestone.kind == "hardening":
        return "proof_aggregation" if entry.implementation_writer and entry.implementation_writer.strategy == "proof_aggregation_writer" else "implementation_proof"
    if milestone.kind == "release":
        return "handoff_closeout"
    return None


def default_closeout_strategy_for_execution_kind(execution_kind: str | None) -> CloseoutStrategy | None:
    if execution_kind == "design_contract":
        return "bounded_contract_closeout"
    if execution_kind in {"implementation_proof", "proof_aggregation"}:
        return "runtime_proven_closeout"
    if execution_kind == "handoff_closeout":
        return "handoff_closeout"
    if execution_kind == "extraction_gate":
        return "extraction_gate_closeout"
    return None


def complete_contracts_for_manifest_data(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
) -> tuple[dict, list[str], list[str]]:
    base_errors = manifest_alignment_errors(
        context.loaded,
        track=context.track,
        roadmap=context.roadmap,
        ordered_milestone_ids=[milestone.id for milestone in ordered_track_milestones(context.track)],
    )
    if base_errors:
        raise WorkflowError("\n".join(manifest_audit_blocker_lines(base_errors)))

    data = context.loaded.manifest.model_dump(exclude_none=True, mode="json")
    production_by_id = {milestone.id: milestone for milestone in context.track.milestones}
    changed_milestones: list[str] = []
    blockers: list[str] = []
    manifest = context.loaded.manifest
    for milestone_data in data["milestones"]:
        milestone_id = milestone_data["milestone_id"]
        milestone = production_by_id[milestone_id]
        if milestone.state == "completed":
            continue
        entry = TrackExecutionManifestMilestone.model_validate(milestone_data)
        try:
            template_key = contract_template_key(entry)
        except WorkflowError as error:
            blockers.append(str(error))
            continue
        changed = False
        milestone_data["milestone_kind"] = milestone.kind
        execution_kind = entry.execution_kind or default_execution_kind_for_milestone(milestone, entry)
        if execution_kind is not None:
            milestone_data["execution_kind"] = execution_kind
        closeout_strategy = entry.closeout_strategy or default_closeout_strategy_for_execution_kind(execution_kind)
        if closeout_strategy is not None:
            milestone_data["closeout_strategy"] = closeout_strategy
        milestone_data["permission_classes_required"] = contract_permission_classes(entry)
        milestone_data["template_key"] = template_key
        milestone_data["generated_contract_marker"] = CONTRACT_GENERATED_MARKER
        milestone_data["generated_from_template_version"] = CONTRACT_TEMPLATE_VERSION
        milestone_data["required_evidence_categories"] = (
            ["runtime_test", "diagnostics", "source_maps", "artifact", "closeout"]
            if entry.milestone_type in {"implementation", "hardening"}
            else ["design_evidence", "closeout"]
        )
        if entry.milestone_type in {"implementation", "hardening"}:
            milestone_data["implementation_proof_kind"] = entry.implementation_proof_kind or slugify(entry.title)
            milestone_data["closeout_kind"] = "runtime_proven"
        elif entry.milestone_type == "closeout":
            milestone_data["closeout_kind"] = "final_handoff"
        else:
            milestone_data["closeout_kind"] = "bounded_contract"

        if entry.future_wr_candidate and not milestone_data.get("auto_safe_contract"):
            milestone_data["auto_safe_contract"] = build_auto_safe_contract_data(
                entry,
                milestone,
                track_id=context.track.id,
                manifest_path=context.loaded.path,
                production_source=production_source,
                roadmap_source=roadmap_source,
            )
            changed = True
        if entry.milestone_type in {"docs_only", "design_only", "implementation", "hardening"} and not (
            milestone_data.get("agent_design_contract") or milestone_data.get("agent_design")
        ):
            milestone_data["agent_design_contract"] = build_agent_design_contract_data(
                manifest,
                entry,
                milestone,
                manifest_path=context.loaded.path,
                production_source=production_source,
                roadmap_source=roadmap_source,
            )
            changed = True
        if entry.milestone_type in {"implementation", "hardening"}:
            try:
                if not milestone_data.get("product_code_contract"):
                    milestone_data["product_code_contract"] = build_product_code_contract_data(entry)
                    changed = True
                if not milestone_data.get("runtime_closeout_contract"):
                    milestone_data["runtime_closeout_contract"] = build_runtime_closeout_contract_data(entry)
                    changed = True
            except WorkflowError as error:
                blockers.append(str(error))
                continue
        if entry.milestone_type in {"docs_only", "design_only", "closeout"} and not milestone_data.get("agent_closeout_contract"):
            milestone_data["agent_closeout_contract"] = build_agent_closeout_contract_data(entry)
            changed = True
        if entry.milestone_type == "closeout" and not milestone_data.get("handoff_contract"):
            milestone_data["handoff_contract"] = build_handoff_contract_data(entry)
            changed = True
        if changed:
            changed_milestones.append(milestone_id)
    return data, changed_milestones, blockers


def update_manifest_report_after_contract_completion(
    path: Path,
    *,
    completed_milestones: list[str],
    blockers: list[str],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        text = path.read_text(encoding="utf-8")
    else:
        text = "---\ntitle: Track Execution Manifest Report\nstatus: active\n---\n\n# Track Execution Manifest Report\n"
    completed_lines = "\n".join(f"- `{milestone_id}`" for milestone_id in completed_milestones) or "- No contract blocks changed."
    blocker_lines = "\n".join(f"- {blocker}" for blocker in blockers) or "- No remaining contract blockers."
    section = "\n".join(
        [
            "## Track Contract Completion",
            "",
            f"Last contract completion pass: {date.today().isoformat()}.",
            "",
            "This section mirrors machine-readable contract completion. The YAML manifest remains execution authority.",
            "",
            "Completed or refreshed milestone contracts:",
            "",
            completed_lines,
            "",
            "Remaining blockers:",
            "",
            blocker_lines,
        ]
    )
    path.write_text(upsert_markdown_section(text, heading="## Track Contract Completion", section=section, before_heading="## Milestone Details"), encoding="utf-8", newline="\n")


def complete_track_contracts(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    run_validations: bool = True,
) -> ContractCompletionResult:
    manifest_data, completed_milestones, blockers = complete_contracts_for_manifest_data(
        context,
        production_source=production_source,
        roadmap_source=roadmap_source,
    )
    if blockers:
        return ContractCompletionResult(
            track_id=context.track.id,
            manifest_path=context.loaded.path,
            manifest_report_path=REPO_ROOT / manifest_report_path(context.track.id),
            completed_milestones=tuple(completed_milestones),
            validation_commands=(),
            remaining_blockers=tuple(blockers),
        )
    manifest = TrackExecutionManifest.model_validate(manifest_data)
    loaded = LoadedTrackExecutionManifest(manifest=manifest, path=context.loaded.path)
    audit_manifest_or_raise(loaded, track=context.track, roadmap=context.roadmap)
    write_yaml_mapping(context.loaded.path, manifest_data)
    report_path = REPO_ROOT / manifest_report_path(context.track.id)
    update_manifest_report_after_contract_completion(report_path, completed_milestones=completed_milestones, blockers=[])
    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    return ContractCompletionResult(
        track_id=context.track.id,
        manifest_path=context.loaded.path,
        manifest_report_path=report_path,
        completed_milestones=tuple(completed_milestones),
        validation_commands=validation_results,
        remaining_blockers=(),
    )


def slugify(value: str) -> str:
    cleaned = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return cleaned or "track-execution-manifest"


def write_manifest(path: Path, manifest: TrackExecutionManifest) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data = manifest.model_dump(exclude_none=True, mode="json")
    write_yaml_mapping(path, data)


def write_yaml_mapping(path: Path, data: dict, *, indent_sequences: bool = True) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    dumper = IndentedSafeDumper if indent_sequences else yaml.SafeDumper
    path.write_text(
        yaml.dump(data, Dumper=dumper, sort_keys=False, allow_unicode=False, width=4096),
        encoding="utf-8",
        newline="\n",
    )


@dataclass(frozen=True)
class ManifestCommandContext:
    planning: ProductionPlanningState
    roadmap: RoadmapState
    track: ProductionTrack
    loaded: LoadedTrackExecutionManifest


@dataclass(frozen=True)
class AutoSafeExpansionResult:
    track_id: str
    milestone_id: str
    wr_id: str
    manifest_path: Path
    production_source: Path
    roadmap_deferred_source: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class AgentDesignResult:
    track_id: str
    milestone_id: str
    wr_id: str
    plan_path: Path
    design_paths: tuple[Path, ...]
    manifest_path: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str
    agent_transcript_path: Path | None = None


@dataclass(frozen=True)
class AgentCloseoutResult:
    track_id: str
    milestone_id: str
    wr_id: str
    closeout_path: Path
    manifest_path: Path
    production_source: Path
    roadmap_archive_source: Path
    roadmap_deferred_source: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class ProductCodeResult:
    track_id: str
    milestone_id: str
    wr_id: str
    plan_path: Path
    manifest_path: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class ProductImplementationResult:
    track_id: str
    milestone_id: str
    wr_id: str
    plan_path: Path
    manifest_path: Path
    written_paths: tuple[Path, ...]
    validation_commands: tuple[str, ...]
    next_legal_action: str
    agent_transcript_path: Path | None = None


@dataclass(frozen=True)
class StopBeforeProductCodeResult:
    track_id: str
    milestone_id: str
    wr_id: str
    plan_path: Path
    manifest_path: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class StopAtManifestGateResult:
    track_id: str
    milestone_id: str
    manifest_path: Path
    reason: str
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class RuntimeCloseoutResult:
    track_id: str
    milestone_id: str
    wr_id: str
    closeout_path: Path
    manifest_path: Path
    production_source: Path
    roadmap_archive_source: Path
    roadmap_deferred_source: Path
    validation_commands: tuple[str, ...]
    next_legal_action: str


@dataclass(frozen=True)
class ContractCompletionResult:
    track_id: str
    manifest_path: Path
    manifest_report_path: Path
    completed_milestones: tuple[str, ...]
    validation_commands: tuple[str, ...]
    remaining_blockers: tuple[str, ...]


def track_execution_run_path(track_id: str, run_id: str, root: Path = TRACK_EXECUTION_RUN_ROOT) -> Path:
    return root / track_id.lower() / f"{run_id}.yaml"


def new_track_execution_run_id(track_id: str, root: Path = TRACK_EXECUTION_RUN_ROOT) -> str:
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    base = f"{timestamp}-{slugify(track_id)}"
    candidate = base
    index = 1
    while track_execution_run_path(track_id, candidate, root=root).exists():
        index += 1
        candidate = f"{base}-{index}"
    return candidate


def workflow_result_action_kind(result: object) -> str:
    if isinstance(result, AutoSafeExpansionResult):
        return "auto_safe"
    if isinstance(result, AgentDesignResult):
        return "agent_design"
    if isinstance(result, AgentCloseoutResult):
        return "agent_closeout"
    if isinstance(result, ProductCodeResult):
        return "product_code"
    if isinstance(result, ProductImplementationResult):
        return "product_implementation"
    if isinstance(result, RuntimeCloseoutResult):
        return "runtime_closeout"
    if isinstance(result, StopBeforeProductCodeResult):
        return "stop_before_product_code"
    if isinstance(result, StopAtManifestGateResult):
        return "stop_at_manifest_gate"
    return result.__class__.__name__


def workflow_result_changed_files(result: object) -> list[str]:
    paths: list[Path] = []
    if isinstance(result, AutoSafeExpansionResult):
        paths = [result.manifest_path, result.production_source, result.roadmap_deferred_source]
    elif isinstance(result, AgentDesignResult):
        paths = [result.manifest_path, result.plan_path, *result.design_paths]
        if result.agent_transcript_path is not None:
            paths.append(result.agent_transcript_path)
    elif isinstance(result, AgentCloseoutResult):
        paths = [
            result.manifest_path,
            result.production_source,
            result.roadmap_archive_source,
            result.roadmap_deferred_source,
            result.closeout_path,
        ]
    elif isinstance(result, ProductImplementationResult):
        paths = [result.manifest_path, *result.written_paths]
        if result.agent_transcript_path is not None:
            paths.append(result.agent_transcript_path)
    elif isinstance(result, RuntimeCloseoutResult):
        paths = [
            result.manifest_path,
            result.production_source,
            result.roadmap_archive_source,
            result.roadmap_deferred_source,
            result.closeout_path,
        ]
    return [repo_path(path) for path in paths]


def workflow_result_closeout_paths(result: object) -> list[str]:
    if isinstance(result, (AgentCloseoutResult, RuntimeCloseoutResult)):
        return [repo_path(result.closeout_path)]
    if isinstance(result, AgentDesignResult) and result.agent_transcript_path is not None:
        return [repo_path(result.agent_transcript_path)]
    if isinstance(result, ProductImplementationResult) and result.agent_transcript_path is not None:
        return [repo_path(result.agent_transcript_path)]
    return []


def workflow_result_wr_id(result: object) -> str | None:
    value = getattr(result, "wr_id", None)
    return str(value) if value is not None else None


def append_track_execution_run_action(
    *,
    track_id: str,
    run_id: str,
    run_root: Path,
    entry: TrackExecutionManifestMilestone,
    result: object,
    before_digests: dict[str, str],
    after_digests: dict[str, str],
) -> Path:
    path = track_execution_run_path(track_id, run_id, root=run_root)
    if path.exists():
        data = load_yaml(path)
        if not isinstance(data, dict):
            raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    else:
        data = {
            "version": 1,
            "track_id": track_id,
            "run_id": run_id,
            "started_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
            "actions": [],
        }
    strategy = entry.closeout_strategy or entry.execution_kind or entry.milestone_type
    if isinstance(result, ProductImplementationResult) and entry.implementation_writer is not None:
        strategy = entry.implementation_writer.strategy
    data.setdefault("actions", []).append(
        {
            "action_index": len(data.get("actions", [])) + 1,
            "completed_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
            "action_kind": workflow_result_action_kind(result),
            "milestone_id": entry.milestone_id,
            "wr_id": workflow_result_wr_id(result),
            "strategy": strategy,
            "pre_action_digests": before_digests,
            "post_action_digests": after_digests,
            "files_changed": workflow_result_changed_files(result),
            "validation_results": list(getattr(result, "validation_commands", ())),
            "evidence_paths": workflow_result_closeout_paths(result),
            "closeout_paths": workflow_result_closeout_paths(result),
            "next_legal_action": getattr(result, "next_legal_action", ""),
            "stop_reason": getattr(result, "next_legal_action", "") if workflow_result_action_kind(result).startswith("stop_") else "",
        }
    )
    data["last_action_at"] = datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    data["last_next_legal_action"] = getattr(result, "next_legal_action", "")
    write_yaml_mapping(path, data)
    return path


def resolve_manifest_command_context(
    track_id: str,
    *,
    production_source: Path,
    roadmap_source: Path,
    manifest_source_root: Path,
) -> ManifestCommandContext:
    planning = load_production_tracks(production_source)
    roadmap = load_roadmap(roadmap_source)
    track = find_track(planning, track_id)
    loaded = load_track_execution_manifest(track_id, root=manifest_source_root)
    if loaded is None:
        raise WorkflowError(
            f"{track_id}: no Track Execution Manifest source at {repo_path(manifest_source_path(track_id, root=manifest_source_root))}"
        )
    return ManifestCommandContext(planning=planning, roadmap=roadmap, track=track, loaded=loaded)


def allocate_next_wr_id(roadmap: RoadmapState) -> str:
    existing_numbers = [int(item.id.split("-")[1]) for item in roadmap.items if ROADMAP_ID_PATTERN.fullmatch(item.id)]
    if not existing_numbers:
        return "WR-001"
    return f"WR-{max(existing_numbers) + 1:03d}"


def manifest_report_path(track_id: str) -> str:
    return f"docs-site/src/content/docs/reports/track-execution-manifests/{track_id.lower()}/manifest.md"


def production_generated_scopes() -> list[str]:
    return [
        "generated: production docs from task production:render",
        "generated: roadmap docs from task roadmap:render",
    ]


def implementation_plan_path(wr_id: str, milestone: ProductionMilestone) -> str:
    return (
        "docs-site/src/content/docs/reports/implementation-plans/"
        f"{wr_id.lower()}-{slugify(milestone.title)}/plan.md"
    )


def exact_auto_safe_write_scope(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track_id: str,
    wr_id: str,
    production_source: Path | None = None,
    manifest_source: Path | None = None,
    roadmap_source: Path | None = None,
    deferred_source: Path | None = None,
) -> list[str]:
    scopes = [
        "docs-site/src/content/docs/workspace/production-tracks.yaml",
        "docs-site/src/content/docs/workspace/roadmap-archive.yaml",
        "docs-site/src/content/docs/workspace/roadmap-deferred.yaml",
        f"docs-site/src/content/docs/workspace/track-execution-manifests/{track_id.lower()}.yaml",
        manifest_report_path(track_id),
        implementation_plan_path(wr_id, milestone),
        entry.expected_closeout_path,
        *production_generated_scopes(),
    ]
    if entry.milestone_type in {"implementation", "hardening"}:
        scopes.append("docs-site/src/content/docs/workspace/roadmap-items.yaml")
        scopes.extend(manifest_write_scopes_for_entry(entry))
    if production_source is not None:
        scopes.append(repo_path(production_source))
    if manifest_source is not None:
        scopes.append(repo_path(manifest_source))
    if roadmap_source is not None:
        scopes.append(repo_path(roadmap_source))
    if deferred_source is not None:
        scopes.append(repo_path(deferred_source))
    return list(dict.fromkeys(scopes))


def predecessor_wr_dependencies(
    entry: TrackExecutionManifestMilestone,
    track: ProductionTrack,
) -> list[str]:
    by_milestone_id = {milestone.id: milestone for milestone in track.milestones}
    dependencies: list[str] = []
    for dependency in entry.predecessor_dependencies:
        dependency_milestone = by_milestone_id.get(dependency)
        if dependency_milestone is None:
            continue
        dependencies.extend(dependency_milestone.roadmap_links)
    return list(dict.fromkeys(dependencies))


def decision_gates_for_milestone(milestone: ProductionMilestone) -> list[dict]:
    return [
        {
            "kind": gate.kind,
            "path": gate.path,
            "required_status": gate.required_status,
            "applies_to": "discovery",
            "reason": gate.reason,
        }
        for gate in milestone.design_gates
    ]


def roadmap_row_for_auto_safe_expansion(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    track: ProductionTrack,
    wr_id: str,
    production_source: Path | None = None,
    manifest_source: Path | None = None,
    deferred_source: Path | None = None,
    roadmap_source: Path | None = None,
) -> dict:
    write_scope = exact_auto_safe_write_scope(
        entry,
        milestone,
        track_id=track.id,
        wr_id=wr_id,
        production_source=production_source,
        manifest_source=manifest_source,
        roadmap_source=roadmap_source,
        deferred_source=deferred_source,
    )
    return {
        "id": wr_id,
        "title": milestone.title,
        "diagram_title": milestone.title[:48],
        "alias": wr_id.replace("-", ""),
        "lane": "Product planning",
        "dependency_level": "L4",
        "gate": (
            "Closeout Track Expansion gate"
            if entry.milestone_type == "closeout"
            else "Design-only Track Expansion gate"
            if entry.milestone_type in {"docs_only", "design_only"}
            else "Implementation plan Track Expansion gate"
        ),
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
        "dependencies": predecessor_wr_dependencies(entry, track),
        "write_scopes": write_scope,
        "validations": entry.validation_commands,
        "next_evidence": (
            f"{entry.milestone_id} requires a dedicated production plan and closeout evidence before completion."
        ),
        "current_decision": (
            f"Auto-safe Track Expansion created this deferred WR from {track.id} manifest data only; "
            "it authorizes planning metadata, not product code."
        ),
        "current_call": (
            f"Run task production:run-track -- --track {track.id} --allow agent_closeout --max-actions 1."
            if entry.milestone_type == "closeout"
            else (
                f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
                "stop before product code until the bounded contract is accepted."
            )
        ),
        "first_move": (
            f"Run task production:run-track -- --track {track.id} --allow agent_closeout --max-actions 1."
            if entry.milestone_type == "closeout"
            else f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}."
        ),
        "main_blocker": (
            "Handoff closeout evidence is still missing."
            if entry.milestone_type == "closeout"
            else "Dedicated production plan and closeout evidence are still missing."
        ),
        "why_not_ready": "Track Expansion linked WR authority mechanically; the milestone work itself has not started.",
        "completion_quality": "not_applicable",
        "known_quality_gaps": [],
        "completion_audit": "",
        "diagram_call": ["track expansion", "no implementation"],
        "decision_gates": decision_gates_for_milestone(milestone),
        "ddd_owner": "domain/ui owns UiProgram contract design; workspace governance coordinates production sequencing.",
        "adr_requirement": "No new ADR unless the UiProgram contract changes accepted ownership, dependency direction, or extraction gates.",
        "fitness_function_requirement": "Docs, production, roadmap, and planning validation before closeout.",
        "ownership_mode": "Stream-aligned UI proving-domain planning with governance-owned sequencing.",
    }


def assert_auto_safe_expansion_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    roadmap: RoadmapState,
    allow: set[str],
) -> None:
    unsupported = sorted(allow - {"auto_safe"})
    if unsupported:
        raise WorkflowError(f"Manifest Runner V1 does not support permissions: {', '.join(unsupported)}")
    if "auto_safe" not in allow:
        raise WorkflowError("Manifest Runner requires --allow auto_safe for mechanical Track Expansion")
    if milestone.state == "completed":
        raise WorkflowError(f"{entry.milestone_id}: completed milestones must not be mutated by Track Expansion")
    if milestone.roadmap_links:
        raise WorkflowError(f"{entry.milestone_id}: production milestone already links WR rows {milestone.roadmap_links}")
    if not entry.future_wr_candidate:
        raise WorkflowError(f"{entry.milestone_id}: no future WR candidate is available for Track Expansion")
    if entry.milestone_type not in {"docs_only", "design_only", "implementation", "hardening", "closeout"}:
        raise WorkflowError(
            f"{entry.milestone_id}: auto_safe expansion supports docs, design, implementation, hardening, or closeout milestones only"
        )
    if roadmap.by_id.get(entry.future_wr_candidate):
        raise WorkflowError(f"{entry.future_wr_candidate}: future WR candidate unexpectedly exists as a concrete WR")


def updated_production_data_with_wr(
    production_source: Path,
    *,
    track_id: str,
    milestone_id: str,
    wr_id: str,
) -> dict:
    data = load_yaml(production_source)
    changed = False
    for track_data in data.get("tracks", []):
        if track_data.get("id") != track_id:
            continue
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") == milestone_id:
                milestone_data["roadmap_links"] = [wr_id]
                changed = True
                break
    if not changed:
        raise WorkflowError(f"{milestone_id}: not found in production source {repo_path(production_source)}")
    return data


def updated_manifest_data_with_wr(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    wr_id: str,
    production_source: Path | None = None,
    roadmap_source: Path | None = None,
    deferred_source: Path | None = None,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    changed = False
    for milestone_data in data["milestones"]:
        if milestone_data["milestone_id"] != entry.milestone_id:
            continue
        milestone_data.pop("future_wr_candidate", None)
        milestone_data["owning_wr"] = wr_id
        if entry.milestone_type in {"docs_only", "design_only", "closeout"}:
            milestone_data["write_scope"] = exact_auto_safe_write_scope(
                entry,
                milestone,
                track_id=loaded.manifest.track_id,
                wr_id=wr_id,
                production_source=production_source,
                manifest_source=loaded.path,
                roadmap_source=roadmap_source,
                deferred_source=deferred_source,
            )
        if entry.milestone_type in {"implementation", "hardening"} and (
            milestone_data.get("agent_design") or milestone_data.get("agent_design_contract")
        ):
            planning_scope = exact_auto_safe_write_scope(
                entry,
                milestone,
                track_id=loaded.manifest.track_id,
                wr_id=wr_id,
                production_source=production_source,
                manifest_source=loaded.path,
                roadmap_source=roadmap_source,
                deferred_source=deferred_source,
            )
            if milestone_data.get("agent_design"):
                milestone_data["agent_design"]["planning_write_scope"] = planning_scope
            if milestone_data.get("agent_design_contract"):
                milestone_data["agent_design_contract"]["planning_write_scope"] = planning_scope
        if entry.milestone_type == "closeout":
            milestone_data["next_legal_action"] = (
                f"Run task production:run-track -- --track {loaded.manifest.track_id} --allow agent_closeout --max-actions 1 "
                f"to close {entry.milestone_id}; do not start downstream implementation."
            )
        else:
            milestone_data["next_legal_action"] = (
                f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
                "stop before product code until that contract is accepted."
            )
        changed = True
        break
    if not changed:
        raise WorkflowError(f"{entry.milestone_id}: not found in manifest {repo_path(loaded.path)}")
    if entry.milestone_type == "closeout":
        data["next_legal_action"] = (
            f"Run task production:run-track -- --track {loaded.manifest.track_id} --allow agent_closeout --max-actions 1 "
            f"to close {entry.milestone_id}; do not start downstream implementation."
        )
    else:
        data["next_legal_action"] = (
            f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
            "do not run product_code until the bounded plan is accepted."
        )
    return data


def updated_deferred_roadmap_data(
    roadmap_source: Path,
    *,
    roadmap: RoadmapState,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    track: ProductionTrack,
    wr_id: str,
    production_source: Path,
    manifest_source: Path,
) -> tuple[Path, dict]:
    active_data = load_yaml(roadmap_source)
    _, deferred_source = split_source_paths(roadmap_source)
    if deferred_source.exists():
        deferred_data = load_yaml(deferred_source)
    else:
        deferred_data = empty_split_source_like(active_data)
    if any(item.get("id") == wr_id for item in deferred_data.get("items", [])):
        raise WorkflowError(f"{wr_id}: already present in deferred roadmap source")
    if roadmap.by_id.get(wr_id):
        raise WorkflowError(f"{wr_id}: already present in combined roadmap state")
    deferred_data.setdefault("items", []).append(
        roadmap_row_for_auto_safe_expansion(
            entry,
            milestone,
            track=track,
            wr_id=wr_id,
            production_source=production_source,
            manifest_source=manifest_source,
            roadmap_source=roadmap_source,
            deferred_source=deferred_source,
        )
    )
    return deferred_source, deferred_data


def empty_split_source_like(active_data: dict) -> dict:
    return {
        "version": active_data.get("version", 1),
        "roadmap": active_data.get("roadmap", {}),
        "items": [],
    }


def validate_auto_safe_expansion_data(
    *,
    production_data: dict,
    manifest_data: dict,
    active_roadmap_data: dict,
    archive_roadmap_data: dict | None,
    deferred_roadmap_data: dict,
    production_source: Path,
    roadmap_source: Path,
    manifest_path: Path,
    track_id: str,
) -> tuple[ProductionPlanningState, RoadmapState, LoadedTrackExecutionManifest]:
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            active_roadmap_data,
            roadmap_source,
            archive_data=archive_roadmap_data,
            deferred_data=deferred_roadmap_data,
        )
    )
    manifest = TrackExecutionManifest.model_validate(manifest_data)
    loaded = LoadedTrackExecutionManifest(manifest=manifest, path=manifest_path)
    track = find_track(planning, track_id)
    audit_manifest_or_raise(loaded, track=track, roadmap=roadmap)
    return planning, roadmap, loaded


def run_validation_commands(commands: list[str], *, cwd: Path = REPO_ROOT) -> tuple[str, ...]:
    outputs: list[str] = []
    for command in commands:
        completed = subprocess.run(command, cwd=cwd, shell=True, text=True, capture_output=True)
        combined = "\n".join(part for part in (completed.stdout.strip(), completed.stderr.strip()) if part)
        outputs.append(f"{command}: exit {completed.returncode}")
        if completed.returncode != 0:
            detail = f"\n{combined}" if combined else ""
            raise WorkflowError(f"validation command failed: {command}{detail}")
    return tuple(outputs)


def auto_safe_validation_commands() -> list[str]:
    return [
        "task production:render",
        "task roadmap:render",
        "task production:validate",
        "task roadmap:validate",
        "task production:check",
        "task roadmap:check",
        "task docs:validate",
        "task planning:validate",
    ]


def apply_auto_safe_track_expansion(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    run_validations: bool = True,
) -> AutoSafeExpansionResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if workflow_action != "track_expansion_required":
        raise WorkflowError(
            f"{entry.milestone_id}: next legal action is {workflow_action}, not auto_safe Track Expansion"
        )
    dependency_blockers = [blocker for blocker in blockers if "Track Expansion must create or link" not in blocker]
    if dependency_blockers:
        raise WorkflowError("\n".join(dependency_blockers))
    assert_auto_safe_expansion_allowed(entry, milestone, roadmap=context.roadmap, allow=allow)

    wr_id = allocate_next_wr_id(context.roadmap)
    production_data = updated_production_data_with_wr(
        production_source,
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=wr_id,
    )
    deferred_source, deferred_data = updated_deferred_roadmap_data(
        roadmap_source,
        roadmap=context.roadmap,
        entry=entry,
        milestone=milestone,
        track=context.track,
        wr_id=wr_id,
        production_source=production_source,
        manifest_source=context.loaded.path,
    )
    manifest_data = updated_manifest_data_with_wr(
        context.loaded,
        entry=entry,
        milestone=milestone,
        wr_id=wr_id,
        production_source=production_source,
        roadmap_source=roadmap_source,
        deferred_source=deferred_source,
    )
    active_data = load_yaml(roadmap_source)
    archive_source, _ = split_source_paths(roadmap_source)
    archive_data = load_yaml(archive_source) if archive_source.exists() else None
    validate_auto_safe_expansion_data(
        production_data=production_data,
        manifest_data=manifest_data,
        active_roadmap_data=active_data,
        archive_roadmap_data=archive_data,
        deferred_roadmap_data=deferred_data,
        production_source=production_source,
        roadmap_source=roadmap_source,
        manifest_path=context.loaded.path,
        track_id=context.track.id,
    )

    write_yaml_mapping(production_source, production_data)
    write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)
    write_yaml_mapping(context.loaded.path, manifest_data)
    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    next_legal_action = (
        f"Run task production:plan -- --milestone {entry.milestone_id} --roadmap {wr_id}; "
        "do not start design authoring until that plan is accepted."
    )
    return AutoSafeExpansionResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=wr_id,
        manifest_path=context.loaded.path,
        production_source=production_source,
        roadmap_deferred_source=deferred_source,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
    )


def manifest_write_scopes_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    scopes = product_implementation_scopes_for_entry(entry) if entry.milestone_type in {"implementation", "hardening"} else entry.write_scope
    return [
        normalized
        for normalized in (manifest_write_scope_path(scope) for scope in scopes)
        if normalized is not None
    ]


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


def runtime_closeout_validation_commands_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.runtime_closeout_contract is not None:
        return list(entry.runtime_closeout_contract.validation_commands)
    return product_validation_commands_for_entry(entry)


NON_EXECUTABLE_VALIDATION_MARKERS = (
    "focused tests named",
    "named by the owning production plan",
    "run relevant tests",
    "relevant tests",
    "tbd",
    "to be decided",
    "validate manually",
    "manual validation",
)

EXECUTABLE_VALIDATION_PREFIXES = (
    "cargo ",
    "task ",
    "uv run ",
    "python ",
    "pytest",
    "npm ",
    "pnpm ",
    "bun ",
    "npx ",
    "git ",
)


def validation_command_errors(
    commands: list[str],
    *,
    label: str,
    product_code_eligible: bool,
) -> list[str]:
    errors: list[str] = []
    for command in commands:
        normalized = command.strip().lower()
        if not normalized:
            errors.append(f"{label}: validation command must not be empty")
            continue
        if any(marker in normalized for marker in NON_EXECUTABLE_VALIDATION_MARKERS):
            errors.append(f"{label}: validation command is a non-executable placeholder: {command}")
            continue
        if product_code_eligible and not normalized.startswith(EXECUTABLE_VALIDATION_PREFIXES):
            errors.append(f"{label}: product_code validation command is not executable: {command}")
    return errors


def path_is_covered_by_scope(path: str, scopes: list[str]) -> bool:
    normalized = normalize_repo_path(path)
    return any(path_within_scope(normalized, scope) for scope in scopes)


def assert_agent_design_write_scope(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    contract: ManifestAgentDesignContract,
    write_paths: list[str],
) -> None:
    if entry.milestone_type in {"implementation", "hardening"}:
        manifest_scopes = [
            normalized
            for normalized in (manifest_write_scope_path(scope) for scope in contract.planning_write_scope)
            if normalized is not None
        ]
        wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
        missing_manifest = [path for path in write_paths if not path_is_covered_by_scope(path, manifest_scopes)]
        missing_wr = [path for path in write_paths if not path_is_covered_by_scope(path, wr_scopes)]
        if missing_manifest:
            raise WorkflowError(
                f"{entry.milestone_id}: agent_design write paths are not covered by agent_design planning_write_scope: "
                + ", ".join(missing_manifest)
            )
        if missing_wr:
            raise WorkflowError(
                f"{entry.milestone_id}: agent_design write paths are not covered by owning WR {roadmap_item.id} write_scopes: "
                + ", ".join(missing_wr)
            )
        return
    assert_runner_write_scope(
        entry=entry,
        roadmap_item=roadmap_item,
        write_paths=write_paths,
        action_label="agent_design",
    )


def assert_runner_write_scope(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    write_paths: list[str],
    action_label: str,
) -> None:
    manifest_scopes = manifest_write_scopes_for_entry(entry)
    wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
    missing_manifest = [path for path in write_paths if not path_is_covered_by_scope(path, manifest_scopes)]
    missing_wr = [path for path in write_paths if not path_is_covered_by_scope(path, wr_scopes)]
    if missing_manifest:
        raise WorkflowError(
            f"{entry.milestone_id}: {action_label} write paths are not covered by manifest write_scope: "
            + ", ".join(missing_manifest)
        )
    if missing_wr:
        raise WorkflowError(
            f"{entry.milestone_id}: {action_label} write paths are not covered by owning WR {roadmap_item.id} write_scopes: "
            + ", ".join(missing_wr)
        )


PRODUCT_CODE_REQUIRED_PLAN_MARKERS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("exact files/modules allowed", ("expected implementation files", "files/modules expected to change", "exact files/modules")),
    ("exact methods/functions", ("expected methods/functions", "methods/functions", "functions/methods")),
    ("files/modules forbidden", ("forbidden", "non-goals", "non-goals")),
    ("tests to add/change", ("tests to add/change", "focused tests", "tests")),
    ("validation commands", ("## validation", "validation commands")),
    ("closeout evidence", ("## closeout requirements", "closeout evidence")),
    ("compatibility/rollback plan", ("rollback", "compatibility")),
    ("stop conditions", ("## stop conditions", "stop conditions")),
)


STALE_SLICE_TERMS_BY_CURRENT_SLICE: dict[str, tuple[str, ...]] = {
    "6B": (
        "label text output",
        "font/style intent",
        "text layout request",
        "structural uiframe text",
        "6a label",
        "6a implementation",
        "6a closeout",
    ),
    "6C": (
        "button route/event/host-command",
        "6b button",
        "6b implementation",
        "colorpicker",
        "6d colorpicker",
    ),
    "6D": (
        "inspectorfield binding",
        "6c inspectorfield",
        "world-space prompt",
        "6e world",
    ),
}


def implementation_plan_consistency_errors_from_text(
    entry: TrackExecutionManifestMilestone,
    *,
    roadmap_item: RoadmapItem,
    text: str,
    plan_path: Path,
) -> list[str]:
    errors: list[str] = []
    normalized = text.lower()
    proof_slice_id = proof_slice_id_for_entry(entry)
    proof_slice_title = proof_slice_title_for_entry(entry)
    target_surface = target_control_surface_for_entry(entry)
    implementation_proof_kind = entry.implementation_proof_kind or proof_slice_id.lower()
    required_terms = [
        entry.milestone_id,
        roadmap_item.id,
        entry.title,
        proof_slice_id,
        proof_slice_title,
        target_surface,
        implementation_proof_kind,
    ]
    for term in required_terms:
        if term and term.lower() not in normalized:
            errors.append(f"{entry.milestone_id}: generated implementation plan is missing current proof term {term!r}")
    for scope in product_implementation_scopes_for_entry(entry):
        scope_path = manifest_write_scope_path(scope)
        scope_terms = [scope.lower()]
        if scope_path is not None:
            scope_terms.append(scope_path.lower())
        if not any(term in normalized for term in scope_terms):
            errors.append(f"{entry.milestone_id}: generated implementation plan is missing exact write scope {scope}")
    for command in implementation_plan_validation_commands(entry):
        if command.lower() not in normalized:
            errors.append(f"{entry.milestone_id}: generated implementation plan is missing validation command {command!r}")
    errors.extend(
        validation_command_errors(
            implementation_plan_validation_commands(entry),
            label=f"{entry.milestone_id}: product_code plan",
            product_code_eligible=True,
        )
    )
    for stale_term in STALE_SLICE_TERMS_BY_CURRENT_SLICE.get(proof_slice_id.upper(), ()):
        if stale_term in normalized:
            errors.append(
                f"{entry.milestone_id}: generated implementation plan contains stale slice term {stale_term!r}"
            )
    if f"stop before {entry.milestone_id.lower()}" in normalized:
        errors.append(f"{entry.milestone_id}: generated implementation plan says to stop before the current milestone")
    current_slice_stop = f"stop before {proof_slice_id.lower()}"
    if current_slice_stop in normalized:
        errors.append(f"{entry.milestone_id}: generated implementation plan says to stop before the current proof slice")
    if entry.expected_closeout_path.lower() not in normalized:
        errors.append(
            f"{entry.milestone_id}: generated implementation plan is missing current closeout path {entry.expected_closeout_path}"
        )
    if plan_path.suffix.lower() != ".md":
        errors.append(f"{entry.milestone_id}: generated implementation plan path must be markdown: {repo_path(plan_path)}")
    return errors


def exact_product_write_scope_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors = exact_scope_list_errors(
        product_implementation_scopes_for_entry(entry),
        label=f"{entry.milestone_id}: product_code write_scope",
    )
    for scope in product_implementation_scopes_for_entry(entry):
        normalized = manifest_write_scope_path(scope)
        if normalized is not None and normalized.endswith("/"):
            errors.append(f"{entry.milestone_id}: product_code write_scope must be an exact file or module path: {scope}")
    return errors


def crate_creation_scope_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    crate_scopes = [
        scope
        for scope in product_implementation_scopes_for_entry(entry)
        if is_new_write_scope(scope) and normalize_write_scope_path(scope).endswith("Cargo.toml")
    ]
    if not crate_scopes:
        errors.append(
            f"{entry.milestone_id}: crate_creation requires exact 'new:' Cargo.toml scope; "
            "add at least one exact new: <crate>/Cargo.toml write scope"
        )
    for scope in product_implementation_scopes_for_entry(entry):
        if "foundation/meta" in scope.lower():
            errors.append(f"{entry.milestone_id}: crate_creation cannot target foundation/meta")
    return errors


RUNTIME_EVIDENCE_COMMAND_PREFIXES = (
    "cargo test",
    "cargo nextest",
    "cargo llvm-cov",
    "uv run pytest",
    "pytest",
    "task test",
)


def command_is_runtime_evidence(command: str) -> bool:
    normalized = command.strip().lower()
    return any(normalized.startswith(prefix) for prefix in RUNTIME_EVIDENCE_COMMAND_PREFIXES)


def existing_scope_path(normalized: str) -> Path | None:
    repo_candidate = REPO_ROOT / normalized
    if repo_candidate.exists():
        return repo_candidate
    absolute_candidate = Path("/" + normalized)
    if absolute_candidate.exists():
        return absolute_candidate
    return None


def runtime_evidence_errors(entry: TrackExecutionManifestMilestone) -> list[str]:
    errors: list[str] = []
    commands = runtime_closeout_validation_commands_for_entry(entry)
    if not any(command_is_runtime_evidence(command) for command in commands):
        errors.append(
            f"{entry.milestone_id}: runtime closeout requires at least one runtime/test validation command"
        )
    errors.extend(
        validation_command_errors(
            commands,
            label=f"{entry.milestone_id}: runtime_closeout",
            product_code_eligible=True,
        )
    )
    missing_paths = []
    expected_closeout = normalize_repo_path(entry.expected_closeout_path)
    for scope in product_implementation_scopes_for_entry(entry):
        if is_generated_or_derived_scope(scope):
            continue
        normalized = manifest_write_scope_path(scope)
        if normalized is None:
            continue
        if normalized == expected_closeout:
            continue
        if existing_scope_path(normalized) is None:
            missing_paths.append(normalized)
    if missing_paths:
        errors.append(
            f"{entry.milestone_id}: runtime closeout requires scoped runtime evidence paths to exist: "
            + ", ".join(missing_paths)
        )
    return errors


def product_plan_contract_errors(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    plan_path: Path,
) -> list[str]:
    errors: list[str] = []
    if not plan_path.exists():
        return [f"{entry.milestone_id}: accepted production plan is missing: {repo_path(plan_path)}"]
    status = document_frontmatter_status(plan_path)
    if status is None:
        errors.append(f"{entry.milestone_id}: accepted production plan has no frontmatter status: {repo_path(plan_path)}")
    elif status.lower() not in {"active", "accepted", "completed"}:
        errors.append(
            f"{entry.milestone_id}: accepted production plan status {status!r} is not active, accepted, or completed"
        )
    plan_text = plan_path.read_text(encoding="utf-8").lower()
    for label, markers in PRODUCT_CODE_REQUIRED_PLAN_MARKERS:
        if not any(marker in plan_text for marker in markers):
            errors.append(f"{entry.milestone_id}: accepted production plan is missing {label}")
    for scope in product_implementation_scopes_for_entry(entry):
        normalized = manifest_write_scope_path(scope)
        if normalized is None or is_generated_or_derived_scope(scope):
            continue
        if normalized.lower() not in plan_text:
            errors.append(
                f"{entry.milestone_id}: accepted production plan does not name product_code write_scope {normalized}"
            )
    errors.extend(
        implementation_plan_consistency_errors_from_text(
            entry,
            roadmap_item=roadmap_item,
            text=plan_path.read_text(encoding="utf-8"),
            plan_path=plan_path,
        )
    )
    if roadmap_item.completion_quality == "runtime_proven":
        errors.append(
            f"{entry.milestone_id}: product_code cannot claim runtime_proven before closeout runtime/test evidence exists"
        )
    return errors


def assert_product_code_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    roadmap_item: RoadmapItem,
    *,
    plan_path: Path,
    allow: set[str],
) -> None:
    errors: list[str] = []
    if "product_code" not in allow:
        errors.append("Manifest Runner V4 requires --allow product_code for product/runtime code automation")
    if "foundation_extraction" in allow:
        errors.append("foundation_extraction automation is not implemented and remains blocked")
    if entry.may_create_crates and "crate_creation" not in allow:
        errors.append(f"{entry.milestone_id}: crate_creation is required but --allow crate_creation was not granted")
    if entry.milestone_type not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: product_code supports implementation/runtime-proof milestones only")
    if milestone.kind not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: production milestone kind {milestone.kind!r} cannot run product_code")
    if not entry.may_create_code:
        errors.append(f"{entry.milestone_id}: manifest does not authorize code creation")
    if not entry.may_modify_production_behavior:
        errors.append(f"{entry.milestone_id}: manifest does not authorize production behavior changes")
    if not entry.owning_wr:
        errors.append(f"{entry.milestone_id}: product_code requires an active owning WR")
    if entry.future_wr_candidate:
        errors.append(f"{entry.milestone_id}: product_code requires a concrete active WR, not a future WR candidate")
    if roadmap_item.planning_state != "current_candidate":
        errors.append(
            f"{entry.milestone_id}: product_code requires active current_candidate WR; "
            f"{roadmap_item.id} is {roadmap_item.planning_state}"
        )
    if roadmap_item.blocker > 2:
        errors.append(f"{entry.milestone_id}: product_code requires B2 or lower implementation blocker; {roadmap_item.id} is {roadmap_item.blocker_label}")
    if not entry.forbidden_scope:
        errors.append(f"{entry.milestone_id}: product_code requires explicit forbidden scope")
    if not product_validation_commands_for_entry(entry):
        errors.append(f"{entry.milestone_id}: product_code requires validation commands")
    if not entry.expected_closeout_path or entry.expected_closeout_path.startswith("blocked:"):
        errors.append(f"{entry.milestone_id}: product_code requires an expected closeout path")
    if any("foundation/meta" in scope.lower() for scope in product_implementation_scopes_for_entry(entry)):
        errors.append(f"{entry.milestone_id}: product_code cannot authorize shared foundation/meta extraction")
    errors.extend(exact_product_write_scope_errors(entry))
    errors.extend(product_plan_contract_errors(entry=entry, roadmap_item=roadmap_item, plan_path=plan_path))
    if entry.may_create_crates:
        errors.extend(crate_creation_scope_errors(entry))
    if errors:
        raise WorkflowError("\n".join(errors))


def apply_product_code(
    context: ManifestCommandContext,
    *,
    allow: set[str],
    run_validations: bool = True,
) -> ProductCodeResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if blockers:
        raise WorkflowError("\n".join(blockers))
    if workflow_action != "write_implementation_contract":
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not product_code")
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
    plan_path = default_contract_path(roadmap_item)
    assert_product_code_allowed(entry, milestone, roadmap_item, plan_path=plan_path, allow=allow)
    validation_results = run_validation_commands(product_validation_commands_for_entry(entry)) if run_validations else ()
    return ProductCodeResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        plan_path=plan_path,
        manifest_path=context.loaded.path,
        validation_commands=validation_results,
        next_legal_action=(
            f"{entry.milestone_id} product_code gate passed for {entry.owning_wr}; "
            "stop after this implementation WR and close out with runtime/test evidence before continuing."
        ),
    )


def optional_file_digest(path: Path) -> str | None:
    return sha256_file(path) if path.exists() else None


def safe_action_relative_path(path: Path) -> Path:
    return Path(normalize_repo_path(repo_path(path)))


def action_workspace_path(workspace: Path, original: Path) -> Path:
    return workspace / safe_action_relative_path(original)


def action_tree_digests(workspace: Path) -> dict[str, str]:
    digests: dict[str, str] = {}
    if not workspace.exists():
        return digests
    for path in sorted(candidate for candidate in workspace.rglob("*") if candidate.is_file()):
        if ".git" in path.parts:
            continue
        relative = slash_path(path.relative_to(workspace))
        digests[relative] = sha256(path.read_bytes()).hexdigest()
    return digests


def slash_path(path: Path) -> str:
    return str(path).replace("\\", "/")


def changed_action_workspace_files(before: dict[str, str], after: dict[str, str]) -> list[str]:
    return sorted(path for path, digest in after.items() if before.get(path) != digest)


def copy_agent_workspace_input(workspace: Path, original: Path) -> None:
    target = action_workspace_path(workspace, original)
    target.parent.mkdir(parents=True, exist_ok=True)
    if original.exists():
        shutil.copy2(original, target)


def writer_original_path(scope: str) -> Path | None:
    if is_generated_or_derived_scope(scope):
        return None
    normalized = normalize_write_scope_path(scope)
    if not normalized:
        return None
    raw = Path(normalized)
    if raw.is_absolute():
        return raw
    absolute_candidate = Path("/" + normalized)
    if absolute_candidate.exists():
        return absolute_candidate
    return REPO_ROOT / normalized


def existing_context_path(scope: str) -> Path | None:
    original = writer_original_path(scope)
    if original is None:
        return None
    if original.exists() or is_new_write_scope(scope):
        return original
    return None


def run_codex_agent(workspace: Path, prompt: str) -> subprocess.CompletedProcess[str]:
    codex_bin = os.environ.get("MANIFEST_RUNNER_CODEX_BIN", "codex")
    return subprocess.run(
        [
            codex_bin,
            "exec",
            "--ephemeral",
            "--sandbox",
            "workspace-write",
            "-C",
            str(workspace),
            "-",
        ],
        input=prompt,
        text=True,
        capture_output=True,
        check=False,
    )


def agent_writer_prompt(
    *,
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
    plan_path: Path,
    allowed_map: dict[str, Path],
) -> str:
    allowed_lines = "\n".join(f"- `{relative}` -> `{repo_path(original)}`" for relative, original in sorted(allowed_map.items()))
    forbidden_lines = "\n".join(f"- {scope}" for scope in implementation_writer_forbidden_scopes(writer) + list(entry.forbidden_scope))
    validation_lines = "\n".join(f"- `{command}`" for command in writer.validation_commands)
    required_output_lines = "\n".join(f"- {output}" for output in [*writer.required_outputs, *writer.agent_required_outputs])
    extra_prompt = writer.agent_prompt or "Implement the bounded product/runtime change required by the active WR plan."
    return "\n".join(
        [
            "You are running inside an isolated Manifest Runner action workspace.",
            "Modify only the allowed files listed below. Do not create folders or files outside those paths.",
            "",
            f"Milestone: {entry.milestone_id} - {entry.title}",
            f"Accepted implementation plan: {repo_path(plan_path)}",
            "",
            "Allowed output files:",
            allowed_lines or "- none",
            "",
            "Required outputs:",
            required_output_lines or "- Bounded implementation evidence named by the active plan.",
            "",
            "Forbidden scopes and patterns:",
            forbidden_lines or "- none",
            "",
            "Validation commands expected after import:",
            validation_lines or "- none",
            "",
            "Task:",
            extra_prompt,
            "",
            "Return normally after editing files. Do not run destructive git commands.",
        ]
    )


def agent_transcript_path_for(
    *,
    track_id: str,
    run_id: str,
    entry: TrackExecutionManifestMilestone,
    run_root: Path,
    action_kind: str = "agent-writer",
) -> Path | None:
    if not run_id:
        return None
    return run_root / track_id.lower() / f"{run_id}-{entry.milestone_id.lower()}-{action_kind}.txt"


@dataclass(frozen=True)
class AgentWriterResult:
    files: dict[Path, str]
    transcript_path: Path | None


def write_agent_transcript(
    path: Path | None,
    *,
    workspace: Path,
    prompt: str,
    completed: subprocess.CompletedProcess[str],
    changed_files: list[str],
) -> None:
    if path is None:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    content = "\n".join(
        [
            "---",
            "title: Manifest Agent Writer Transcript",
            "status: completed",
            f"last_reviewed: {date.today().isoformat()}",
            "---",
            "",
            "# Manifest Agent Writer Transcript",
            "",
            f"- Action workspace: `{workspace}`",
            f"- Exit code: `{completed.returncode}`",
            "",
            "## Changed Files",
            "",
            *[f"- `{path}`" for path in changed_files],
            "",
            "## Prompt",
            "",
            "```text",
            prompt,
            "```",
            "",
            "## Stdout",
            "",
            "```text",
            completed.stdout.strip(),
            "```",
            "",
            "## Stderr",
            "",
            "```text",
            completed.stderr.strip(),
            "```",
            "",
        ]
    )
    path.write_text(content, encoding="utf-8", newline="\n")


def agent_writer_files(
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
    *,
    plan_path: Path,
    manifest_path: Path,
    track_id: str,
    run_id: str,
    run_ledger_root: Path,
) -> AgentWriterResult:
    if writer.agent_worktree_policy != "isolated_action_workspace":
        raise WorkflowError(f"{entry.milestone_id}: unsupported agent_writer workspace policy {writer.agent_worktree_policy}")
    allowed_originals = [
        original
        for original in (writer_original_path(scope) for scope in implementation_writer_allowed_scopes(writer))
        if original is not None
    ]
    if not allowed_originals:
        raise WorkflowError(f"{entry.milestone_id}: agent_writer requires exact allowed output files")
    if any(path.exists() and path.is_dir() for path in allowed_originals):
        raise WorkflowError(f"{entry.milestone_id}: agent_writer allowed outputs must be exact files, not directories")
    context_originals = [
        original
        for original in (
            writer_original_path(scope)
            for scope in [repo_path(plan_path), repo_path(manifest_path), *writer.agent_context_files]
        )
        if original is not None and original.exists()
    ]
    original_baseline = {repo_path(path): optional_file_digest(path) for path in allowed_originals}
    transcript_path = agent_transcript_path_for(
        track_id=track_id,
        run_id=run_id,
        entry=entry,
        run_root=run_ledger_root,
        action_kind="agent-writer",
    )
    with tempfile.TemporaryDirectory(prefix=f"manifest-agent-{entry.milestone_id.lower()}-") as temp_dir:
        workspace = Path(temp_dir) / "workspace"
        workspace.mkdir(parents=True, exist_ok=True)
        for original in list(dict.fromkeys([*allowed_originals, *context_originals])):
            copy_agent_workspace_input(workspace, original)
        allowed_map = {
            slash_path(action_workspace_path(workspace, original).relative_to(workspace)): original
            for original in allowed_originals
        }
        before = action_tree_digests(workspace)
        prompt = agent_writer_prompt(entry=entry, writer=writer, plan_path=plan_path, allowed_map=allowed_map)
        completed = run_codex_agent(workspace, prompt)
        after = action_tree_digests(workspace)
        changed = changed_action_workspace_files(before, after)
        write_agent_transcript(
            transcript_path,
            workspace=workspace,
            prompt=prompt,
            completed=completed,
            changed_files=changed,
        )
        if completed.returncode != 0:
            raise WorkflowError(
                f"{entry.milestone_id}: agent_writer codex exec failed with exit {completed.returncode}"
            )
        undeclared = sorted(path for path in changed if path not in allowed_map)
        if undeclared:
            raise WorkflowError(
                f"{entry.milestone_id}: agent_writer changed undeclared files: " + ", ".join(undeclared)
            )
        if not changed:
            raise WorkflowError(f"{entry.milestone_id}: agent_writer produced no scoped file changes")
        files: dict[Path, str] = {}
        for relative in changed:
            original = allowed_map[relative]
            baseline = original_baseline[repo_path(original)]
            current = optional_file_digest(original)
            if current != baseline:
                raise WorkflowError(
                    f"{entry.milestone_id}: target file digest drifted before import: {repo_path(original)}"
                )
            files[original] = (workspace / relative).read_text(encoding="utf-8")
    return AgentWriterResult(files=files, transcript_path=transcript_path)


def codex_contract_writer_prompt(
    *,
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
    plan_path: Path,
    allowed_map: dict[str, Path],
) -> str:
    allowed_lines = "\n".join(f"- `{relative}` -> `{repo_path(original)}`" for relative, original in sorted(allowed_map.items()))
    source_lines = "\n".join(f"- `{path}`" for path in contract.source_documents)
    forbidden_lines = "\n".join(f"- {scope}" for scope in [*contract.forbidden_scopes, *entry.forbidden_scope])
    validation_lines = "\n".join(f"- `{command}`" for command in contract.validation_commands)
    required_output_lines = "\n".join(f"- {output}" for output in contract.agent_required_outputs)
    section_lines = "\n".join(f"- {section}" for section in contract.required_sections)
    decision_lines = "\n".join(f"- {decision}" for decision in contract.required_decisions)
    extra_prompt = contract.agent_prompt or "Author the bounded design/contract outputs required by the active WR plan."
    return "\n".join(
        [
            "You are running inside an isolated Manifest Runner contract-authoring workspace.",
            "Modify only the allowed output files listed below. Do not create folders or files outside those paths.",
            "",
            f"Milestone: {entry.milestone_id} - {entry.title}",
            f"Seed production plan: {repo_path(plan_path)}",
            "",
            "Allowed output files:",
            allowed_lines or "- none",
            "",
            "Source documents:",
            source_lines or "- none",
            "",
            "Required sections:",
            section_lines or "- none",
            "",
            "Required decisions:",
            decision_lines or "- none",
            "",
            "Required outputs:",
            required_output_lines or "- Bounded contract evidence named by the active manifest entry.",
            "",
            "Forbidden scopes and patterns:",
            forbidden_lines or "- none",
            "",
            "Validation commands expected after import:",
            validation_lines or "- none",
            "",
            "Task:",
            extra_prompt,
            "",
            "Return normally after editing files. Do not run destructive git commands.",
        ]
    )


def codex_contract_writer_files(
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
    *,
    plan_path: Path,
    manifest_path: Path,
    track_id: str,
    run_id: str,
    run_ledger_root: Path,
    seed_plan_content: str,
) -> AgentWriterResult:
    if contract.agent_worktree_policy != "isolated_action_workspace":
        raise WorkflowError(f"{entry.milestone_id}: unsupported codex_contract_writer workspace policy {contract.agent_worktree_policy}")
    allowed_originals = [
        original
        for original in (writer_original_path(scope) for scope in contract.expected_output_paths)
        if original is not None
    ]
    if not allowed_originals:
        raise WorkflowError(f"{entry.milestone_id}: codex_contract_writer requires exact expected_output_paths")
    if any(path.exists() and path.is_dir() for path in allowed_originals):
        raise WorkflowError(f"{entry.milestone_id}: codex_contract_writer expected outputs must be exact files, not directories")
    context_originals = [
        original
        for original in (
            writer_original_path(scope)
            for scope in [repo_path(plan_path), repo_path(manifest_path), *contract.source_documents, *contract.agent_context_files]
        )
        if original is not None and original.exists()
    ]
    original_baseline = {repo_path(path): optional_file_digest(path) for path in allowed_originals}
    transcript_path = agent_transcript_path_for(
        track_id=track_id,
        run_id=run_id,
        entry=entry,
        run_root=run_ledger_root,
        action_kind="contract-writer",
    )
    with tempfile.TemporaryDirectory(prefix=f"manifest-contract-{entry.milestone_id.lower()}-") as temp_dir:
        workspace = Path(temp_dir) / "workspace"
        workspace.mkdir(parents=True, exist_ok=True)
        for original in list(dict.fromkeys([*allowed_originals, *context_originals])):
            copy_agent_workspace_input(workspace, original)
        seed_plan_path = action_workspace_path(workspace, plan_path)
        seed_plan_path.parent.mkdir(parents=True, exist_ok=True)
        seed_plan_path.write_text(seed_plan_content, encoding="utf-8", newline="\n")
        allowed_map = {
            slash_path(action_workspace_path(workspace, original).relative_to(workspace)): original
            for original in allowed_originals
        }
        before = action_tree_digests(workspace)
        prompt = codex_contract_writer_prompt(entry=entry, contract=contract, plan_path=plan_path, allowed_map=allowed_map)
        completed = run_codex_agent(workspace, prompt)
        after = action_tree_digests(workspace)
        changed = changed_action_workspace_files(before, after)
        write_agent_transcript(
            transcript_path,
            workspace=workspace,
            prompt=prompt,
            completed=completed,
            changed_files=changed,
        )
        if completed.returncode != 0:
            raise WorkflowError(
                f"{entry.milestone_id}: codex_contract_writer codex exec failed with exit {completed.returncode}"
            )
        undeclared = sorted(path for path in changed if path not in allowed_map)
        if undeclared:
            raise WorkflowError(
                f"{entry.milestone_id}: codex_contract_writer changed undeclared files: " + ", ".join(undeclared)
            )
        if not changed:
            raise WorkflowError(f"{entry.milestone_id}: codex_contract_writer produced no scoped file changes")
        files: dict[Path, str] = {}
        for relative in changed:
            original = allowed_map[relative]
            baseline = original_baseline[repo_path(original)]
            current = optional_file_digest(original)
            if current != baseline:
                raise WorkflowError(
                    f"{entry.milestone_id}: target file digest drifted before contract import: {repo_path(original)}"
                )
            files[original] = (workspace / relative).read_text(encoding="utf-8")
    return AgentWriterResult(files=files, transcript_path=transcript_path)


def apply_product_implementation(
    context: ManifestCommandContext,
    *,
    allow: set[str],
    roadmap_source: Path = ROADMAP_SOURCE,
    run_id: str = "",
    run_ledger_root: Path = TRACK_EXECUTION_RUN_ROOT,
    run_validations: bool = True,
) -> ProductImplementationResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if blockers:
        raise WorkflowError("\n".join(blockers))
    if workflow_action != "write_implementation_contract":
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not product_implementation")
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
    plan_path = default_contract_path(roadmap_item)
    assert_product_code_allowed(entry, milestone, roadmap_item, plan_path=plan_path, allow=allow)
    files_result = product_implementation_files(
        entry,
        plan_path=plan_path,
        manifest_path=context.loaded.path,
        track_id=context.track.id,
        run_id=run_id,
        run_ledger_root=run_ledger_root,
    )
    files = files_result.files
    assert_product_implementation_allowed(entry, roadmap_item, files=files, allow=allow)
    for path, content in files.items():
        if not path.parent.exists():
            raise WorkflowError(
                f"{entry.milestone_id}: product_implementation cannot create placeholder folders for {repo_path(path)}"
            )
        path.write_text(content, encoding="utf-8", newline="\n")
    validation_results = run_validation_commands(product_validation_commands_for_entry(entry)) if run_validations else ()
    manifest_data = updated_manifest_data_after_product_implementation(context.loaded, entry=entry)
    write_yaml_mapping(context.loaded.path, manifest_data)
    roadmap_data = updated_roadmap_data_after_product_implementation(
        roadmap_source,
        track_id=context.track.id,
        wr_id=entry.owning_wr,
        entry=entry,
    )
    write_yaml_mapping(roadmap_source, roadmap_data, indent_sequences=False)
    return ProductImplementationResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        plan_path=plan_path,
        manifest_path=context.loaded.path,
        written_paths=tuple(files.keys()),
        validation_commands=validation_results,
        next_legal_action=(
            f"{entry.milestone_id} product_implementation wrote bounded files for {entry.owning_wr}; "
            "rerun with agent_closeout to close only after runtime/test evidence remains valid."
        ),
        agent_transcript_path=files_result.transcript_path,
    )


def updated_manifest_data_after_product_implementation(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = load_yaml(loaded.path)
    next_action = (
        f"{entry.milestone_id} product_implementation completed; runtime closeout is the next legal action "
        "after validation evidence remains valid."
    )
    data["next_legal_action"] = next_action
    for milestone_data in data.get("milestones", []):
        if milestone_data.get("milestone_id") == entry.milestone_id:
            milestone_data["next_legal_action"] = next_action
            stop_conditions = milestone_data.setdefault("stop_conditions", [])
            marker = "product_implementation completed; stop before runtime closeout unless agent_closeout is explicitly allowed"
            if marker not in stop_conditions:
                stop_conditions.append(marker)
            break
    return data


def updated_roadmap_data_after_product_implementation(
    roadmap_source: Path,
    *,
    track_id: str,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = load_yaml(roadmap_source)
    changed = False
    for item in data.get("items", []):
        if item.get("id") != wr_id:
            continue
        item["next_evidence"] = (
            f"{entry.milestone_id} product_implementation completed under the accepted plan; "
            "runtime closeout evidence is now the next legal action."
        )
        item["current_decision"] = (
            f"Manifest Runner V5 wrote bounded product implementation for {entry.stage}. "
            "The WR remains active until runtime_proven closeout completes."
        )
        item["current_call"] = (
            "Next legal command, only for closeout: "
            f"task production:run-track -- --track {track_id} --allow agent_closeout --max-actions 1"
        )
        item["first_move"] = "Run runtime closeout only after validation evidence remains valid."
        item["main_blocker"] = "Runtime closeout evidence is missing; product implementation is present."
        item["why_not_ready"] = (
            f"{entry.stage} product implementation exists, but {entry.milestone_id} has not closed as runtime_proven."
        )
        item["diagram_call"] = ["product implementation complete", "runtime closeout next"]
        changed = True
        break
    if not changed:
        raise WorkflowError(f"{wr_id}: not present in active roadmap source")
    return data


def assert_product_implementation_allowed(
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    *,
    files: dict[Path, str],
    allow: set[str],
) -> None:
    errors: list[str] = []
    writer = entry.implementation_writer
    if "product_code" not in allow:
        errors.append("Manifest Runner V5 requires --allow product_code before product_implementation")
    if "product_implementation" not in allow:
        errors.append("Manifest Runner V5 requires --allow product_implementation to write product/runtime files")
    if entry.milestone_type not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: product_implementation supports implementation/runtime-proof milestones only")
    if writer is None:
        errors.append(f"{entry.milestone_id}: product_implementation requires implementation_writer config")
    elif writer.strategy == "no_writer":
        errors.append(f"{entry.milestone_id}: implementation_writer strategy is no_writer")
    elif not implementation_writer_allowed_scopes(writer):
        errors.append(f"{entry.milestone_id}: implementation_writer.allowed_files or allowed_write_scopes must be exact and non-empty")
    elif not writer.required_outputs:
        errors.append(f"{entry.milestone_id}: implementation_writer.required_outputs must describe proof evidence")
    elif not writer.validation_commands:
        errors.append(f"{entry.milestone_id}: implementation_writer.validation_commands must be explicit")
    elif not writer.stop_conditions:
        errors.append(f"{entry.milestone_id}: implementation_writer.stop_conditions must be explicit")
    if not files and not (writer is not None and writer.strategy == "proof_aggregation_writer" and writer.aggregation_only):
        errors.append(f"{entry.milestone_id}: product_implementation has no bounded writer for this milestone")
    write_paths = [repo_path(path) for path in files]
    if writer is not None:
        allowed_paths = {
            normalize_write_scope_path(scope)
            for scope in implementation_writer_allowed_scopes(writer)
            if normalize_write_scope_path(scope) is not None
        }
        for path in write_paths:
            normalized_path = normalize_repo_path(path)
            if normalized_path not in allowed_paths:
                errors.append(
                    f"{entry.milestone_id}: product_implementation writer output {normalized_path} is not declared in implementation_writer.allowed_files"
                )
        missing_writer_commands = [
            command
            for command in writer.validation_commands
            if command not in product_validation_commands_for_entry(entry)
        ]
        if missing_writer_commands:
            errors.append(
                f"{entry.milestone_id}: implementation_writer validation commands are not covered by product_code_contract: "
                + ", ".join(missing_writer_commands)
            )
    try:
        assert_runner_write_scope(
            entry=entry,
            roadmap_item=roadmap_item,
            write_paths=write_paths,
            action_label="product_implementation",
        )
    except WorkflowError as error:
        errors.extend(str(error).splitlines())
    wr_scopes = list(roadmap_item.write_scopes)
    product_scopes = product_implementation_scopes_for_entry(entry)
    new_scope_sources = [*wr_scopes, *product_scopes]
    for path in files:
        normalized = repo_path(path)
        if not write_scope_path_requires_new_marker(path):
            continue
        if writer is not None and writer.new_file_policy == "existing_files_only":
            errors.append(
                f"{entry.milestone_id}: implementation_writer new_file_policy forbids creating {normalized}"
            )
            continue
        if not new_scope_is_marked(normalized, new_scope_sources):
            errors.append(
                f"{entry.milestone_id}: new product file {normalized} must be marked with 'new:' in the owning WR or product_code_contract write scope"
            )
    if writer is not None:
        for forbidden in implementation_writer_forbidden_scopes(writer):
            forbidden_path = manifest_write_scope_path(forbidden)
            if forbidden_path is None:
                continue
            for path in write_paths:
                normalized_path = normalize_repo_path(path)
                if path_within_scope(normalized_path, forbidden_path) or path_within_scope(forbidden_path, normalized_path):
                    errors.append(
                        f"{entry.milestone_id}: product_implementation would touch implementation_writer forbidden file {forbidden_path}"
                    )
        for pattern in writer.forbidden_patterns:
            try:
                compiled = re.compile(pattern)
            except re.error as error:
                errors.append(f"{entry.milestone_id}: invalid implementation_writer forbidden pattern {pattern!r}: {error}")
                continue
            for path, content in files.items():
                searchable = f"{repo_path(path)}\n{content}"
                if compiled.search(searchable):
                    errors.append(
                        f"{entry.milestone_id}: product_implementation output matches forbidden pattern {pattern!r}"
                    )
    for forbidden in product_forbidden_scopes_for_entry(entry):
        forbidden_path = manifest_write_scope_path(forbidden)
        if forbidden_path is None:
            continue
        for path in write_paths:
            normalized_path = normalize_repo_path(path)
            if path_within_scope(normalized_path, forbidden_path) or path_within_scope(forbidden_path, normalized_path):
                errors.append(f"{entry.milestone_id}: product_implementation would touch forbidden scope {forbidden_path}")
    if errors:
        raise WorkflowError("\n".join(errors))


def new_scope_is_marked(path: str, scopes: list[str]) -> bool:
    normalized = normalize_repo_path(path)
    return any(is_new_write_scope(scope) and normalize_write_scope_path(scope) == normalized for scope in scopes)


def product_forbidden_scopes_for_entry(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return [*entry.forbidden_scope, *entry.product_code_contract.forbidden_implementation_scopes]
    return list(entry.forbidden_scope)


def implementation_writer_allowed_scopes(writer: ManifestImplementationWriter) -> list[str]:
    return [*writer.allowed_files, *writer.allowed_write_scopes]


def implementation_writer_forbidden_scopes(writer: ManifestImplementationWriter) -> list[str]:
    return [*writer.forbidden_files, *writer.forbidden_scopes]


def implementation_writer_output_scopes(writer: ManifestImplementationWriter) -> list[str]:
    outputs = [*writer.allowed_files, *writer.allowed_write_scopes]
    outputs.extend(template.file for template in writer.templates)
    outputs.extend(patch.file for patch in writer.patches)
    return outputs


def product_implementation_files(
    entry: TrackExecutionManifestMilestone,
    *,
    plan_path: Path | None = None,
    manifest_path: Path | None = None,
    track_id: str = "",
    run_id: str = "",
    run_ledger_root: Path = TRACK_EXECUTION_RUN_ROOT,
) -> AgentWriterResult:
    writer = entry.implementation_writer
    if writer is None or writer.strategy == "no_writer":
        raise WorkflowError(f"{entry.milestone_id}: implementation_writer strategy is no_writer")
    if writer.strategy == "template_writer":
        return AgentWriterResult(files=template_writer_files(entry, writer), transcript_path=None)
    if writer.strategy == "patch_writer":
        return AgentWriterResult(files=patch_writer_files(entry, writer), transcript_path=None)
    if writer.strategy == "agent_writer":
        if plan_path is None or manifest_path is None:
            raise WorkflowError(f"{entry.milestone_id}: agent_writer requires plan and manifest context")
        return agent_writer_files(
            entry,
            writer,
            plan_path=plan_path,
            manifest_path=manifest_path,
            track_id=track_id,
            run_id=run_id,
            run_ledger_root=run_ledger_root,
        )
    if writer.strategy == "proof_aggregation_writer":
        return AgentWriterResult(files=proof_aggregation_writer_files(entry, writer), transcript_path=None)
    raise WorkflowError(f"{entry.milestone_id}: unsupported implementation_writer strategy {writer.strategy!r}")


def proof_aggregation_writer_files(
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
) -> dict[Path, str]:
    if writer.templates and writer.patches:
        raise WorkflowError(f"{entry.milestone_id}: proof_aggregation_writer cannot mix templates and patches")
    if writer.templates:
        return template_writer_files(entry, writer)
    if writer.patches:
        return patch_writer_files(entry, writer)
    return {}


def template_writer_files(
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
) -> dict[Path, str]:
    if not writer.templates:
        raise WorkflowError(f"{entry.milestone_id}: template_writer requires at least one declared template")
    return {
        implementation_writer_path(template.file): template.content
        for template in writer.templates
    }


def patch_writer_files(
    entry: TrackExecutionManifestMilestone,
    writer: ManifestImplementationWriter,
) -> dict[Path, str]:
    if not writer.patches:
        raise WorkflowError(f"{entry.milestone_id}: {writer.strategy} requires declared bounded patches")
    outputs: dict[Path, str] = {}
    errors: list[str] = []
    for patch in writer.patches:
        path = implementation_writer_path(patch.file)
        if path in outputs:
            current = outputs[path]
        elif path.exists():
            current = path.read_text(encoding="utf-8")
        else:
            current = ""
        if patch.find not in current:
            if patch.replace in current:
                outputs[path] = current
                continue
            errors.append(f"{entry.milestone_id}: patch find text not found in {patch.file}")
            continue
        outputs[path] = current.replace(patch.find, patch.replace, 1)
    if errors:
        raise WorkflowError("\n".join(errors))
    return outputs


def implementation_writer_path(path: str) -> Path:
    candidate = Path(path.strip())
    if candidate.is_absolute():
        return candidate
    return REPO_ROOT / normalize_repo_path(path)


def assert_agent_design_source_documents(contract: ManifestAgentDesignContract) -> None:
    missing = [path for path in contract.source_documents if not (REPO_ROOT / path).exists()]
    if missing:
        raise WorkflowError("agent_design source documents are missing: " + ", ".join(missing))


def assert_agent_design_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    allow: set[str],
    deny: set[str],
) -> ManifestAgentDesignContract:
    if "agent_design" not in allow:
        raise WorkflowError("Manifest Runner V2 requires --allow agent_design for design/planning document mutation")
    if milestone.state == "completed":
        raise WorkflowError(f"{entry.milestone_id}: agent_design cannot mutate completed milestones")
    if "agent_design completed design/planning writes" in entry.next_legal_action:
        raise WorkflowError(
            f"{entry.milestone_id}: agent_design output already exists; rerun with --allow agent_closeout for closeout"
        )
    if entry.milestone_type not in {"docs_only", "design_only", "implementation", "hardening"}:
        raise WorkflowError(f"{entry.milestone_id}: agent_design supports docs, design, implementation, or hardening milestones only")
    if not entry.owning_wr:
        raise WorkflowError(f"{entry.milestone_id}: agent_design requires an owning WR")
    contract = agent_design_contract_for_entry(entry)
    if contract is None:
        raise WorkflowError(f"{entry.milestone_id}: manifest milestone is missing agent_design contract")
    assert_agent_design_source_documents(contract)
    return contract


def agent_design_plan_content(
    plan_context: ProductionPlanContext,
    *,
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
    plan_path: Path,
) -> str:
    if entry.milestone_type in {"implementation", "hardening"}:
        return agent_design_implementation_plan_content(
            plan_context,
            entry=entry,
            contract=contract,
            plan_path=plan_path,
        )
    readiness = render_readiness_report(plan_context, classify_plan_action(plan_context), plan_path)
    stage_label = entry.stage
    required_sections_heading = "Required Design Sections"
    source_lines = "\n".join(f"- `{path}`" for path in contract.source_documents)
    related_design_lines = "\n".join(f"  - ../../../{path.removeprefix('docs-site/src/content/docs/')}" for path in contract.source_documents)
    section_lines = "\n".join(f"- {section}" for section in contract.required_sections)
    decision_lines = "\n".join(f"- {decision}" for decision in contract.required_decisions)
    acceptance_lines = "\n".join(f"- {item}" for item in contract.acceptance_checklist)
    implementation_scopes = product_implementation_scopes_for_entry(entry)
    write_scope_lines = "\n".join(f"- `{scope}`" for scope in implementation_scopes)
    forbidden_lines = "\n".join(f"- {scope}" for scope in entry.forbidden_scope)
    validation_lines = "\n".join(f"- `{command}`" for command in entry.validation_commands)
    stop_lines = "\n".join(f"- {condition}" for condition in entry.stop_conditions)
    return "\n".join(
        [
            "---",
            f"title: {plan_context.roadmap_item.id} {entry.title} Plan",
            f"description: Design/planning contract for {entry.milestone_id} under {plan_context.roadmap_item.id}.",
            "status: active",
            "owner: ui",
            "layer: workspace / domain/ui",
            "canonical: false",
            f"last_reviewed: {plan_context.planning.production.last_reviewed}",
            "related:",
            "  - ../../../workspace/production-tracks.yaml",
            "  - ../../../workspace/roadmap-deferred.yaml",
            f"  - ../../../workspace/track-execution-manifests/{plan_context.track.id.lower()}.yaml",
            "related_designs:",
            related_design_lines or "  - ../../../workspace/track-execution-manifest.md",
            "---",
            "",
            f"# {plan_context.roadmap_item.id} {entry.title} Plan",
            "",
            "## Status And Authority",
            "",
            f"- Production milestone: `{entry.milestone_id}` - {entry.title}",
            f"- Stage: {stage_label}",
            f"- Roadmap item: `{plan_context.roadmap_item.id}` - {plan_context.roadmap_item.title}",
            "- Authority: design/planning only.",
            "- This plan does not authorize product/runtime code, crate creation, placeholder future folders, runtime proof work, downstream implementation, or shared foundation/meta extraction.",
            "- This plan does not close the milestone. Closeout requires separate manual evidence or an explicit `agent_closeout` automation run.",
            "",
            "## Production Planning Output",
            "",
            readiness,
            "",
            "## Source Documents",
            "",
            source_lines,
            "",
            f"## {required_sections_heading}",
            "",
            section_lines,
            "",
            "## Required Decisions",
            "",
            decision_lines,
            "",
            "## Exact Write Scope",
            "",
            write_scope_lines,
            "",
            "## Forbidden Scope",
            "",
            forbidden_lines,
            "",
            "## Acceptance Checklist",
            "",
            acceptance_lines,
            "",
            "## Validation Commands",
            "",
            validation_lines,
            "",
            "## Stop Conditions",
            "",
            stop_lines,
            f"- Stop after writing the design/planning contract and rerun `task production:next -- --track {plan_context.track.id}`.",
            "- Stop before closeout unless rerun with `--allow agent_closeout` and the required evidence and validation pass.",
            "",
            "## Closeout Expectation",
            "",
            f"- Expected closeout path: `{entry.expected_closeout_path}`",
            f"- Closeout must prove the {stage_label} design sections exist, answer or explicitly block open questions, and preserve all forbidden-scope constraints.",
            "",
            agent_design_contract_section(entry, contract),
        ]
    )


def proof_slice_id_for_entry(entry: TrackExecutionManifestMilestone) -> str:
    if entry.contract_parameters and entry.contract_parameters.proof_slice_id:
        return entry.contract_parameters.proof_slice_id
    stage_match = re.search(r"stage\s+([0-9]+[a-z])", entry.stage.lower())
    if stage_match:
        return stage_match.group(1).upper()
    proof_kind = entry.implementation_proof_kind or entry.closeout_kind or entry.milestone_id
    proof_match = re.search(r"([0-9]+[a-z])", proof_kind.lower())
    if proof_match:
        return proof_match.group(1).upper()
    return entry.milestone_id


def proof_slice_title_for_entry(entry: TrackExecutionManifestMilestone) -> str:
    if entry.contract_parameters and entry.contract_parameters.proof_slice_title:
        return entry.contract_parameters.proof_slice_title
    return entry.title


def target_control_surface_for_entry(entry: TrackExecutionManifestMilestone) -> str:
    if entry.contract_parameters and entry.contract_parameters.target_control_surface:
        return entry.contract_parameters.target_control_surface
    title = re.sub(r"^\s*[0-9]+[A-Za-z]\s+", "", entry.title).strip()
    title = re.sub(r"\s+Proof\s*$", "", title).strip()
    return title or entry.title


def bullet_lines(items: list[str]) -> str:
    return "\n".join(f"- {item}" for item in items)


def code_bullet_lines(items: list[str]) -> str:
    return "\n".join(f"- `{item}`" for item in items)


def implementation_plan_validation_commands(entry: TrackExecutionManifestMilestone) -> list[str]:
    return product_validation_commands_for_entry(entry)


def implementation_plan_forbidden_scopes(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.forbidden_implementation_scopes)
    return list(entry.forbidden_scope)


def implementation_plan_tests(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.tests_to_add_change)
    return default_contract_params_list(entry, "tests_to_add_change", [f"Focused {entry.stage} tests in scoped modules only."])


def implementation_plan_runtime_evidence(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.runtime_evidence_required)
    return default_contract_params_list(entry, "runtime_evidence_required", [f"{entry.stage} runtime/test evidence."])


def implementation_plan_rollback(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.rollback_compatibility_expectations)
    return default_contract_params_list(
        entry,
        "rollback_compatibility_expectations",
        ["Rollback is limited to the exact allowed implementation write scopes."],
    )


def implementation_plan_method_scope(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.product_code_contract is not None:
        return list(entry.product_code_contract.required_function_method_scope)
    return default_contract_params_list(
        entry,
        "required_function_method_scope",
        [f"Bounded {proof_slice_title_for_entry(entry)} functions/methods only."],
    )


def agent_design_implementation_plan_content(
    plan_context: ProductionPlanContext,
    *,
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
    plan_path: Path,
) -> str:
    readiness = "\n".join(
        [
            f"- Production track: `{plan_context.track.id}` - {plan_context.track.title}",
            f"- Production milestone: `{entry.milestone_id}` - {entry.title}",
            "- Production milestone state after plan acceptance: `active`",
            f"- Roadmap item: `{plan_context.roadmap_item.id}` - {plan_context.roadmap_item.title}",
            "- Roadmap planning state after plan acceptance: `current_candidate`",
            "- Roadmap blocker after plan acceptance: `B2`",
            f"- Contract target: `{repo_path(plan_path)}`",
            "- Post-plan transition: Manifest Runner V2 records the milestone as `active` and the WR as `current_candidate`/`B2`.",
            "- Next action after this plan is accepted: stop before product/runtime code unless rerun with explicit `--allow product_code --allow product_implementation` and all V5 gates pass.",
        ]
    )
    source_lines = "\n".join(f"- `{path}`" for path in contract.source_documents)
    related_design_lines = "\n".join(f"  - ../../../{path.removeprefix('docs-site/src/content/docs/')}" for path in contract.source_documents)
    write_scope_lines = "\n".join(f"- `{scope}`" for scope in product_implementation_scopes_for_entry(entry))
    forbidden_lines = bullet_lines(implementation_plan_forbidden_scopes(entry))
    validation_lines = code_bullet_lines(implementation_plan_validation_commands(entry))
    section_lines = "\n".join(f"- {section}" for section in contract.required_sections)
    decision_lines = "\n".join(f"- {decision}" for decision in contract.required_decisions)
    acceptance_lines = "\n".join(f"- {item}" for item in contract.acceptance_checklist)
    stop_conditions = (
        list(entry.product_code_contract.stop_conditions)
        if entry.product_code_contract is not None
        else list(entry.stop_conditions)
    )
    stop_lines = bullet_lines(stop_conditions)
    method_scope_lines = bullet_lines(implementation_plan_method_scope(entry))
    test_lines = bullet_lines(implementation_plan_tests(entry))
    runtime_evidence_lines = bullet_lines(implementation_plan_runtime_evidence(entry))
    rollback_lines = bullet_lines(implementation_plan_rollback(entry))
    proof_slice_id = proof_slice_id_for_entry(entry)
    proof_slice_title = proof_slice_title_for_entry(entry)
    target_surface = target_control_surface_for_entry(entry)
    implementation_proof_kind = entry.implementation_proof_kind or proof_slice_id.lower()
    content = "\n".join(
        [
            "---",
            f"title: {plan_context.roadmap_item.id} {entry.title} Implementation Plan",
            f"description: Product-code implementation planning contract for {entry.milestone_id} under {plan_context.roadmap_item.id}.",
            "status: active",
            "owner: ui",
            "layer: domain/ui",
            "canonical: false",
            f"last_reviewed: {plan_context.planning.production.last_reviewed}",
            "related:",
            "  - ../../../workspace/production-tracks.yaml",
            "  - ../../../workspace/roadmap-items.yaml",
            f"  - ../../../workspace/track-execution-manifests/{plan_context.track.id.lower()}.yaml",
            "related_designs:",
            related_design_lines or "  - ../../../workspace/track-execution-manifest.md",
            "---",
            "",
            f"# {plan_context.roadmap_item.id} {entry.title} Implementation Plan",
            "",
            "## Status And Authority",
            "",
            f"- Production milestone: `{entry.milestone_id}` - {entry.title}",
            f"- Stage: {entry.stage}",
            f"- Roadmap item: `{plan_context.roadmap_item.id}` - {plan_context.roadmap_item.title}",
            f"- Proof slice id: `{proof_slice_id}`",
            f"- Proof slice title: {proof_slice_title}",
            f"- Target control/surface: {target_surface}",
            f"- Implementation proof kind: `{implementation_proof_kind}`",
            "- Authority: implementation planning only.",
            "- This plan is the accepted production plan required before Manifest Runner V5 may run with `--allow product_code --allow product_implementation`.",
            "- This plan does not execute product/runtime code and does not close the milestone.",
            "- This plan does not authorize crate creation, placeholder future folders, downstream implementation outside the active WR, broad retained rewrites, or shared `foundation/meta` extraction.",
            "",
            "## Production Planning Output",
            "",
            readiness,
            "",
            "## Source Documents",
            "",
            source_lines,
            "",
            "## Exact Files/Modules Expected To Change",
            "",
            write_scope_lines,
            "",
            "## Expected Methods/Functions",
            "",
            method_scope_lines,
            "",
            "## Required Implementation Scope",
            "",
            section_lines,
            "",
            "## Required Decisions",
            "",
            decision_lines,
            "",
            "## Forbidden Files/Modules",
            "",
            forbidden_lines,
            "",
            "## Tests To Add/Change",
            "",
            test_lines,
            "- Do not create new test folders, new crates, placeholder future modules, or broad fixture frameworks.",
            "",
            "## Validation Commands",
            "",
            validation_lines,
            "",
            "## Closeout Requirements",
            "",
            f"- Expected closeout path: `{entry.expected_closeout_path}`",
            "- Closeout evidence must include:",
            runtime_evidence_lines,
            "- Closeout must explicitly state that no out-of-scope domain semantics, downstream implementation, crates, placeholder folders, or `foundation/meta` extraction occurred.",
            "",
            "## Compatibility / Rollback Plan",
            "",
            rollback_lines,
            "",
            "## Acceptance Checklist",
            "",
            acceptance_lines,
            "",
            "## Stop Conditions",
            "",
            stop_lines,
            "- Stop before product/runtime code unless the command is rerun with `--allow product_code --allow product_implementation` and all V5 gates pass.",
            "- Stop before the next milestone until this milestone has runtime/test closeout evidence.",
            "",
            "## Next Command If Product Code Is Permitted",
            "",
            f"`task production:run-track -- --track {plan_context.track.id} --allow product_code --allow product_implementation --max-actions 1`",
            "",
        ]
    )
    errors = implementation_plan_consistency_errors_from_text(
        entry,
        roadmap_item=plan_context.roadmap_item,
        text=content,
        plan_path=plan_path,
    )
    if errors:
        raise WorkflowError("\n".join(errors))
    return content


def upsert_markdown_section(text: str, *, heading: str, section: str, before_heading: str) -> str:
    if heading in text:
        start = text.index(heading)
        next_heading = text.find("\n## ", start + len(heading))
        end = next_heading if next_heading != -1 else len(text)
        return text[:start] + section.rstrip() + "\n\n" + text[end:].lstrip("\n")
    before = text.find(before_heading)
    if before == -1:
        return text.rstrip() + "\n\n" + section.rstrip() + "\n"
    return text[:before] + section.rstrip() + "\n\n" + text[before:]


def generic_agent_design_contract_section(
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
) -> str:
    section_lines = "\n".join(f"- {section}" for section in contract.required_sections)
    decision_lines = "\n".join(f"- {decision}" for decision in contract.required_decisions)
    acceptance_lines = "\n".join(f"- {item}" for item in contract.acceptance_checklist)
    return "\n".join(
        [
            f"## {entry.milestone_id} {entry.stage} Contract",
            "",
            f"This section is the bounded {entry.stage} contract produced for `{entry.milestone_id}`.",
            "It tightens design-level contracts only. It does not authorize code,",
            "crates, placeholder folders, runtime proof work, downstream implementation, or",
            "shared `foundation/meta` extraction.",
            "",
            "### Required Design Coverage",
            "",
            section_lines,
            "",
            "### Required Decisions",
            "",
            decision_lines,
            "",
            "### Acceptance Checklist",
            "",
            acceptance_lines,
            "",
            "### Remaining Authority Boundary",
            "",
            "- This bounded design evidence may close only as `bounded_contract`.",
            "- It must not claim `runtime_proven` evidence.",
            "- It must not start any product-code slice.",
            "- Later milestones still require their own WR, plan, validation, and closeout evidence.",
            "",
        ]
    )


def agent_design_contract_section(
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
) -> str:
    return generic_agent_design_contract_section(entry, contract)


def update_ui_program_architecture(
    path: Path,
    *,
    entry: TrackExecutionManifestMilestone,
    contract: ManifestAgentDesignContract,
) -> None:
    original = path.read_text(encoding="utf-8")
    heading = f"## {entry.milestone_id} {entry.stage} Contract"
    updated = upsert_markdown_section(
        original,
        heading=heading,
        section=agent_design_contract_section(entry, contract),
        before_heading="## 13. Staged Implementation Plan",
    )
    path.write_text(updated, encoding="utf-8", newline="\n")


def updated_manifest_data_after_agent_design(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    if entry.milestone_type in {"implementation", "hardening"}:
        next_action = (
            f"{entry.milestone_id} implementation plan exists; stop before product_code. "
            f"Rerun `task production:run-track -- --track {loaded.manifest.track_id} --allow product_code --allow product_implementation --max-actions 1` only if product/runtime code is permitted."
        )
    else:
        next_action = (
            f"{entry.milestone_id} agent_design completed design/planning writes; "
            "stop for closeout; rerun with --allow agent_closeout after evidence is valid."
        )
    data["next_legal_action"] = next_action
    for milestone_data in data["milestones"]:
        if milestone_data["milestone_id"] != entry.milestone_id:
            continue
        milestone_data["next_legal_action"] = next_action
        if entry.milestone_type in {"implementation", "hardening"}:
            product_stop = "implementation plan exists; stop before product_code unless explicitly allowed and V5 product_implementation gates pass"
            if product_stop not in milestone_data["stop_conditions"]:
                milestone_data["stop_conditions"].append(product_stop)
        else:
            closeout_stop = "stop before closeout unless agent_closeout is explicitly allowed and evidence is valid"
            if closeout_stop not in milestone_data["stop_conditions"]:
                milestone_data["stop_conditions"].append(closeout_stop)
    return data


def updated_deferred_roadmap_data_after_agent_design(
    roadmap_source: Path,
    *,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
    plan_path: Path,
) -> tuple[Path, dict]:
    _, deferred_source = split_source_paths(roadmap_source)
    deferred_data = load_yaml(deferred_source)
    changed = False
    for item in deferred_data.get("items", []):
        if item.get("id") != wr_id:
            continue
        item["next_evidence"] = (
            f"{entry.milestone_id} design/planning output exists at {repo_path(plan_path)}; "
            "closeout evidence is still required."
        )
        item["current_decision"] = (
            f"Manifest Runner V2 agent_design wrote the {entry.stage} {entry.title} plan and architecture sections. "
            "This remains design/planning evidence only."
        )
        item["current_call"] = (
            "Stop before closeout unless agent_closeout is explicitly allowed and evidence is valid. No product code, crates, runtime proof work, downstream implementation, "
            "or foundation/meta work is authorized."
        )
        item["main_blocker"] = "Closeout evidence is missing; rerun with agent_closeout only after evidence and validation are valid."
        item["why_not_ready"] = f"Design/planning content exists, but {entry.milestone_id} has not been closed with evidence."
        changed = True
        break
    if not changed:
        raise WorkflowError(f"{wr_id}: not present in deferred roadmap source")
    return deferred_source, deferred_data


def implementation_roadmap_item_after_agent_design(
    item: dict,
    *,
    entry: TrackExecutionManifestMilestone,
    plan_path: Path,
    track_id: str,
    production_source: Path,
    roadmap_source: Path,
    manifest_source: Path,
) -> dict:
    updated = dict(item)
    updated["planning_state"] = "current_candidate"
    updated["gate"] = "Implementation plan accepted"
    updated["blocker"] = 2
    planning_scopes = []
    contract = agent_design_contract_for_entry(entry)
    if contract is not None:
        planning_scopes = [
            normalized
            for normalized in (
                manifest_write_scope_path(scope)
                for scope in contract.planning_write_scope
                if not is_generated_or_derived_scope(scope)
            )
            if normalized is not None and normalized != normalize_repo_path(entry.expected_closeout_path)
        ]
    product_scopes = [
        roadmap_write_scope_for_product_scope(normalized)
        for normalized in (manifest_write_scope_path(scope) for scope in product_implementation_scopes_for_entry(entry))
        if normalized is not None
    ]
    updated["write_scopes"] = list(dict.fromkeys([*planning_scopes, *product_scopes]))
    updated["validations"] = product_validation_commands_for_entry(entry)
    updated["next_evidence"] = (
        f"{entry.milestone_id} accepted implementation plan exists at {repo_path(plan_path)}; "
        "product_code has not run."
    )
    updated["current_decision"] = (
        f"Manifest Runner V2 agent_design created the accepted {entry.stage} implementation plan. "
        "This authorizes only the next V5 product_implementation gate when explicitly allowed."
    )
    updated["current_call"] = (
        f"Next legal command, only when product/runtime code is permitted: "
        f"task production:run-track -- --track {track_id} --allow product_code --allow product_implementation --max-actions 1"
    )
    updated["first_move"] = f"Run task production:run-track -- --track {track_id} --allow product_code --allow product_implementation --max-actions 1."
    updated["main_blocker"] = "Product_code has not run; runtime/test evidence and closeout are still missing."
    updated["why_not_ready"] = (
        f"Accepted implementation plan exists, but {entry.stage} product_code and closeout evidence have not started."
    )
    updated["completion_quality"] = "not_applicable"
    updated["known_quality_gaps"] = []
    updated["completion_audit"] = ""
    updated["diagram_call"] = ["current candidate", "implementation plan only"]
    return updated


def roadmap_write_scope_for_product_scope(normalized: str) -> str:
    scope_path = REPO_ROOT / normalized
    if not scope_path.exists() and scope_path.parent.exists():
        return f"new: {normalized}"
    return normalized


def updated_production_data_after_agent_design(
    production_source: Path,
    *,
    track_id: str,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = load_yaml(production_source)
    changed = False
    for track_data in data.get("tracks", []):
        if track_data.get("id") != track_id:
            continue
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") != entry.milestone_id:
                continue
            if entry.milestone_type in {"implementation", "hardening"} and milestone_data.get("state") == "designing":
                milestone_data["state"] = "active"
            changed = True
            break
    if not changed:
        raise WorkflowError(f"{entry.milestone_id}: not found in production source {repo_path(production_source)}")
    return data


def updated_implementation_roadmap_sources_after_agent_design(
    roadmap_source: Path,
    *,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
    plan_path: Path,
    track_id: str,
    production_source: Path,
    manifest_source: Path,
) -> tuple[Path, dict, Path, dict]:
    active_data = load_yaml(roadmap_source)
    _, deferred_source = split_source_paths(roadmap_source)
    deferred_data = load_yaml(deferred_source)
    item_data: dict | None = None
    for index, candidate in enumerate(list(deferred_data.get("items", []))):
        if candidate.get("id") == wr_id:
            item_data = deferred_data["items"].pop(index)
            break
    if item_data is None:
        for index, candidate in enumerate(list(active_data.get("items", []))):
            if candidate.get("id") == wr_id:
                item_data = active_data["items"].pop(index)
                break
    if item_data is None:
        raise WorkflowError(f"{wr_id}: not present in active or deferred roadmap source")
    active_data.setdefault("items", []).append(
        implementation_roadmap_item_after_agent_design(
            item_data,
            entry=entry,
            plan_path=plan_path,
            track_id=track_id,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source=manifest_source,
        )
    )
    return roadmap_source, active_data, deferred_source, deferred_data


def apply_agent_design(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    deny: set[str],
    run_validations: bool = True,
    allow_regenerate_invalid_implementation_plan: bool = False,
    run_id: str = "",
    run_ledger_root: Path = TRACK_EXECUTION_RUN_ROOT,
) -> AgentDesignResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    if blockers:
        raise WorkflowError("\n".join(blockers))
    if workflow_action != "design_first" and not (
        allow_regenerate_invalid_implementation_plan
        and workflow_action == "write_implementation_contract"
        and entry.milestone_type in {"implementation", "hardening"}
    ):
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not agent_design")
    contract = assert_agent_design_allowed(entry, milestone, allow=allow, deny=deny)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")

    plan_context = resolve_plan_context(
        entry.milestone_id,
        entry.owning_wr,
        production_source=production_source,
        roadmap_source=roadmap_source,
    )
    plan_path = default_contract_path(plan_context.roadmap_item)
    design_paths: tuple[Path, ...] = ()
    agent_files: dict[Path, str] = {}
    agent_transcript_path: Path | None = None
    _, deferred_source = split_source_paths(roadmap_source)
    write_paths = [
        repo_path(plan_path),
        repo_path(context.loaded.path),
        repo_path(deferred_source),
        *(repo_path(path) for path in design_paths),
    ]
    if contract.authoring_strategy == "codex_contract_writer":
        for output_path in contract.expected_output_paths:
            normalized = manifest_write_scope_path(output_path)
            if normalized is not None and not is_generated_or_derived_scope(output_path):
                write_paths.append(normalized)
    if entry.milestone_type in {"implementation", "hardening"}:
        write_paths.append(repo_path(production_source))
        write_paths.append(repo_path(roadmap_source))
    assert_agent_design_write_scope(entry=entry, roadmap_item=roadmap_item, contract=contract, write_paths=write_paths)

    plan_content = agent_design_plan_content(plan_context, entry=entry, contract=contract, plan_path=plan_path)
    if contract.authoring_strategy == "codex_contract_writer":
        writer_result = codex_contract_writer_files(
            entry,
            contract,
            plan_path=plan_path,
            manifest_path=context.loaded.path,
            track_id=context.track.id,
            run_id=run_id,
            run_ledger_root=run_ledger_root,
            seed_plan_content=plan_content,
        )
        agent_files = writer_result.files
        agent_transcript_path = writer_result.transcript_path

    plan_path.parent.mkdir(parents=True, exist_ok=True)
    plan_path.write_text(agent_files.pop(plan_path, plan_content), encoding="utf-8", newline="\n")
    for path, content in agent_files.items():
        if not path.parent.exists():
            raise WorkflowError(
                f"{entry.milestone_id}: codex_contract_writer cannot create placeholder folders for {repo_path(path)}"
            )
        path.write_text(content, encoding="utf-8", newline="\n")
    for design_path in design_paths:
        update_ui_program_architecture(design_path, entry=entry, contract=contract)

    loaded_for_update = context.loaded
    if context.loaded.path in agent_files:
        try:
            loaded_for_update = LoadedTrackExecutionManifest(
                manifest=TrackExecutionManifest.model_validate(load_yaml(context.loaded.path)),
                path=context.loaded.path,
            )
        except Exception as error:
            raise WorkflowError(
                f"{entry.milestone_id}: codex_contract_writer produced invalid manifest YAML: {error}"
            ) from error
    manifest_data = updated_manifest_data_after_agent_design(loaded_for_update, entry=entry)
    write_yaml_mapping(context.loaded.path, manifest_data)
    if entry.milestone_type in {"implementation", "hardening"}:
        production_data = updated_production_data_after_agent_design(
            production_source,
            track_id=context.track.id,
            entry=entry,
        )
        active_source, active_data, deferred_source, deferred_data = updated_implementation_roadmap_sources_after_agent_design(
            roadmap_source,
            wr_id=entry.owning_wr,
            entry=entry,
            plan_path=plan_path,
            track_id=context.track.id,
            production_source=production_source,
            manifest_source=context.loaded.path,
        )
        write_yaml_mapping(production_source, production_data)
        write_yaml_mapping(active_source, active_data, indent_sequences=False)
        write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)
    else:
        deferred_source, deferred_data = updated_deferred_roadmap_data_after_agent_design(
            roadmap_source,
            wr_id=entry.owning_wr,
            entry=entry,
            plan_path=plan_path,
        )
        write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)

    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    if entry.milestone_type in {"implementation", "hardening"}:
        next_legal_action = (
            f"{entry.milestone_id} implementation plan exists; stop before product_code. "
            f"Next permitted command, only with explicit authorization: "
            f"task production:run-track -- --track {context.track.id} --allow product_code --allow product_implementation --max-actions 1"
        )
    else:
        next_legal_action = (
            f"{entry.milestone_id} design/planning output exists; stop for closeout and rerun with --allow agent_closeout after evidence is valid."
        )
    return AgentDesignResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        plan_path=plan_path,
        design_paths=design_paths,
        manifest_path=context.loaded.path,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
        agent_transcript_path=agent_transcript_path,
    )


def agent_closeout_pending(entry: TrackExecutionManifestMilestone) -> bool:
    if entry.milestone_type == "closeout" and entry.closeout_strategy == "handoff_closeout":
        return True
    return "agent_design completed design/planning writes" in entry.next_legal_action


def agent_closeout_completion_quality(entry: TrackExecutionManifestMilestone) -> str:
    if entry.milestone_type == "closeout" and entry.closeout_strategy == "handoff_closeout":
        allowed = entry.agent_closeout_contract.completion_quality_allowed if entry.agent_closeout_contract else []
        if "runtime_proven" in allowed:
            return "runtime_proven"
        if allowed:
            return allowed[0]
    return "bounded_contract"


def assert_agent_closeout_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    *,
    allow: set[str],
    deny: set[str],
) -> None:
    if "agent_closeout" not in allow:
        raise WorkflowError("Manifest Runner V3 requires --allow agent_closeout for closeout mutation")
    if entry.may_create_code or entry.may_create_crates or entry.may_modify_production_behavior:
        raise WorkflowError(
            f"{entry.milestone_id}: agent_closeout supports docs, design, or governance milestones only; "
            "runtime/product milestones need a future runtime evidence closeout path"
        )
    is_design_closeout = milestone.kind == "design" and entry.milestone_type in {"docs_only", "design_only"}
    is_handoff_closeout = (
        milestone.kind == "release"
        and entry.milestone_type == "closeout"
        and entry.closeout_strategy == "handoff_closeout"
        and entry.handoff_contract is not None
    )
    if not is_design_closeout and not is_handoff_closeout:
        raise WorkflowError(
            f"{entry.milestone_id}: agent_closeout supports docs/design milestones or explicit handoff_closeout milestones only"
        )
    if milestone.state == "completed":
        raise WorkflowError(f"{entry.milestone_id}: milestone is already completed")
    if is_design_closeout and milestone.completion_quality == "runtime_proven":
        raise WorkflowError(f"{entry.milestone_id}: docs or design milestones cannot close as runtime_proven")
    if not entry.owning_wr:
        raise WorkflowError(f"{entry.milestone_id}: agent_closeout requires an owning WR")
    if not agent_closeout_pending(entry):
        raise WorkflowError(
            f"{entry.milestone_id}: closeout evidence is not ready; run the required design/governance evidence step first"
        )


def required_agent_design_evidence_markers(entry: TrackExecutionManifestMilestone) -> list[str]:
    markers = [f"## {entry.milestone_id} {entry.stage} Contract"]
    contract = agent_design_contract_for_entry(entry)
    if contract is not None:
        markers.extend(contract.required_sections)
        markers.extend(contract.required_decisions)
        markers.extend(contract.acceptance_checklist)
    return markers


def agent_closeout_evidence_paths(
    *,
    entry: TrackExecutionManifestMilestone,
    roadmap_item: RoadmapItem,
    plan_path: Path,
) -> tuple[list[Path], list[str]]:
    if entry.milestone_type == "closeout" and entry.closeout_strategy == "handoff_closeout":
        evidence_paths: list[Path] = []
        errors: list[str] = []
        contract = entry.agent_closeout_contract
        for evidence_file in contract.evidence_files if contract is not None else []:
            normalized = manifest_write_scope_path(evidence_file)
            if normalized is None:
                continue
            if normalized == normalize_repo_path(entry.expected_closeout_path):
                continue
            path = REPO_ROOT / normalized
            if not path.exists():
                errors.append(f"{entry.milestone_id}: required handoff evidence is missing: {normalized}")
            else:
                evidence_paths.append(path)
        if not evidence_paths:
            errors.append(f"{entry.milestone_id}: handoff closeout requires existing evidence files before closeout")
        return evidence_paths, errors

    evidence_paths = [plan_path]
    errors: list[str] = []
    if not plan_path.exists():
        errors.append(f"{entry.milestone_id}: required production plan evidence is missing: {repo_path(plan_path)}")
    elif entry.milestone_type in {"docs_only", "design_only"}:
        plan_text = plan_path.read_text(encoding="utf-8")
        missing_markers = [marker for marker in required_agent_design_evidence_markers(entry) if marker not in plan_text]
        if missing_markers:
            errors.append(
                f"{entry.milestone_id}: implementation/design plan is missing closeout evidence markers: "
                + ", ".join(missing_markers)
            )
    return evidence_paths, errors


def bounded_contract_known_gaps(entry: TrackExecutionManifestMilestone) -> list[str]:
    return [
        f"{entry.milestone_id} is a bounded design/governance closeout, not runtime_proven evidence.",
        "No product/runtime code, crates, placeholder future folders, downstream implementation, or shared foundation/meta extraction was performed.",
        "Later production-track milestones still require their own WRs, production plans, validation, and closeout evidence.",
    ]


def agent_closeout_known_gaps(entry: TrackExecutionManifestMilestone) -> list[str]:
    if entry.milestone_type == "closeout" and entry.closeout_strategy == "handoff_closeout":
        return [
            f"{entry.milestone_id} is a handoff closeout for the completed production track, not authorization for downstream implementation.",
            "The second-domain proof path remains planning-only until its own WR, manifest, plan, validation, and closeout gates exist.",
            "No product/runtime code, crates, placeholder future folders, downstream implementation, or shared foundation/meta extraction was performed.",
        ]
    return bounded_contract_known_gaps(entry)


def closeout_report_content(
    *,
    track_id: str,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    roadmap_item: RoadmapItem,
    closeout_path: Path,
    evidence_paths: list[Path],
) -> str:
    completion_quality = agent_closeout_completion_quality(entry)
    is_handoff = entry.milestone_type == "closeout" and entry.closeout_strategy == "handoff_closeout"
    evidence_lines = "\n".join(f"- `{repo_path(path)}`" for path in evidence_paths)
    validation_lines = "\n".join(f"- `{command}`" for command in entry.validation_commands)
    forbidden_lines = "\n".join(f"- {scope}" for scope in entry.forbidden_scope)
    gap_lines = "\n".join(f"- {gap}" for gap in agent_closeout_known_gaps(entry))
    next_action = f"After this closeout, rerun `task ai:goal -- --track {track_id}` and continue only to the next manifest legal action."
    closeout_label = "handoff" if is_handoff else "bounded-contract"
    authority_sentence = (
        "This closeout records the final runtime-proven UI proof handoff. It does not authorize downstream implementation, crate creation, placeholder future folders, or shared `foundation/meta` extraction."
        if is_handoff
        else "This closeout does not authorize product/runtime code, crate creation, placeholder future folders, downstream implementation, or shared `foundation/meta` extraction."
    )
    return "\n".join(
        [
            "---",
            f"title: {entry.milestone_id} {entry.title} Closeout",
            f"description: {closeout_label.title()} closeout for {entry.milestone_id} / {roadmap_item.id}.",
            "status: completed",
            "owner: ui",
            "layer: workspace / domain-ui",
            "canonical: false",
            f"last_reviewed: {date.today().isoformat()}",
            "related_reports:",
            f"  - ../../implementation-plans/{roadmap_item.id.lower()}-{slugify(roadmap_item.title)}/plan.md",
            f"  - ../../track-execution-manifests/{track_id.lower()}/manifest.md",
            "related_roadmaps:",
            "  - ../../../workspace/production-tracks.yaml",
            "  - ../../../workspace/roadmap-archive.yaml",
            f"  - ../../../workspace/track-execution-manifests/{track_id.lower()}.yaml",
            "---",
            "",
            f"# {entry.milestone_id} {entry.title} Closeout",
            "",
            "## Summary",
            "",
            f"`{entry.milestone_id}` / `{roadmap_item.id}` is closed as `{completion_quality}` {closeout_label} evidence for `{track_id}`.",
            "",
            authority_sentence,
            "",
            "## Authority",
            "",
            f"- Milestone id: `{entry.milestone_id}`",
            f"- WR id: `{roadmap_item.id}`",
            f"- Authority level: `{entry.authority_level}`",
            f"- Milestone type: `{entry.milestone_type}`",
            f"- Production milestone kind/state before closeout: `{milestone.kind}` / `{milestone.state}`",
            f"- Completion quality: `{completion_quality}`",
            "",
            "## Evidence Files",
            "",
            evidence_lines,
            f"- `{repo_path(closeout_path)}`",
            "",
            "## Validation Commands",
            "",
            validation_lines,
            "",
            "The Manifest Runner executes these commands after writing closeout, production, roadmap, and manifest state. Command output records the exit codes.",
            "",
            "## Forbidden Scope Preserved",
            "",
            forbidden_lines,
            "",
            "No product/runtime source files, crates, placeholder folders, downstream implementation, or shared foundation/meta extraction were created or modified by this closeout.",
            "",
            "## Known Gaps",
            "",
            gap_lines,
            "",
            "## Next Legal Action",
            "",
            next_action,
            "",
            "The next milestone may not start design authoring or implementation inside this closeout action.",
            "",
        ]
    )


def updated_production_data_after_agent_closeout(
    production_source: Path,
    *,
    milestone_id: str,
    closeout_path: Path,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = load_yaml(production_source)
    changed = False
    closeout_repo_path = repo_path(closeout_path)
    for track_data in data.get("tracks", []):
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") != milestone_id:
                continue
            completion_quality = agent_closeout_completion_quality(entry)
            milestone_data["state"] = "completed"
            milestone_data["completion_quality"] = completion_quality
            milestone_data["known_quality_gaps"] = agent_closeout_known_gaps(entry)
            milestone_data["completion_audit"] = closeout_repo_path
            evidence_gates = milestone_data.setdefault("evidence_gates", [])
            if not any(gate.get("path") == closeout_repo_path for gate in evidence_gates):
                evidence_gates.append(
                    {
                        "path": closeout_repo_path,
                        "required_status": "completed",
                        "reason": f"{milestone_id} requires completed {completion_quality} closeout evidence.",
                    }
                )
            if all(candidate.get("state") == "completed" for candidate in track_data.get("milestones", [])):
                track_data["state"] = "completed"
            changed = True
            break
        if changed:
            break
    if not changed:
        raise WorkflowError(f"{milestone_id}: not found in production source")
    return data


def updated_roadmap_sources_after_agent_closeout(
    roadmap_source: Path,
    *,
    track_id: str,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
    closeout_path: Path,
) -> tuple[Path, dict, Path, dict, Path, dict | None]:
    active_data = load_yaml(roadmap_source)
    archive_source, deferred_source = split_source_paths(roadmap_source)
    archive_data = load_yaml(archive_source) if archive_source.exists() else empty_split_source_like(active_data)
    deferred_data = load_yaml(deferred_source) if deferred_source.exists() else empty_split_source_like(active_data)
    if any(item.get("id") == wr_id for item in archive_data.get("items", [])):
        raise WorkflowError(f"{wr_id}: already present in archive roadmap source")

    item_data: dict | None = None
    active_changed = False
    active_items = active_data.get("items", [])
    deferred_items = deferred_data.get("items", [])
    for source_name, source_items in (("deferred", deferred_items), ("active", active_items)):
        for index, candidate in enumerate(list(source_items)):
            if candidate.get("id") == wr_id:
                item_data = source_items.pop(index)
                active_changed = source_name == "active"
                break
        if item_data is not None:
            break
    if item_data is None:
        raise WorkflowError(f"{wr_id}: not present in active or deferred roadmap source")

    closeout_repo_path = repo_path(closeout_path)
    write_scopes = item_data.setdefault("write_scopes", [])
    if closeout_repo_path not in write_scopes:
        write_scopes.append(closeout_repo_path)
    completion_quality = agent_closeout_completion_quality(entry)
    known_gaps = agent_closeout_known_gaps(entry)
    item_data["gate"] = "Completed"
    item_data["planning_state"] = "completed"
    item_data["next_evidence"] = f"Completed through {closeout_repo_path}."
    item_data["current_decision"] = (
        f"Completed at {completion_quality} by Manifest Runner V3 agent_closeout for {entry.milestone_id}. "
        "This is closeout/handoff evidence only."
    )
    item_data["current_call"] = (
        f"Complete. Preserve as {completion_quality} closeout evidence; continue {track_id} only through the next manifest legal action."
    )
    item_data["first_move"] = f"Completed; run task ai:goal -- --track {track_id} for the next legal milestone."
    item_data["main_blocker"] = "Complete."
    item_data["why_not_ready"] = ""
    item_data["completion_quality"] = completion_quality
    item_data["known_quality_gaps"] = known_gaps
    item_data["completion_audit"] = closeout_repo_path
    item_data["diagram_call"] = [completion_quality.replace("_", " "), "no implementation"]
    archive_data.setdefault("items", []).append(item_data)
    return archive_source, archive_data, deferred_source, deferred_data, roadmap_source, active_data if active_changed else None


def updated_manifest_data_after_agent_closeout(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    milestones = data["milestones"]
    current_index = next(
        index for index, milestone_data in enumerate(milestones) if milestone_data["milestone_id"] == entry.milestone_id
    )
    next_milestone = milestones[current_index + 1] if current_index + 1 < len(milestones) else None
    if next_milestone is None:
        next_action = f"{entry.milestone_id} is complete; run track closeout if all milestones are complete."
    elif next_milestone.get("owning_wr"):
        next_action = (
            f"Continue to {next_milestone['milestone_id']} {next_milestone['title']} only through its owning WR "
            f"{next_milestone['owning_wr']} and bounded plan."
        )
    else:
        next_action = (
            f"Create or link the design WR for {next_milestone['milestone_id']} {next_milestone['title']}; "
            "stop before design authoring until that WR and plan exist."
        )
    data["next_legal_action"] = next_action
    for milestone_data in milestones:
        if milestone_data["milestone_id"] == entry.milestone_id:
            completion_quality = agent_closeout_completion_quality(entry)
            milestone_data["next_legal_action"] = (
                f"{entry.milestone_id} completed by agent_closeout as {completion_quality}; "
                "continue only to the next manifest legal action."
            )
            milestone_data["stop_conditions"] = [
                condition
                for condition in milestone_data["stop_conditions"]
                if "agent_closeout is not implemented" not in condition
                and "stop before closeout unless agent_closeout" not in condition
            ]
            milestone_data["stop_conditions"].append(f"completed by agent_closeout as {completion_quality}")
        elif next_milestone is not None and milestone_data["milestone_id"] == next_milestone["milestone_id"]:
            milestone_data["next_legal_action"] = next_action
    return data


def update_manifest_report_after_agent_closeout(path: Path) -> None:
    if not path.exists():
        return
    text = path.read_text(encoding="utf-8")
    marker = "## Agent Closeout Update"
    section = "\n".join(
        [
            marker,
            "",
            f"Last agent closeout update: {date.today().isoformat()}.",
            "",
            "The machine-readable manifest remains execution authority.",
            "",
        ]
    )
    path.write_text(
        upsert_markdown_section(text, heading=marker, section=section, before_heading="## Milestone Details"),
        encoding="utf-8",
        newline="\n",
    )


def validate_agent_closeout_data(
    *,
    production_data: dict,
    manifest_data: dict,
    active_roadmap_data: dict,
    archive_roadmap_data: dict,
    deferred_roadmap_data: dict,
    roadmap_source: Path,
    manifest_path: Path,
    track_id: str,
) -> None:
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            active_roadmap_data,
            roadmap_source,
            archive_data=archive_roadmap_data,
            deferred_data=deferred_roadmap_data,
        )
    )
    manifest = TrackExecutionManifest.model_validate(manifest_data)
    loaded = LoadedTrackExecutionManifest(manifest=manifest, path=manifest_path)
    track = find_track(planning, track_id)
    audit_manifest_or_raise(loaded, track=track, roadmap=roadmap)


def apply_agent_closeout(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    allow: set[str],
    deny: set[str],
    run_validations: bool = True,
) -> AgentCloseoutResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    workflow_action, blockers = next_action_blockers(
        entry,
        milestone,
        planning=context.planning,
        track=context.track,
        roadmap=context.roadmap,
    )
    closeout_blockers = [blocker for blocker in blockers if "Track Expansion must create or link" not in blocker]
    if closeout_blockers:
        raise WorkflowError("\n".join(closeout_blockers))
    if workflow_action != "design_first":
        raise WorkflowError(f"{entry.milestone_id}: next legal action is {workflow_action}, not agent_closeout")
    assert_agent_closeout_allowed(entry, milestone, allow=allow, deny=deny)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")

    plan_path = default_contract_path(roadmap_item)
    closeout_path = REPO_ROOT / entry.expected_closeout_path
    archive_source, deferred_source = split_source_paths(roadmap_source)
    manifest_report = REPO_ROOT / manifest_report_path(context.track.id)
    write_paths = [
        repo_path(production_source),
        repo_path(context.loaded.path),
        repo_path(manifest_report),
        repo_path(archive_source),
        repo_path(deferred_source),
        repo_path(closeout_path),
    ]
    assert_runner_write_scope(entry=entry, roadmap_item=roadmap_item, write_paths=write_paths, action_label="agent_closeout")
    evidence_paths, evidence_errors = agent_closeout_evidence_paths(
        entry=entry,
        roadmap_item=roadmap_item,
        plan_path=plan_path,
    )
    if evidence_errors:
        raise WorkflowError("\n".join(evidence_errors))

    closeout_path.parent.mkdir(parents=True, exist_ok=True)
    closeout_path.write_text(
        closeout_report_content(
            track_id=context.track.id,
            entry=entry,
            milestone=milestone,
            roadmap_item=roadmap_item,
            closeout_path=closeout_path,
            evidence_paths=evidence_paths,
        ),
        encoding="utf-8",
        newline="\n",
    )
    production_data = updated_production_data_after_agent_closeout(
        production_source,
        milestone_id=entry.milestone_id,
        closeout_path=closeout_path,
        entry=entry,
    )
    archive_source, archive_data, deferred_source, deferred_data, active_source, active_data = (
        updated_roadmap_sources_after_agent_closeout(
            roadmap_source,
            track_id=context.track.id,
            wr_id=entry.owning_wr,
            entry=entry,
            closeout_path=closeout_path,
        )
    )
    manifest_data = updated_manifest_data_after_agent_closeout(context.loaded, entry=entry)
    validate_agent_closeout_data(
        production_data=production_data,
        manifest_data=manifest_data,
        active_roadmap_data=active_data if active_data is not None else load_yaml(roadmap_source),
        archive_roadmap_data=archive_data,
        deferred_roadmap_data=deferred_data,
        roadmap_source=roadmap_source,
        manifest_path=context.loaded.path,
        track_id=context.track.id,
    )

    write_yaml_mapping(production_source, production_data)
    if active_data is not None:
        write_yaml_mapping(active_source, active_data, indent_sequences=False)
    write_yaml_mapping(archive_source, archive_data, indent_sequences=False)
    write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)
    write_yaml_mapping(context.loaded.path, manifest_data)
    update_manifest_report_after_agent_closeout(manifest_report)
    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    next_legal_action = str(manifest_data["next_legal_action"])
    return AgentCloseoutResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        closeout_path=closeout_path,
        manifest_path=context.loaded.path,
        production_source=production_source,
        roadmap_archive_source=archive_source,
        roadmap_deferred_source=deferred_source,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
    )


def runtime_proven_known_gaps(entry: TrackExecutionManifestMilestone) -> list[str]:
    return [
        f"{entry.milestone_id} is runtime_proven only for its bounded manifest write scope.",
        "Later milestones in this production track still require separate WRs, plans, validation, and closeout evidence.",
        "No crate creation or shared foundation/meta extraction is authorized by this closeout.",
    ]


def runtime_closeout_report_content(
    *,
    track_id: str,
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    roadmap_item: RoadmapItem,
    closeout_path: Path,
    plan_path: Path,
    product_validation_results: tuple[str, ...],
) -> str:
    scoped_file_sources = (
        entry.runtime_closeout_contract.files_changed_report
        if entry.runtime_closeout_contract is not None
        else entry.write_scope
    )
    scoped_files = [
        normalized
        for normalized in (manifest_write_scope_path(scope) for scope in scoped_file_sources)
        if normalized is not None
    ]
    evidence_categories = sorted(
        {
            normalized
            for normalized in (normalize_evidence_category(category) for category in entry.required_evidence_categories)
            if normalized in GENERIC_EVIDENCE_CATEGORIES
        }
    )
    if not evidence_categories:
        evidence_categories = ["runtime_test"]
    frontmatter = {
        "title": f"{entry.milestone_id} {entry.title} Runtime Closeout",
        "description": f"Runtime-proof closeout for {entry.milestone_id} / {roadmap_item.id}.",
        "status": "completed",
        "owner": roadmap_item.ddd_owner,
        "layer": "production-track",
        "canonical": False,
        "last_reviewed": date.today().isoformat(),
        "closeout_evidence": {
            "milestone_id": entry.milestone_id,
            "wr_id": roadmap_item.id,
            "completion_quality": "runtime_proven",
            "evidence_categories": evidence_categories,
            "validation_commands": product_validation_commands_for_entry(entry),
            "validation_results": list(product_validation_results),
            "files_changed": scoped_files or ["No scoped runtime files were declared."],
            "runtime_artifacts": [category for category in evidence_categories if category == "artifact"],
            "diagnostics": [category for category in evidence_categories if category == "diagnostics"],
            "source_maps": [category for category in evidence_categories if category == "source_maps"],
            "known_gaps": runtime_proven_known_gaps(entry),
            "closeout_path": repo_path(closeout_path),
        },
        "related_reports": [
            f"../../implementation-plans/{roadmap_item.id.lower()}-{slugify(roadmap_item.title)}/plan.md",
            f"../../track-execution-manifests/{track_id.lower()}/manifest.md",
        ],
        "related_roadmaps": [
            "../../../workspace/production-tracks.yaml",
            "../../../workspace/roadmap-archive.yaml",
            f"../../../workspace/track-execution-manifests/{track_id.lower()}.yaml",
        ],
    }
    frontmatter_text = yaml.safe_dump(frontmatter, sort_keys=False, allow_unicode=False).strip()
    scoped_file_lines = "\n".join(f"- `{path}`" for path in scoped_files)
    validation_lines = "\n".join(f"- `{result}`" for result in product_validation_results)
    forbidden_lines = "\n".join(f"- {scope}" for scope in entry.forbidden_scope)
    gap_lines = "\n".join(f"- {gap}" for gap in runtime_proven_known_gaps(entry))
    next_action = f"After this closeout, rerun `task ai:goal -- --track {track_id}` and continue only to the next manifest legal action."
    return "\n".join(
        [
            "---",
            frontmatter_text,
            "---",
            "",
            f"# {entry.milestone_id} {entry.title} Runtime Closeout",
            "",
            "## Summary",
            "",
            f"`{entry.milestone_id}` / `{roadmap_item.id}` is closed as `runtime_proven` evidence for `{track_id}`.",
            "",
            "This closeout records runtime/test validation evidence for the bounded manifest scope only. It does not authorize crate creation, unrelated downstream implementation, or shared `foundation/meta` extraction.",
            "",
            "## Authority",
            "",
            f"- Milestone id: `{entry.milestone_id}`",
            f"- WR id: `{roadmap_item.id}`",
            f"- Authority level: `{entry.authority_level}`",
            f"- Milestone type: `{entry.milestone_type}`",
            f"- Production milestone kind/state before closeout: `{milestone.kind}` / `{milestone.state}`",
            "- Completion quality: `runtime_proven`",
            "",
            "## Files Changed / Scoped Evidence",
            "",
            scoped_file_lines or "- No scoped runtime files were declared.",
            f"- Closeout report: `{repo_path(closeout_path)}`",
            f"- Accepted plan: `{repo_path(plan_path)}`",
            "",
            "## Tests Run",
            "",
            validation_lines,
            "",
            "## Evidence",
            "",
            "- Product/runtime validation commands completed successfully.",
            "- Every exact manifest runtime evidence path existed before closeout.",
            f"- Closeout evidence is recorded at `{repo_path(closeout_path)}`.",
            "",
            "## Forbidden Scope Preserved",
            "",
            forbidden_lines,
            "",
            "## Known Gaps",
            "",
            gap_lines,
            "",
            "## Next Legal Action",
            "",
            next_action,
            "",
        ]
    )


def assert_runtime_closeout_allowed(
    entry: TrackExecutionManifestMilestone,
    milestone: ProductionMilestone,
    roadmap_item: RoadmapItem,
    *,
    closeout_path: Path,
    product_validation_results: tuple[str, ...],
    allow: set[str],
) -> None:
    errors: list[str] = []
    if "agent_closeout" not in allow:
        errors.append("runtime closeout requires --allow agent_closeout")
    if entry.milestone_type not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: runtime closeout supports implementation or hardening milestones only")
    if milestone.kind not in {"implementation", "hardening"}:
        errors.append(f"{entry.milestone_id}: production milestone kind {milestone.kind!r} cannot close as runtime proof")
    if not product_validation_results:
        errors.append(f"{entry.milestone_id}: runtime closeout requires product/runtime validation results")
    if closeout_path.suffix != ".md":
        errors.append(f"{entry.milestone_id}: runtime closeout path must be Markdown")
    if any("foundation/meta" in scope.lower() for scope in product_implementation_scopes_for_entry(entry)):
        errors.append(f"{entry.milestone_id}: runtime closeout cannot authorize shared foundation/meta extraction")
    errors.extend(runtime_evidence_errors(entry))
    wr_scopes = normalized_write_scopes_with_generated_outputs(roadmap_item.write_scopes)
    expected_closeout = normalize_repo_path(entry.expected_closeout_path)
    product_paths = [
        normalized
        for normalized in (manifest_write_scope_path(scope) for scope in product_implementation_scopes_for_entry(entry))
        if normalized is not None and normalized != expected_closeout
    ]
    missing_wr = [path for path in product_paths if not path_is_covered_by_scope(path, wr_scopes)]
    if missing_wr:
        errors.append(
            f"{entry.milestone_id}: runtime closeout product evidence paths are not covered by owning WR {roadmap_item.id} write_scopes: "
            + ", ".join(missing_wr)
        )
    if errors:
        raise WorkflowError("\n".join(errors))


def updated_production_data_after_runtime_closeout(
    production_source: Path,
    *,
    milestone_id: str,
    closeout_path: Path,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = load_yaml(production_source)
    changed = False
    closeout_repo_path = repo_path(closeout_path)
    for track_data in data.get("tracks", []):
        for milestone_data in track_data.get("milestones", []):
            if milestone_data.get("id") != milestone_id:
                continue
            milestone_data["state"] = "completed"
            milestone_data["completion_quality"] = "runtime_proven"
            milestone_data["known_quality_gaps"] = runtime_proven_known_gaps(entry)
            milestone_data["completion_audit"] = closeout_repo_path
            evidence_gates = milestone_data.setdefault("evidence_gates", [])
            if not any(gate.get("path") == closeout_repo_path for gate in evidence_gates):
                evidence_gates.append(
                    {
                        "path": closeout_repo_path,
                        "required_status": "completed",
                        "reason": f"{milestone_id} requires completed runtime-proof closeout evidence.",
                    }
                )
            if all(candidate.get("state") == "completed" for candidate in track_data.get("milestones", [])):
                track_data["state"] = "completed"
            changed = True
            break
        if changed:
            break
    if not changed:
        raise WorkflowError(f"{milestone_id}: not found in production source")
    return data


def updated_roadmap_sources_after_runtime_closeout(
    roadmap_source: Path,
    *,
    track_id: str,
    wr_id: str,
    entry: TrackExecutionManifestMilestone,
    closeout_path: Path,
) -> tuple[Path, dict, Path, dict, Path, dict | None]:
    active_data = load_yaml(roadmap_source)
    archive_source, deferred_source = split_source_paths(roadmap_source)
    archive_data = load_yaml(archive_source) if archive_source.exists() else empty_split_source_like(active_data)
    deferred_data = load_yaml(deferred_source) if deferred_source.exists() else empty_split_source_like(active_data)
    if any(item.get("id") == wr_id for item in archive_data.get("items", [])):
        raise WorkflowError(f"{wr_id}: already present in archive roadmap source")

    item_data: dict | None = None
    active_changed = False
    active_items = active_data.get("items", [])
    deferred_items = deferred_data.get("items", [])
    for source_name, source_items in (("active", active_items), ("deferred", deferred_items)):
        for index, candidate in enumerate(list(source_items)):
            if candidate.get("id") == wr_id:
                item_data = source_items.pop(index)
                active_changed = source_name == "active"
                break
        if item_data is not None:
            break
    if item_data is None:
        raise WorkflowError(f"{wr_id}: not present in active or deferred roadmap source")

    closeout_repo_path = repo_path(closeout_path)
    write_scopes = item_data.setdefault("write_scopes", [])
    if closeout_repo_path not in write_scopes:
        write_scopes.append(closeout_repo_path)
    item_data["gate"] = "Completed"
    item_data["planning_state"] = "completed"
    item_data["next_evidence"] = f"Runtime proof completed through {closeout_repo_path}."
    item_data["current_decision"] = (
        f"Completed at runtime_proven by Manifest Runner runtime closeout for {entry.milestone_id}."
    )
    item_data["current_call"] = (
        f"Complete. Preserve as bounded runtime proof evidence; continue {track_id} only through the next manifest legal action."
    )
    item_data["first_move"] = f"Completed; run task ai:goal -- --track {track_id} for the next legal milestone."
    item_data["main_blocker"] = "Complete; later milestones require separate WRs, plans, validation, and closeout evidence."
    item_data["why_not_ready"] = ""
    item_data["completion_quality"] = "runtime_proven"
    item_data["known_quality_gaps"] = runtime_proven_known_gaps(entry)
    item_data["completion_audit"] = closeout_repo_path
    item_data["diagram_call"] = ["runtime proven", "bounded proof"]
    archive_data.setdefault("items", []).append(item_data)
    return archive_source, archive_data, deferred_source, deferred_data, roadmap_source, active_data if active_changed else None


def updated_manifest_data_after_runtime_closeout(
    loaded: LoadedTrackExecutionManifest,
    *,
    entry: TrackExecutionManifestMilestone,
) -> dict:
    data = loaded.manifest.model_dump(exclude_none=True, mode="json")
    milestones = data["milestones"]
    current_index = next(
        index for index, milestone_data in enumerate(milestones) if milestone_data["milestone_id"] == entry.milestone_id
    )
    next_milestone = milestones[current_index + 1] if current_index + 1 < len(milestones) else None
    if next_milestone is None:
        next_action = f"{entry.milestone_id} is complete; all manifest milestones are complete."
    elif next_milestone.get("owning_wr"):
        next_action = (
            f"Continue to {next_milestone['milestone_id']} {next_milestone['title']} only through its owning WR "
            f"{next_milestone['owning_wr']} and bounded plan."
        )
    else:
        next_action = (
            f"Create or link the WR for {next_milestone['milestone_id']} {next_milestone['title']}; "
            "stop before implementation until that WR and plan exist."
        )
    data["next_legal_action"] = next_action
    for milestone_data in milestones:
        if milestone_data["milestone_id"] == entry.milestone_id:
            milestone_data["next_legal_action"] = (
                f"{entry.milestone_id} completed by runtime closeout as runtime_proven; "
                "continue only to the next manifest legal action."
            )
            milestone_data["stop_conditions"] = [
                condition
                for condition in milestone_data["stop_conditions"]
                if "stop before product_code" not in condition
                and "product_code unless explicitly allowed" not in condition
            ]
            milestone_data["stop_conditions"].append("completed by runtime closeout as runtime_proven")
        elif next_milestone is not None and milestone_data["milestone_id"] == next_milestone["milestone_id"]:
            milestone_data["next_legal_action"] = next_action
    return data


def validate_runtime_closeout_data(
    *,
    production_data: dict,
    manifest_data: dict,
    active_roadmap_data: dict,
    archive_roadmap_data: dict,
    deferred_roadmap_data: dict,
    roadmap_source: Path,
    manifest_path: Path,
    track_id: str,
) -> None:
    planning = ProductionPlanningState.model_validate(production_data)
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            active_roadmap_data,
            roadmap_source,
            archive_data=archive_roadmap_data,
            deferred_data=deferred_roadmap_data,
        )
    )
    manifest = TrackExecutionManifest.model_validate(manifest_data)
    loaded = LoadedTrackExecutionManifest(manifest=manifest, path=manifest_path)
    track = find_track(planning, track_id)
    audit_manifest_or_raise(loaded, track=track, roadmap=roadmap)


def apply_runtime_closeout(
    context: ManifestCommandContext,
    *,
    production_source: Path,
    roadmap_source: Path,
    product_validation_results: tuple[str, ...],
    allow: set[str],
    run_validations: bool = True,
) -> RuntimeCloseoutResult:
    audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
    entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
    assert entry.owning_wr is not None
    roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
    if roadmap_item is None:
        raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
    plan_path = default_contract_path(roadmap_item)
    closeout_path = REPO_ROOT / entry.expected_closeout_path
    archive_source, deferred_source = split_source_paths(roadmap_source)
    assert_runtime_closeout_allowed(
        entry,
        milestone,
        roadmap_item,
        closeout_path=closeout_path,
        product_validation_results=product_validation_results,
        allow=allow,
    )

    closeout_path.parent.mkdir(parents=True, exist_ok=True)
    closeout_path.write_text(
        runtime_closeout_report_content(
            track_id=context.track.id,
            entry=entry,
            milestone=milestone,
            roadmap_item=roadmap_item,
            closeout_path=closeout_path,
            plan_path=plan_path,
            product_validation_results=product_validation_results,
        ),
        encoding="utf-8",
        newline="\n",
    )
    production_data = updated_production_data_after_runtime_closeout(
        production_source,
        milestone_id=entry.milestone_id,
        closeout_path=closeout_path,
        entry=entry,
    )
    archive_source, archive_data, deferred_source, deferred_data, active_source, active_data = (
        updated_roadmap_sources_after_runtime_closeout(
            roadmap_source,
            track_id=context.track.id,
            wr_id=entry.owning_wr,
            entry=entry,
            closeout_path=closeout_path,
        )
    )
    manifest_data = updated_manifest_data_after_runtime_closeout(context.loaded, entry=entry)
    validate_runtime_closeout_data(
        production_data=production_data,
        manifest_data=manifest_data,
        active_roadmap_data=active_data if active_data is not None else load_yaml(roadmap_source),
        archive_roadmap_data=archive_data,
        deferred_roadmap_data=deferred_data,
        roadmap_source=roadmap_source,
        manifest_path=context.loaded.path,
        track_id=context.track.id,
    )

    write_yaml_mapping(production_source, production_data)
    if active_data is not None:
        write_yaml_mapping(active_source, active_data, indent_sequences=False)
    write_yaml_mapping(archive_source, archive_data, indent_sequences=False)
    write_yaml_mapping(deferred_source, deferred_data, indent_sequences=False)
    write_yaml_mapping(context.loaded.path, manifest_data)
    validation_results = run_validation_commands(auto_safe_validation_commands()) if run_validations else ()
    next_legal_action = str(manifest_data["next_legal_action"])
    return RuntimeCloseoutResult(
        track_id=context.track.id,
        milestone_id=entry.milestone_id,
        wr_id=entry.owning_wr,
        closeout_path=closeout_path,
        manifest_path=context.loaded.path,
        production_source=production_source,
        roadmap_archive_source=archive_source,
        roadmap_deferred_source=deferred_source,
        validation_commands=validation_results,
        next_legal_action=next_legal_action,
    )


def print_errors(title: str, errors: list[str]) -> None:
    console.print(f"[red]{title}[/red]")
    for error in errors:
        console.print(f"- {error}")


@app.command("plan-track")
def plan_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
    force: bool = typer.Option(False, "--force", help="Overwrite an existing manifest scaffold. Use only for manual repair."),
) -> None:
    try:
        planning = load_production_tracks(production_source)
        roadmap = load_roadmap(roadmap_source)
        production_track = find_track(planning, track)
        path = manifest_source_path(track, root=manifest_source_root)
        if path.exists() and not force:
            loaded = load_track_execution_manifest(track, root=manifest_source_root)
            assert loaded is not None
            errors = audit_manifest(loaded, track=production_track, roadmap=roadmap)
            console.print(f"Track Execution Manifest already exists: {repo_path(path)}")
            console.print("No implementation authority is created by this command.")
            if errors:
                print_manifest_audit_blockers(errors)
                raise typer.Exit(1)
            console.print("[green]manifest audit passed[/green]")
            return
        manifest = build_manifest_scaffold(production_track, roadmap)
        write_manifest(path, manifest)
        console.print(f"Wrote conservative Track Execution Manifest scaffold: {repo_path(path)}")
        console.print("The scaffold is planning-only and contains blocked fields until reviewed.")
        console.print("No implementation authority is created by this command.")
    except WorkflowError as error:
        console.print("[red]production:plan-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("expand-track")
def expand_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
) -> None:
    try:
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        errors = audit_manifest(context.loaded, track=context.track, roadmap=context.roadmap)
        if errors:
            print_manifest_audit_blockers(errors)
            raise typer.Exit(1)
        candidates = [entry for entry in context.loaded.manifest.milestones if entry.future_wr_candidate]
        console.print(f"Track Expansion candidates for {track}:")
        for entry in candidates:
            console.print(f"- {entry.future_wr_candidate}: {entry.milestone_id} - {entry.title}")
        console.print("production:expand-track is read-only; run production:run-track -- --allow auto_safe for the guarded V1 mutation path.")
        console.print("No implementation authority is created by this command.")
    except WorkflowError as error:
        console.print("[red]production:expand-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("complete-track-contracts")
def complete_track_contracts_command(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
) -> None:
    try:
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        result = complete_track_contracts(
            context,
            production_source=production_source,
            roadmap_source=roadmap_source,
            run_validations=True,
        )
        console.print("[green]Track contract completion finished.[/green]")
        console.print(f"Manifest: {repo_path(result.manifest_path)}")
        console.print(f"Manifest report: {repo_path(result.manifest_report_path)}")
        if result.completed_milestones:
            console.print("Completed contract blocks:")
            for milestone_id in result.completed_milestones:
                console.print(f"- {milestone_id}")
        else:
            console.print("Completed contract blocks: none")
        if result.remaining_blockers:
            console.print("[red]Remaining contract blockers:[/red]")
            for blocker in result.remaining_blockers:
                console.print(f"- {blocker}")
            raise typer.Exit(1)
        if result.validation_commands:
            console.print("Validation commands:")
            for command_result in result.validation_commands:
                console.print(f"- {command_result}")
        console.print("No implementation authority is created by this command.")
    except WorkflowError as error:
        console.print("[red]production:complete-track-contracts failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("lock-track")
def lock_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    locked_by: str = typer.Option("human", "--locked-by", help="Identity or role that locked the track for AI execution."),
    allow: list[str] = typer.Option(
        list(FULL_TRACK_PERMISSION_SET),
        "--allow",
        help="Permission tier to grant in the execution lock.",
    ),
    deny: list[str] = typer.Option(
        ["crate_creation", "foundation_extraction"],
        "--deny",
        help="Permission tier to deny in the execution lock.",
    ),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
    lock_source_root: Path = typer.Option(TRACK_EXECUTION_LOCK_ROOT, help="Track Execution Lock source root."),
) -> None:
    try:
        allow_set = set(allow)
        deny_set = set(deny)
        unknown_permissions = sorted((allow_set | deny_set) - MANIFEST_RUNNER_PERMISSIONS)
        if unknown_permissions:
            raise WorkflowError(f"unknown Track Execution Lock permissions: {', '.join(unknown_permissions)}")
        if allow_set & deny_set:
            raise WorkflowError("the same permission cannot be both granted and denied by a Track Execution Lock")
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        full_automation_preflight_or_raise(
            context.loaded,
            track=context.track,
            roadmap=context.roadmap,
            allow=allow_set,
        )
        data = build_track_execution_lock_data(
            context.loaded,
            production_source=production_source,
            roadmap_source=roadmap_source,
            locked_by=locked_by,
            granted_permissions=sorted(allow_set),
            denied_permissions=sorted(deny_set),
        )
        lock = TrackExecutionLock.model_validate(data)
        path = lock_source_path(track, root=lock_source_root)
        write_yaml_mapping(path, lock.model_dump(mode="json"))
        console.print(f"[green]Track Execution Lock written.[/green]")
        console.print(f"Lock: {repo_path(path)}")
        console.print("The lock grants full-track execution only while source digests remain unchanged.")
    except WorkflowError as error:
        console.print("[red]production:lock-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("run-track")
def run_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    allow: list[str] = typer.Option(
        [],
        "--allow",
        help="Permission tier to allow. Supported: auto_safe, agent_design, agent_closeout, product_code, product_implementation, crate_creation, foundation_extraction.",
    ),
    deny: list[str] = typer.Option(
        [],
        "--deny",
        help="Permission tier to deny explicitly, for example product_code.",
    ),
    max_actions: int = typer.Option(1, "--max-actions", min=1, help="Maximum mechanical actions before stopping."),
    mode: str = typer.Option(
        "bounded-segment",
        "--mode",
        help="Automation mode: single-step, bounded-segment, full-track, or agent-track.",
    ),
    preflight_only: bool = typer.Option(
        False,
        "--preflight-only",
        help="Run full automation readiness preflight for the remaining track and exit without mutation.",
    ),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
    lock_source_root: Path = typer.Option(TRACK_EXECUTION_LOCK_ROOT, help="Track Execution Lock source root."),
    contract_pack_root: Path = typer.Option(EXECUTION_CONTRACT_PACK_ROOT, help="Execution Contract Pack root."),
    run_ledger_root: Path = typer.Option(TRACK_EXECUTION_RUN_ROOT, help="Track Execution Run ledger root."),
) -> None:
    try:
        allow_set = set(allow)
        deny_set = set(deny)
        if mode not in {"single-step", "bounded-segment", "full-track", "agent-track"}:
            raise WorkflowError("--mode must be one of single-step, bounded-segment, full-track, agent-track")
        unknown_permissions = sorted((allow_set | deny_set) - MANIFEST_RUNNER_PERMISSIONS)
        if unknown_permissions:
            raise WorkflowError(f"unknown Manifest Runner permissions: {', '.join(unknown_permissions)}")
        if allow_set & deny_set:
            raise WorkflowError("the same Manifest Runner permission cannot be both allowed and denied")
        if not allow_set:
            raise WorkflowError("Manifest Runner requires at least one --allow permission")
        if "product_implementation" in allow_set and "product_code" not in allow_set:
            raise WorkflowError("product_implementation requires product_code")
        if mode == "single-step" and max_actions != 1:
            raise WorkflowError("--mode single-step requires --max-actions 1")
        if (
            mode not in {"full-track", "agent-track"}
            and max_actions > 1
            and FULL_TRACK_PERMISSION_SET.issubset(allow_set)
        ):
            raise WorkflowError(
                "full-track permission set with --max-actions > 1 requires explicit --mode full-track or --mode agent-track"
            )
        unsupported_allowed = sorted(
            allow_set
            - {
                "auto_safe",
                "agent_design",
                "agent_closeout",
                "product_code",
                "product_implementation",
                "crate_creation",
                "foundation_extraction",
            }
        )
        if unsupported_allowed:
            raise WorkflowError(
                "Manifest Runner does not implement allowed permissions: " + ", ".join(unsupported_allowed)
            )

        if mode == "full-track":
            from execution.cli import preflight_command as execution_preflight_command
            from execution.cli import run_command as execution_run_command
            from execution.compiler import contract_pack_path, load_contract_pack
            from execution.locks import EXECUTION_LOCK_ROOT

            if load_contract_pack(track, root=contract_pack_root) is not None:
                console.print("[cyan]Delegating locked full-track execution to the Track Execution Harness.[/cyan]")
                if preflight_only:
                    execution_preflight_command(track=track, allow=allow, contract_pack_root=contract_pack_root)
                else:
                    execution_run_command(
                        track=track,
                        mode="full-track",
                        allow=allow,
                        deny=deny,
                        max_actions=max_actions,
                        production_source=production_source,
                        roadmap_source=roadmap_source,
                        manifest_source_root=manifest_source_root,
                        contract_pack_root=contract_pack_root,
                        lock_root=lock_source_root,
                        run_ledger_root=run_ledger_root,
                        repo_root=REPO_ROOT,
                    )
                return
            context = resolve_manifest_command_context(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
            )
            if context.loaded.manifest.ai_executable or context.loaded.manifest.full_automation_target:
                raise WorkflowError(
                    f"{track}: executable/full-automation track requires Execution Contract Pack at "
                    f"{repo_path(contract_pack_path(track, root=contract_pack_root))}; legacy fallback is forbidden"
                )
            console.print("[yellow]Track Execution Harness fallback: no Execution Contract Pack exists; using legacy Manifest Runner.[/yellow]")

        if should_run_full_automation_preflight(
            mode=mode,
            preflight_only=preflight_only,
        ):
            context = resolve_manifest_command_context(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
            )
            preflight_errors = full_automation_preflight_errors(
                context.loaded,
                track=context.track,
                roadmap=context.roadmap,
                allow=allow_set,
            )
            loaded_lock: LoadedTrackExecutionLock | None = None
            lock_errors: list[str] = []
            if mode == "full-track":
                loaded_lock = load_track_execution_lock(track, root=lock_source_root)
                lock_errors = track_execution_lock_errors(
                    context.loaded,
                    loaded_lock,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=allow_set,
                    deny=deny_set,
                    track=context.track,
                )
            if preflight_errors or lock_errors:
                lines: list[str] = []
                if preflight_errors:
                    lines.extend(full_automation_blocker_lines(preflight_errors))
                if lock_errors:
                    if lines:
                        lines.append("")
                    lines.extend(["Track Execution Lock blockers:", *[f"- {error}" for error in lock_errors]])
                raise WorkflowError("\n".join(lines))
            if preflight_only:
                remaining = remaining_manifest_entries(context.loaded.manifest, context.track)
                console.print("[green]full automation readiness preflight passed[/green]")
                console.print(f"Manifest: {repo_path(context.loaded.path)}")
                if loaded_lock is not None:
                    console.print(f"Execution lock: {repo_path(loaded_lock.path)}")
                    console.print("Execution lock status: current")
                console.print(f"Remaining milestones inspected: {len(remaining)}")
                for entry, _milestone, _index in remaining:
                    console.print(f"- {entry.milestone_id}: {entry.execution_kind}")
                return

        actions: list[
            AutoSafeExpansionResult
            | AgentDesignResult
            | AgentCloseoutResult
            | ProductCodeResult
            | ProductImplementationResult
            | StopBeforeProductCodeResult
            | StopAtManifestGateResult
            | RuntimeCloseoutResult
        ] = []
        action_entries: list[TrackExecutionManifestMilestone] = []
        run_id = new_track_execution_run_id(track, root=run_ledger_root) if mode in {"full-track", "agent-track"} else ""
        run_ledger_path: Path | None = None

        def record_result(
            action_entry: TrackExecutionManifestMilestone,
            result: object,
            before_digests: dict[str, str],
        ) -> None:
            nonlocal run_ledger_path
            if not run_id:
                return
            after_context = resolve_manifest_command_context(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
            )
            after_digests = source_digest_map(
                after_context.loaded,
                production_source=production_source,
                roadmap_source=roadmap_source,
            )
            run_ledger_path = append_track_execution_run_action(
                track_id=track,
                run_id=run_id,
                run_root=run_ledger_root,
                entry=action_entry,
                result=result,
                before_digests=before_digests,
                after_digests=after_digests,
            )

        def append_action_result(
            action_entry: TrackExecutionManifestMilestone,
            result: object,
            before_digests: dict[str, str],
        ) -> None:
            actions.append(result)  # type: ignore[arg-type]
            action_entries.append(action_entry)
            record_result(action_entry, result, before_digests)
            if mode == "agent-track":
                try_refresh_agent_track_lock(
                    track_id=track,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    manifest_source_root=manifest_source_root,
                    lock_source_root=lock_source_root,
                    allow=allow_set,
                    deny=deny_set,
                )

        track_complete = False
        while len(actions) < max_actions:
            context = resolve_manifest_command_context(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
            )
            audit_manifest_or_raise(context.loaded, track=context.track, roadmap=context.roadmap)
            if all(milestone.state == "completed" for milestone in context.track.milestones):
                track_complete = True
                break
            entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
            before_digests = (
                source_digest_map(
                    context.loaded,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                )
                if run_id
                else {}
            )
            workflow_action, blockers = next_action_blockers(
                entry,
                milestone,
                planning=context.planning,
                track=context.track,
                roadmap=context.roadmap,
            )
            if workflow_action == "track_expansion_required":
                if "auto_safe" not in allow_set:
                    raise WorkflowError(
                        f"{entry.milestone_id}: Track Expansion must create or link {entry.future_wr_candidate}; "
                        "rerun with --allow auto_safe"
                    )
                result = apply_auto_safe_track_expansion(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow={"auto_safe"} if "auto_safe" in allow_set else set(),
                    run_validations=True,
                )
                append_action_result(entry, result, before_digests)
                if "agent_design" not in allow_set:
                    break
                continue
            if workflow_action == "design_first" and agent_closeout_pending(entry):
                if "agent_closeout" not in allow_set:
                    if actions:
                        break
                    raise WorkflowError(
                        f"{entry.milestone_id}: closeout is the next legal action; rerun with --allow agent_closeout"
                    )
                result = apply_agent_closeout(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=allow_set,
                    deny=deny_set,
                    run_validations=True,
                )
                append_action_result(entry, result, before_digests)
                continue
            if workflow_action == "design_first" and "agent_design" in allow_set:
                if agent_design_contract_for_entry(entry) is None:
                    result = StopAtManifestGateResult(
                        track_id=context.track.id,
                        milestone_id=entry.milestone_id,
                        manifest_path=context.loaded.path,
                        reason=f"{entry.milestone_id}: manifest milestone is missing agent_design contract",
                        validation_commands=(),
                        next_legal_action=(
                            f"{entry.milestone_id} requires a bounded agent_design contract before the runner "
                            "can create an implementation plan or run product_code."
                        ),
                    )
                    append_action_result(entry, result, before_digests)
                    break
                result = apply_agent_design(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=allow_set,
                    deny=deny_set,
                    run_validations=True,
                    run_id=run_id,
                    run_ledger_root=run_ledger_root,
                )
                append_action_result(entry, result, before_digests)
                continue
            if workflow_action == "write_implementation_contract":
                assert entry.owning_wr is not None
                roadmap_item = context.roadmap.by_id.get(entry.owning_wr)
                if roadmap_item is None:
                    raise WorkflowError(f"{entry.milestone_id}: owning WR {entry.owning_wr} is not present in roadmap")
                plan_path = default_contract_path(roadmap_item)
                plan_errors = product_plan_contract_errors(
                    entry=entry,
                    roadmap_item=roadmap_item,
                    plan_path=plan_path,
                )
                if plan_errors and "agent_design" in allow_set:
                    result = apply_agent_design(
                        context,
                        production_source=production_source,
                        roadmap_source=roadmap_source,
                        allow=allow_set,
                        deny=deny_set,
                        run_validations=True,
                        allow_regenerate_invalid_implementation_plan=True,
                        run_id=run_id,
                        run_ledger_root=run_ledger_root,
                    )
                    append_action_result(entry, result, before_digests)
                    continue
                if "product_code" not in allow_set:
                    if actions:
                        break
                    if not plan_path.exists():
                        raise WorkflowError(
                            f"{entry.milestone_id}: accepted production plan is missing at {repo_path(plan_path)}"
                        )
                    if plan_errors:
                        raise WorkflowError("\n".join(plan_errors))
                    result = StopBeforeProductCodeResult(
                        track_id=context.track.id,
                        milestone_id=entry.milestone_id,
                        wr_id=entry.owning_wr,
                        plan_path=plan_path,
                        manifest_path=context.loaded.path,
                        validation_commands=(),
                        next_legal_action=(
                            f"{entry.milestone_id} implementation plan exists; product_code is denied. "
                            f"Rerun `task production:run-track -- --track {context.track.id} --allow product_code --allow product_implementation --max-actions 1` "
                            "only if product/runtime code is permitted."
                        ),
                    )
                    append_action_result(entry, result, before_digests)
                    break
                if "product_implementation" in allow_set:
                    if mode == "agent-track" and entry.implementation_writer is not None and entry.implementation_writer.strategy == "agent_writer":
                        try_refresh_agent_track_lock(
                            track_id=track,
                            production_source=production_source,
                            roadmap_source=roadmap_source,
                            manifest_source_root=manifest_source_root,
                            lock_source_root=lock_source_root,
                            allow=allow_set,
                            deny=deny_set,
                        )
                        agent_track_product_lock_or_raise(
                            context,
                            production_source=production_source,
                            roadmap_source=roadmap_source,
                            lock_source_root=lock_source_root,
                            allow=allow_set,
                            deny=deny_set,
                        )
                    result = apply_product_implementation(
                        context,
                        allow=allow_set,
                        roadmap_source=roadmap_source,
                        run_id=run_id,
                        run_ledger_root=run_ledger_root,
                        run_validations=True,
                    )
                else:
                    result = apply_product_code(
                        context,
                        allow=allow_set,
                        run_validations=True,
                    )
                append_action_result(entry, result, before_digests)
                if "agent_closeout" not in allow_set or len(actions) >= max_actions:
                    break
                continue
            if workflow_action == "runtime_closeout":
                if "agent_closeout" not in allow_set:
                    if actions:
                        break
                    raise WorkflowError(
                        f"{entry.milestone_id}: runtime closeout is the next legal action; rerun with --allow agent_closeout"
                    )
                product_validation_results = run_validation_commands(product_validation_commands_for_entry(entry))
                runtime_closeout = apply_runtime_closeout(
                    context,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    product_validation_results=product_validation_results,
                    allow=allow_set,
                    run_validations=True,
                )
                append_action_result(entry, runtime_closeout, before_digests)
                continue
            if blockers:
                raise WorkflowError("\n".join(blockers))
            raise WorkflowError(f"{entry.milestone_id}: workflow action is {workflow_action}; no permitted runner action")

        if not actions:
            if track_complete:
                console.print(f"[green]Track {track} is complete.[/green]")
                return
            raise WorkflowError("Manifest Runner did not apply any action")
        if run_ledger_path is not None:
            console.print(f"Run ledger: {repo_path(run_ledger_path)}")
        for result in actions:
            if isinstance(result, AutoSafeExpansionResult):
                console.print("[green]Manifest Runner V1 applied one auto_safe Track Expansion action.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Created/linked WR: {result.wr_id}")
                console.print(f"Production source: {repo_path(result.production_source)}")
                console.print(f"Roadmap deferred source: {repo_path(result.roadmap_deferred_source)}")
            elif isinstance(result, AgentDesignResult):
                console.print("[green]Manifest Runner V2 applied one agent_design action.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Owning WR: {result.wr_id}")
                console.print(f"Plan path: {repo_path(result.plan_path)}")
                console.print("Design docs:")
                for design_path in result.design_paths:
                    console.print(f"- {repo_path(design_path)}")
                if result.agent_transcript_path is not None:
                    console.print(f"Agent transcript: {repo_path(result.agent_transcript_path)}")
            elif isinstance(result, AgentCloseoutResult):
                console.print("[green]Manifest Runner V3 applied one agent_closeout action.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Closed WR: {result.wr_id}")
                console.print(f"Closeout path: {repo_path(result.closeout_path)}")
                console.print(f"Production source: {repo_path(result.production_source)}")
                console.print(f"Roadmap archive source: {repo_path(result.roadmap_archive_source)}")
                console.print(f"Roadmap deferred source: {repo_path(result.roadmap_deferred_source)}")
            elif isinstance(result, StopBeforeProductCodeResult):
                console.print("[yellow]Manifest Runner stopped before product_code.[/yellow]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Owning WR: {result.wr_id}")
                console.print(f"Plan path: {repo_path(result.plan_path)}")
            elif isinstance(result, StopAtManifestGateResult):
                console.print("[yellow]Manifest Runner stopped at a manifest gate.[/yellow]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Reason: {result.reason}")
            elif isinstance(result, RuntimeCloseoutResult):
                console.print("[green]Manifest Runner runtime closeout completed one implementation milestone.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Closed WR: {result.wr_id}")
                console.print(f"Closeout path: {repo_path(result.closeout_path)}")
                console.print(f"Production source: {repo_path(result.production_source)}")
                console.print(f"Roadmap archive source: {repo_path(result.roadmap_archive_source)}")
                console.print(f"Roadmap deferred source: {repo_path(result.roadmap_deferred_source)}")
            elif isinstance(result, ProductImplementationResult):
                console.print("[green]Manifest Runner V5 wrote one bounded product_implementation slice.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Owning WR: {result.wr_id}")
                console.print(f"Plan path: {repo_path(result.plan_path)}")
                console.print("Written product files:")
                for written_path in result.written_paths:
                    console.print(f"- {repo_path(written_path)}")
                if result.agent_transcript_path is not None:
                    console.print(f"Agent transcript: {repo_path(result.agent_transcript_path)}")
            else:
                console.print("[green]Manifest Runner V4 verified one product_code implementation gate.[/green]")
                console.print(f"Manifest: {repo_path(result.manifest_path)}")
                console.print(f"Milestone: {result.milestone_id}")
                console.print(f"Owning WR: {result.wr_id}")
                console.print(f"Plan path: {repo_path(result.plan_path)}")
            if result.validation_commands:
                console.print("Validation commands:")
                for command_result in result.validation_commands:
                    console.print(f"- {command_result}")
            console.print(f"Next legal action: {result.next_legal_action}")
        console.print("Must stop after this action: yes")
    except WorkflowError as error:
        console.print("[red]production:run-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("next")
def next_action(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
    lock_source_root: Path = typer.Option(TRACK_EXECUTION_LOCK_ROOT, help="Track Execution Lock source root."),
    contract_pack_root: Path = typer.Option(EXECUTION_CONTRACT_PACK_ROOT, help="Execution Contract Pack root."),
) -> None:
    try:
        from execution.cli import next_command as execution_next_command
        from execution.compiler import contract_pack_path, load_contract_pack

        if load_contract_pack(track, root=contract_pack_root) is not None:
            console.print("[cyan]Delegating next-action inspection to the Track Execution Harness.[/cyan]")
            execution_next_command(track=track, contract_pack_root=contract_pack_root)
            return
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        if context.loaded.manifest.ai_executable or context.loaded.manifest.full_automation_target:
            raise WorkflowError(
                f"{track}: executable/full-automation track requires Execution Contract Pack at "
                f"{repo_path(contract_pack_path(track, root=contract_pack_root))}; legacy next-action fallback is forbidden"
            )
        console.print("[yellow]Track Execution Harness fallback: no Execution Contract Pack exists; using legacy Manifest Runner next-action logic.[/yellow]")
        audit_errors = audit_manifest(context.loaded, track=context.track, roadmap=context.roadmap)
        if audit_errors:
            print_manifest_audit_blockers(audit_errors)
            raise typer.Exit(1)
        full_automation_errors: list[str] = []
        if context.loaded.manifest.full_automation_target:
            full_automation_errors = full_automation_preflight_errors(
                context.loaded,
                track=context.track,
                roadmap=context.roadmap,
                allow=None,
            )
        loaded_lock = load_track_execution_lock(track, root=lock_source_root)
        lock_errors = track_execution_lock_errors(
            context.loaded,
            loaded_lock,
            production_source=production_source,
            roadmap_source=roadmap_source,
            allow=FULL_TRACK_PERMISSION_SET,
            deny={"crate_creation", "foundation_extraction"},
            track=context.track,
        ) if context.loaded.manifest.full_automation_target else []
        if all(milestone.state == "completed" for milestone in context.track.milestones):
            console.print(f"Manifest: {repo_path(context.loaded.path)}")
            console.print(f"[green]Track {track} is complete.[/green]")
            for line in truth_claim_summary_lines(context.loaded.manifest):
                console.print(line)
            if context.loaded.manifest.full_automation_target:
                console.print(
                    "Full automation readiness: "
                    + ("ready" if not full_automation_errors else "blocked")
                )
                console.print(
                    "Execution lock: "
                    + (repo_path(loaded_lock.path) if loaded_lock is not None else "missing")
                )
                console.print(
                    "`--mode full-track` can run now: "
                    + ("yes" if not full_automation_errors and not lock_errors else "no")
                )
                if full_automation_errors:
                    print_full_automation_blockers(full_automation_errors)
                    raise typer.Exit(1)
                if lock_errors:
                    console.print("[red]Track Execution Lock blockers:[/red]")
                    for error in lock_errors:
                        console.print(f"- {error}")
                    raise typer.Exit(1)
            return
        entry, milestone = first_current_manifest_entry(context.loaded.manifest, context.track)
        workflow_action, blockers = next_action_blockers(
            entry,
            milestone,
            planning=context.planning,
            track=context.track,
            roadmap=context.roadmap,
        )
        console.print(f"Manifest: {repo_path(context.loaded.path)}")
        console.print(f"Current milestone: {entry.milestone_id} - {entry.title}")
        console.print(f"Next legal action: {entry.next_legal_action}")
        console.print(f"Workflow action: {workflow_action}")
        console.print(f"Implementation authorized now: {implementation_authorization_note(entry, workflow_action, blockers)}")
        console.print("Must stop after this action: yes")
        for line in truth_claim_summary_lines(context.loaded.manifest):
            console.print(line)
        if context.loaded.manifest.full_automation_target:
            console.print(
                "Full automation readiness: "
                + ("ready" if not full_automation_errors else "blocked")
            )
            console.print(
                "Execution lock: "
                + (repo_path(loaded_lock.path) if loaded_lock is not None else "missing")
            )
            console.print(
                "`--mode full-track` can run now: "
                + ("yes" if not full_automation_errors and not lock_errors else "no")
            )
            if full_automation_errors:
                print_full_automation_blockers(full_automation_errors)
                raise typer.Exit(1)
            if lock_errors:
                console.print("[red]Track Execution Lock blockers:[/red]")
                for error in lock_errors:
                    console.print(f"- {error}")
                raise typer.Exit(1)
        if blockers:
            console.print("Unmet gates:")
            for blocker in blockers:
                console.print(f"- {blocker}")
            if workflow_action != "track_expansion_required" or any(
                "Track Expansion must create or link" not in blocker for blocker in blockers
            ):
                raise typer.Exit(1)
    except WorkflowError as error:
        console.print("[red]production:next failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("audit-track")
def audit_track(
    track: str = typer.Option(..., "--track", help="Production track id, for example PT-TEST."),
    full_automation: bool = typer.Option(
        False,
        "--full-automation",
        help="Audit every remaining milestone for full automation readiness without mutation.",
    ),
    require_lock: bool = typer.Option(
        False,
        "--require-lock",
        help="Require a current Track Execution Lock for full-track AI execution.",
    ),
    production_source: Path = typer.Option(PRODUCTION_SOURCE, help="Production tracks YAML source."),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE, help="Active roadmap YAML source."),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT, help="Track Execution Manifest source root."),
    lock_source_root: Path = typer.Option(TRACK_EXECUTION_LOCK_ROOT, help="Track Execution Lock source root."),
    contract_pack_root: Path = typer.Option(EXECUTION_CONTRACT_PACK_ROOT, help="Execution Contract Pack root."),
) -> None:
    try:
        from execution.cli import preflight_command as execution_preflight_command
        from execution.compiler import compile_contract_pack, contract_pack_path, load_contract_pack, write_contract_pack
        from execution.locks import EXECUTION_LOCK_ROOT, execution_lock_errors as harness_lock_errors

        if full_automation and load_contract_pack(track, root=contract_pack_root) is None:
            context_for_harness = resolve_manifest_command_context(
                track,
                production_source=production_source,
                roadmap_source=roadmap_source,
                manifest_source_root=manifest_source_root,
            )
            if context_for_harness.loaded.manifest.ai_executable or context_for_harness.loaded.manifest.full_automation_target:
                try:
                    pack = compile_contract_pack(
                        track,
                        production_source=production_source,
                        roadmap_source=roadmap_source,
                        manifest_root=manifest_source_root,
                    )
                    write_contract_pack(pack, root=contract_pack_root)
                except WorkflowError as error:
                    raise WorkflowError(f"{track}: executable/full-automation Contract Pack compile failed: {error}") from error

        if full_automation and load_contract_pack(track, root=contract_pack_root) is not None:
            console.print("[cyan]Delegating full-automation audit to the Track Execution Harness.[/cyan]")
            execution_preflight_command(
                track=track,
                allow=sorted(FULL_TRACK_PERMISSION_SET),
                contract_pack_root=contract_pack_root,
            )
            if require_lock:
                lock_errors = harness_lock_errors(
                    track,
                    contract_pack_root=contract_pack_root,
                    lock_root=lock_source_root,
                    requested_permissions=FULL_TRACK_PERMISSION_SET,
                )
                if lock_errors:
                    console.print("[red]Execution Harness lock blockers:[/red]")
                    for error in lock_errors:
                        console.print(f"- {error}")
                    raise typer.Exit(1)
                console.print("[green]Execution Harness lock passed[/green]")
            return
        if full_automation:
            console.print("[yellow]Track Execution Harness fallback: no Execution Contract Pack exists; using legacy manifest audit.[/yellow]")
        context = resolve_manifest_command_context(
            track,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        )
        errors = audit_manifest(context.loaded, track=context.track, roadmap=context.roadmap)
        console.print(f"Manifest: {repo_path(context.loaded.path)}")
        for line in truth_claim_summary_lines(context.loaded.manifest):
            console.print(line)
        if errors:
            print_manifest_audit_blockers(errors)
            raise typer.Exit(1)
        console.print("[green]manifest audit passed[/green]")
        if full_automation:
            preflight_errors = full_automation_preflight_errors(
                context.loaded,
                track=context.track,
                roadmap=context.roadmap,
                allow=None,
            )
            lock_errors: list[str] = []
            if require_lock:
                loaded_lock = load_track_execution_lock(track, root=lock_source_root)
                lock_errors = track_execution_lock_errors(
                    context.loaded,
                    loaded_lock,
                    production_source=production_source,
                    roadmap_source=roadmap_source,
                    allow=FULL_TRACK_PERMISSION_SET,
                    deny={"crate_creation", "foundation_extraction"},
                    track=context.track,
                )
            if preflight_errors:
                print_full_automation_blockers(preflight_errors)
            if lock_errors:
                console.print("[red]Track Execution Lock blockers:[/red]")
                for error in lock_errors:
                    console.print(f"- {error}")
            if preflight_errors or lock_errors:
                raise typer.Exit(1)
            if require_lock:
                assert loaded_lock is not None
                console.print(f"[green]track execution lock passed[/green]")
                console.print(f"Execution lock: {repo_path(loaded_lock.path)}")
            remaining = remaining_manifest_entries(context.loaded.manifest, context.track)
            console.print("[green]full automation readiness preflight passed[/green]")
            console.print(f"Remaining milestones inspected: {len(remaining)}")
            for entry, _milestone, _index in remaining:
                console.print(f"- {entry.milestone_id}: {entry.execution_kind}")
    except WorkflowError as error:
        console.print("[red]production:audit-track failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("_commands", hidden=True)
def commands() -> None:
    """Keep Typer in multi-command mode so public subcommands are stable."""
    console.print("plan-track expand-track complete-track-contracts lock-track run-track next audit-track")


if __name__ == "__main__":
    app()
