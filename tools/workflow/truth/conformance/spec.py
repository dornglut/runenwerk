from __future__ import annotations

from pathlib import Path

import yaml
from pydantic import BaseModel, Field, field_validator

from roadmap_state import REPO_ROOT, WorkflowError, repo_path


CONFORMANCE_SPEC_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/truth-conformance-specs"


class RequiredFileSpec(BaseModel):
    path: str
    role: str
    required_symbols: list[str] = Field(default_factory=list)
    forbidden_patterns: list[str] = Field(default_factory=list)

    @field_validator("path", "role")
    @classmethod
    def non_empty_text(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("conformance spec text fields must not be empty")
        return value


class RootFacadeSpec(BaseModel):
    path: str
    forbidden_patterns: list[str] = Field(default_factory=list)


class EvidenceRequirementSpec(BaseModel):
    milestone_id: str
    evidence_kind: str
    required_subject_paths: bool = True
    required_validation_fragments: list[str] = Field(default_factory=list)


class SemanticCheckSpec(BaseModel):
    check_id: str
    description: str
    subject_paths: list[str] = Field(default_factory=list)
    behavior_probe_paths: list[str] = Field(default_factory=list)
    behavior_probe_ids: list[str] = Field(default_factory=list)
    evidence_kinds: list[str] = Field(default_factory=list)
    required_symbols: list[str] = Field(default_factory=list)
    required_validation_fragments: list[str] = Field(default_factory=list)

    @field_validator("check_id", "description")
    @classmethod
    def non_empty_text(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("semantic check text fields must not be empty")
        return value

    @field_validator(
        "subject_paths",
        "behavior_probe_paths",
        "behavior_probe_ids",
        "evidence_kinds",
        "required_symbols",
        "required_validation_fragments",
    )
    @classmethod
    def validate_text_lists(cls, value: list[str]) -> list[str]:
        return [item.strip() for item in value if item.strip()]


class DesignCoverageRequirementSpec(BaseModel):
    requirement_id: str
    source_path: str
    source_section: str
    summary: str
    owner: str
    status: str
    code_subjects: list[str] = Field(default_factory=list)
    test_subjects: list[str] = Field(default_factory=list)
    evidence_kinds: list[str] = Field(default_factory=list)
    semantic_check_ids: list[str] = Field(default_factory=list)
    deferral_authority: str | None = None
    notes: str | None = None

    @field_validator("requirement_id", "source_path", "source_section", "summary", "owner", "status")
    @classmethod
    def non_empty_requirement_text(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("design coverage requirement text fields must not be empty")
        return value


class DesignCoverageMatrix(BaseModel):
    version: int = 1
    track_id: str
    spec_id: str
    source_design_docs: list[str]
    requirements: list[DesignCoverageRequirementSpec]

    @field_validator("track_id", "spec_id")
    @classmethod
    def non_empty_identity(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("design coverage identity fields must not be empty")
        return value


class ConformanceSpec(BaseModel):
    version: int = 1
    track_id: str
    spec_id: str
    claim_ids: list[str]
    final_owner_dirs: list[str]
    design_coverage_path: str | None = None
    required_design_requirement_ids: list[str] = Field(default_factory=list)
    required_design_docs: list[str] = Field(default_factory=list)
    required_design_terms: list[str] = Field(default_factory=list)
    required_files: list[RequiredFileSpec]
    root_facades: list[RootFacadeSpec] = Field(default_factory=list)
    evidence_requirements: list[EvidenceRequirementSpec] = Field(default_factory=list)
    semantic_checks: list[SemanticCheckSpec] = Field(default_factory=list)
    forbidden_code_terms: list[str] = Field(default_factory=list)
    zero_gap_criteria: list[str] = Field(default_factory=list)

    @field_validator("track_id", "spec_id")
    @classmethod
    def non_empty_identity(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("conformance spec identity fields must not be empty")
        return value


def conformance_spec_path(track_id: str, spec_id: str) -> Path:
    return CONFORMANCE_SPEC_ROOT / track_id.lower() / f"{spec_id}.yaml"


def load_conformance_spec(track_id: str, spec_id: str, *, repo_root: Path = REPO_ROOT) -> ConformanceSpec:
    path = repo_root / repo_path(conformance_spec_path(track_id, spec_id))
    return load_conformance_spec_file(path, repo_root=repo_root)


def load_conformance_spec_file(path: Path | str, *, repo_root: Path = REPO_ROOT) -> ConformanceSpec:
    spec_path = Path(path)
    if not spec_path.is_absolute():
        spec_path = repo_root / spec_path
    if not spec_path.exists():
        raise WorkflowError(f"missing conformance spec {repo_path(spec_path)}")
    try:
        data = yaml.safe_load(spec_path.read_text(encoding="utf-8")) or {}
        return ConformanceSpec.model_validate(data)
    except (OSError, yaml.YAMLError, ValueError) as error:
        raise WorkflowError(f"{repo_path(spec_path)} is not a valid conformance spec: {error}") from error


def load_design_coverage_matrix_file(path: Path | str, *, repo_root: Path = REPO_ROOT) -> DesignCoverageMatrix:
    coverage_path = Path(path)
    if not coverage_path.is_absolute():
        coverage_path = repo_root / coverage_path
    if not coverage_path.exists():
        raise WorkflowError(f"missing design coverage matrix {repo_path(coverage_path)}")
    try:
        data = yaml.safe_load(coverage_path.read_text(encoding="utf-8")) or {}
        return DesignCoverageMatrix.model_validate(data)
    except (OSError, yaml.YAMLError, ValueError) as error:
        raise WorkflowError(f"{repo_path(coverage_path)} is not a valid design coverage matrix: {error}") from error
