from __future__ import annotations

from pathlib import Path

import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

from roadmap_state import REPO_ROOT, WorkflowError, repo_path


TRUTH_VERIFIER_REGISTRY_PATH = REPO_ROOT / "docs-site/src/content/docs/workspace/truth-verifier-registry.yaml"


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_assignment=True)


class TruthVerifierRegistryEntry(StrictModel):
    verifier_id: str
    track_id: str
    claim_id: str
    backend: str
    conformance_spec_path: str | None = None
    source_digest_paths: list[str] = Field(default_factory=list)

    @field_validator("verifier_id", "track_id", "claim_id", "backend")
    @classmethod
    def required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("truth verifier registry text fields must not be empty")
        return cleaned

    @field_validator("source_digest_paths")
    @classmethod
    def clean_paths(cls, values: list[str]) -> list[str]:
        return [value.strip() for value in values if value.strip()]

    @model_validator(mode="after")
    def validate_entry(self) -> TruthVerifierRegistryEntry:
        if self.backend == "ui_program_architecture" and not self.conformance_spec_path:
            raise ValueError("ui_program_architecture verifier entries must declare conformance_spec_path")
        return self

    def source_paths(self) -> list[str]:
        paths = list(self.source_digest_paths)
        if self.conformance_spec_path:
            paths.append(self.conformance_spec_path)
        paths.append(repo_path(TRUTH_VERIFIER_REGISTRY_PATH))
        return sorted(set(paths))


class TruthVerifierRegistry(StrictModel):
    version: int = 1
    entries: list[TruthVerifierRegistryEntry]

    @model_validator(mode="after")
    def validate_unique_bindings(self) -> TruthVerifierRegistry:
        seen: set[tuple[str, str, str]] = set()
        for entry in self.entries:
            key = (entry.verifier_id, entry.track_id, entry.claim_id)
            if key in seen:
                raise ValueError(
                    f"duplicate truth verifier binding: {entry.verifier_id}/{entry.track_id}/{entry.claim_id}"
                )
            seen.add(key)
        return self


def load_truth_verifier_registry(
    *,
    repo_root: Path = REPO_ROOT,
    path: Path = TRUTH_VERIFIER_REGISTRY_PATH,
) -> TruthVerifierRegistry:
    registry_path = path if path.is_absolute() else repo_root / path
    if not registry_path.exists():
        raise WorkflowError(f"missing truth verifier registry: {repo_path(registry_path)}")
    try:
        data = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
        return TruthVerifierRegistry.model_validate(data)
    except (OSError, yaml.YAMLError, ValueError) as error:
        raise WorkflowError(f"{repo_path(registry_path)} is not a valid truth verifier registry: {error}") from error


def verifier_binding(
    *,
    verifier_id: str,
    track_id: str,
    claim_id: str,
    repo_root: Path = REPO_ROOT,
) -> TruthVerifierRegistryEntry:
    registry = load_truth_verifier_registry(repo_root=repo_root)
    for entry in registry.entries:
        if entry.verifier_id == verifier_id and entry.track_id == track_id and entry.claim_id == claim_id:
            return entry
    raise WorkflowError(f"{track_id}: truth verifier {verifier_id!r} is not registered for claim {claim_id!r}")


def evidence_referenced_ledger_paths(raw_path: str, *, repo_root: Path) -> list[str]:
    evidence_marker = "docs-site/src/content/docs/reports/execution-evidence/"
    normalized = raw_path.strip().strip("/")
    if not normalized.startswith(evidence_marker):
        return []
    evidence_path = repo_root / normalized
    if not evidence_path.exists():
        return []
    records = [evidence_path] if evidence_path.is_file() else sorted(evidence_path.rglob("*.yaml"))
    ledgers: list[str] = []
    for record in records:
        try:
            data = yaml.safe_load(record.read_text(encoding="utf-8")) or {}
        except (OSError, yaml.YAMLError):
            continue
        if not isinstance(data, dict):
            continue
        for item in data.get("validation_provenance") or []:
            if not isinstance(item, dict):
                continue
            ledger_path = item.get("run_ledger_path")
            if isinstance(ledger_path, str) and ledger_path.strip():
                ledgers.append(ledger_path.strip())
    return sorted(set(ledgers))


def expanded_source_paths(entry: TruthVerifierRegistryEntry, *, repo_root: Path) -> list[str]:
    return sorted(set(entry.source_paths()))


def verifier_source_paths(
    verifier_id: str,
    *,
    track_id: str | None = None,
    claim_id: str | None = None,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    registry = load_truth_verifier_registry(repo_root=repo_root)
    paths: list[str] = []
    for entry in registry.entries:
        if entry.verifier_id != verifier_id:
            continue
        if track_id is not None and entry.track_id != track_id:
            continue
        if claim_id is not None and entry.claim_id != claim_id:
            continue
        paths.extend(expanded_source_paths(entry, repo_root=repo_root))
    if not paths:
        if track_id is not None and claim_id is not None:
            raise WorkflowError(f"{track_id}: truth verifier {verifier_id!r} is not registered for claim {claim_id!r}")
        raise WorkflowError(f"unknown truth verifier {verifier_id!r}")
    return sorted(set(paths))
