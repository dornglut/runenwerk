from __future__ import annotations

from datetime import UTC, datetime
from hashlib import sha256
from pathlib import Path

import yaml
from pydantic import Field, field_validator, model_validator

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from execution.compiler import CONTRACT_PACK_ROOT, contract_pack_path, load_contract_pack
from execution.contracts import ContractPack
from execution.contracts import StrictModel


EXECUTION_LOCK_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/execution-locks"


class ExecutionLock(StrictModel):
    track_id: str
    locked_by: str
    locked_at: str
    contract_pack_digest: str
    source_digests: dict[str, str]
    granted_permissions: list[str] = Field(default_factory=list)
    denied_permissions: list[str] = Field(default_factory=list)

    @field_validator("track_id", "locked_by", "locked_at", "contract_pack_digest")
    @classmethod
    def validate_required_text(cls, value: str) -> str:
        cleaned = value.strip()
        if not cleaned:
            raise ValueError("lock text fields must not be empty")
        return cleaned

    @model_validator(mode="after")
    def validate_permissions(self) -> ExecutionLock:
        overlap = sorted(set(self.granted_permissions) & set(self.denied_permissions))
        if overlap:
            raise ValueError(f"permissions cannot be both granted and denied: {', '.join(overlap)}")
        return self


def lock_path(track_id: str, *, root: Path = EXECUTION_LOCK_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


def sha256_file(path: Path) -> str:
    if not path.exists():
        raise WorkflowError(f"cannot digest missing source file: {repo_path(path)}")
    return sha256(path.read_bytes()).hexdigest()


def path_from_repo(path: str) -> Path:
    candidate = Path(path)
    if candidate.is_absolute():
        return candidate
    return REPO_ROOT / path


def build_execution_lock(
    track_id: str,
    *,
    locked_by: str,
    contract_pack_root: Path = CONTRACT_PACK_ROOT,
    granted_permissions: list[str],
    denied_permissions: list[str],
) -> ExecutionLock:
    pack_path = contract_pack_path(track_id, root=contract_pack_root)
    pack = load_contract_pack(track_id, root=contract_pack_root)
    if pack is None:
        raise WorkflowError(f"{track_id}: missing Execution Contract Pack at {repo_path(pack_path)}")
    return ExecutionLock(
        track_id=track_id,
        locked_by=locked_by,
        locked_at=datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        contract_pack_digest=sha256_file(pack_path),
        source_digests=pack.source_digests,
        granted_permissions=sorted(set(granted_permissions)),
        denied_permissions=sorted(set(denied_permissions)),
    )


def contract_pack_freshness_errors(pack: ContractPack) -> list[str]:
    errors: list[str] = []
    for source, expected_digest in pack.source_digests.items():
        source_path = path_from_repo(source)
        if not source_path.exists():
            errors.append(f"{pack.track_id}: Contract Pack source is missing: {source}")
            continue
        actual = sha256_file(source_path)
        if actual != expected_digest:
            errors.append(f"{pack.track_id}: Contract Pack source digest is stale for {source}")
    return errors


def write_execution_lock(lock: ExecutionLock, *, root: Path = EXECUTION_LOCK_ROOT) -> Path:
    path = lock_path(lock.track_id, root=root)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(lock.model_dump(mode="json"), sort_keys=False, width=4096),
        encoding="utf-8",
        newline="\n",
    )
    return path


def load_execution_lock(track_id: str, *, root: Path = EXECUTION_LOCK_ROOT) -> ExecutionLock | None:
    path = lock_path(track_id, root=root)
    if not path.exists():
        return None
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    try:
        lock = ExecutionLock.model_validate(data)
    except ValueError as error:
        raise WorkflowError(f"{repo_path(path)} is not a valid Execution Lock: {error}") from error
    if lock.track_id != track_id:
        raise WorkflowError(f"{repo_path(path)} declares track_id={lock.track_id}, expected {track_id}")
    return lock


def execution_lock_errors(
    track_id: str,
    *,
    contract_pack_root: Path = CONTRACT_PACK_ROOT,
    lock_root: Path = EXECUTION_LOCK_ROOT,
    requested_permissions: set[str],
) -> list[str]:
    errors: list[str] = []
    pack_path = contract_pack_path(track_id, root=contract_pack_root)
    lock = load_execution_lock(track_id, root=lock_root)
    if lock is None:
        return [f"{track_id}: requires current Execution Lock"]
    if not pack_path.exists():
        return [f"{track_id}: missing Execution Contract Pack at {repo_path(pack_path)}"]
    if sha256_file(pack_path) != lock.contract_pack_digest:
        errors.append(f"{track_id}: execution lock contract_pack_digest is stale")
    pack = load_contract_pack(track_id, root=contract_pack_root)
    if pack is not None:
        errors.extend(contract_pack_freshness_errors(pack))
    for source, expected_digest in lock.source_digests.items():
        source_path = path_from_repo(source)
        if not source_path.exists():
            errors.append(f"{track_id}: execution lock source is missing: {source}")
            continue
        actual = sha256_file(source_path)
        if actual != expected_digest:
            errors.append(f"{track_id}: execution lock source digest is stale for {source}")
    ungranted = sorted(requested_permissions - set(lock.granted_permissions))
    if ungranted:
        errors.append(f"{track_id}: requested permissions exceed execution lock grants: {', '.join(ungranted)}")
    denied = sorted(requested_permissions & set(lock.denied_permissions))
    if denied:
        errors.append(f"{track_id}: requested permissions are denied by execution lock: {', '.join(denied)}")
    return errors
