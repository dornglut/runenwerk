from __future__ import annotations

from datetime import UTC, datetime
from hashlib import sha256
from pathlib import Path
from typing import Literal

import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

from roadmap_state import REPO_ROOT, WorkflowError, repo_path


TRUTH_CERTIFICATE_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/truth-certificates"
STRONG_CERTIFIED_LEVELS = {"runtime_proven", "architecture_runtime_proven", "perfectionist_verified"}

TruthStatus = Literal["passed", "failed"]
FindingSeverity = Literal["error", "warning"]


class StrictModel(BaseModel):
    model_config = ConfigDict(extra="forbid", validate_assignment=True)


def now_utc_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


class TruthFinding(StrictModel):
    finding_id: str
    severity: FindingSeverity = "error"
    message: str
    subject_paths: list[str] = Field(default_factory=list)
    remediation: str

    @field_validator("finding_id", "message", "remediation")
    @classmethod
    def required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("truth finding text fields must not be empty")
        return cleaned

    @field_validator("subject_paths")
    @classmethod
    def clean_subject_paths(cls, values: list[str]) -> list[str]:
        return [value.strip() for value in values if value.strip()]


class TruthCertificate(StrictModel):
    version: int = 1
    track_id: str
    claim_id: str
    verifier: str
    status: TruthStatus
    produced_at: str
    source_digests: dict[str, str] = Field(default_factory=dict)
    findings: list[TruthFinding] = Field(default_factory=list)
    known_gaps: list[str] = Field(default_factory=list)
    known_risks: list[str] = Field(default_factory=list)
    truth_drift: list[str] = Field(default_factory=list)
    checks: list[str] = Field(default_factory=list)

    @field_validator("track_id", "claim_id", "verifier", "produced_at")
    @classmethod
    def required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("truth certificate text fields must not be empty")
        return cleaned

    @field_validator("known_gaps", "known_risks", "truth_drift", "checks")
    @classmethod
    def clean_text_lists(cls, values: list[str]) -> list[str]:
        return [value.strip() for value in values if value.strip()]

    @model_validator(mode="after")
    def validate_status_shape(self) -> TruthCertificate:
        if self.status == "passed" and (self.findings or self.known_gaps or self.known_risks or self.truth_drift):
            raise ValueError("passed truth certificates must have zero findings, gaps, risks, and drift")
        if not self.source_digests:
            raise ValueError("truth certificates must declare source_digests")
        return self


def certificate_path(track_id: str, claim_id: str, *, root: Path = TRUTH_CERTIFICATE_ROOT) -> Path:
    return root / track_id.lower() / f"{claim_id}.yaml"


def write_certificate(certificate: TruthCertificate, *, root: Path = TRUTH_CERTIFICATE_ROOT) -> Path:
    path = certificate_path(certificate.track_id, certificate.claim_id, root=root)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(certificate.model_dump(mode="json"), sort_keys=False, width=4096),
        encoding="utf-8",
        newline="\n",
    )
    return path


def load_certificate(track_id: str, claim_id: str, *, root: Path = TRUTH_CERTIFICATE_ROOT) -> TruthCertificate | None:
    path = certificate_path(track_id, claim_id, root=root)
    if not path.exists():
        return None
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    try:
        return TruthCertificate.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid truth certificate: {error}") from error


def digest_path(path: Path) -> str:
    if path.is_file():
        return sha256(path.read_bytes()).hexdigest()
    if path.is_dir():
        digest = sha256()
        for child in sorted(candidate for candidate in path.rglob("*") if candidate.is_file()):
            if any(part in {"__pycache__", ".pytest_cache", "target"} for part in child.parts):
                continue
            digest.update(str(child.relative_to(path)).encode("utf-8"))
            digest.update(b"\0")
            digest.update(child.read_bytes())
            digest.update(b"\0")
        return digest.hexdigest()
    return "missing"


def source_digests(paths: list[str], *, repo_root: Path = REPO_ROOT) -> dict[str, str]:
    digests: dict[str, str] = {}
    for raw_path in sorted({path for path in paths if path.strip()}):
        path = repo_root / raw_path
        digests[raw_path] = digest_path(path)
    return digests


def certificate_freshness_errors(certificate: TruthCertificate, *, repo_root: Path = REPO_ROOT) -> list[str]:
    errors: list[str] = []
    for raw_path, recorded in certificate.source_digests.items():
        current = digest_path(repo_root / raw_path)
        if current != recorded:
            errors.append(
                f"{certificate.track_id}: truth certificate {certificate.claim_id} is stale for {raw_path}"
            )
    return errors


def strong_claim_requires_certificate(claim: object) -> bool:
    return (
        getattr(claim, "claim_status", None) == "satisfied"
        and getattr(claim, "claim_level", None) in STRONG_CERTIFIED_LEVELS
    )


def certificate_errors_for_claim(
    track_id: str,
    claim: object,
    *,
    root: Path = TRUTH_CERTIFICATE_ROOT,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    if not strong_claim_requires_certificate(claim):
        return []

    claim_id = getattr(claim, "claim_id")
    verifier = getattr(claim, "truth_verifier", None)
    declared_path = getattr(claim, "truth_certificate_path", None)
    errors: list[str] = []
    expected_path = repo_path(certificate_path(track_id, claim_id, root=root))
    if not verifier:
        errors.append(f"{track_id}: satisfied strong truth claim {claim_id} must declare truth_verifier")
    if not declared_path:
        errors.append(f"{track_id}: satisfied strong truth claim {claim_id} must declare truth_certificate_path")
    elif declared_path != expected_path:
        errors.append(
            f"{track_id}: truth claim {claim_id} truth_certificate_path must be {expected_path}, got {declared_path}"
        )
    certificate = load_certificate(track_id, claim_id, root=root)
    if certificate is None:
        errors.append(f"{track_id}: truth claim {claim_id} requires current certificate {expected_path}")
        return errors
    if certificate.track_id != track_id or certificate.claim_id != claim_id:
        errors.append(f"{track_id}: truth certificate {expected_path} is for the wrong track or claim")
    if verifier and certificate.verifier != verifier:
        errors.append(
            f"{track_id}: truth certificate {claim_id} verifier {certificate.verifier!r} does not match {verifier!r}"
        )
    if certificate.status != "passed":
        errors.append(f"{track_id}: truth certificate {claim_id} status is {certificate.status}, expected passed")
    if certificate.findings or certificate.known_gaps or certificate.known_risks or certificate.truth_drift:
        errors.append(f"{track_id}: truth certificate {claim_id} is not zero-finding/zero-gap/zero-risk/zero-drift")
    if verifier:
        try:
            from truth.registry import verifier_source_paths

            required_paths = set(
                verifier_source_paths(
                    verifier,
                    track_id=track_id,
                    claim_id=claim_id,
                    repo_root=repo_root,
                )
            )
        except WorkflowError as error:
            errors.append(f"{track_id}: cannot resolve verifier source paths for {claim_id}: {error}")
        else:
            recorded_paths = set(certificate.source_digests)
            missing_paths = sorted(required_paths - recorded_paths)
            extra_paths = sorted(recorded_paths - required_paths)
            for missing_path in missing_paths:
                errors.append(
                    f"{track_id}: truth certificate {claim_id} is missing required source digest for {missing_path}"
                )
            for extra_path in extra_paths:
                errors.append(
                    f"{track_id}: truth certificate {claim_id} records obsolete source digest for {extra_path}"
                )
    errors.extend(certificate_freshness_errors(certificate, repo_root=repo_root))
    return errors


def certificate_summary_lines(track_id: str, claims: list[object], *, root: Path = TRUTH_CERTIFICATE_ROOT) -> list[str]:
    lines: list[str] = []
    for claim in claims:
        claim_id = getattr(claim, "claim_id", "")
        if not claim_id:
            continue
        certificate = load_certificate(track_id, claim_id, root=root)
        if certificate is None:
            if strong_claim_requires_certificate(claim):
                lines.append(f"- Truth certificate `{claim_id}`: missing")
            continue
        freshness = certificate_freshness_errors(certificate)
        status = certificate.status
        details = (
            f"{len(certificate.findings)} findings, {len(certificate.known_gaps)} gaps, "
            f"{len(certificate.known_risks)} risks, {len(certificate.truth_drift)} drift"
        )
        if freshness:
            details += f", stale={len(freshness)}"
        lines.append(f"- Truth certificate `{claim_id}`: {status} ({details})")
    return lines
