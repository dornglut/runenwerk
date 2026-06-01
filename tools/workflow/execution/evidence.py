from __future__ import annotations

from pathlib import Path

import yaml

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from execution.contracts import ActionContract, EvidenceKind, StrictModel, now_utc_iso


EVIDENCE_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/execution-evidence"


class EvidenceRecord(StrictModel):
    version: int = 1
    track_id: str
    milestone_id: str
    action_id: str
    evidence_kind: EvidenceKind
    name: str
    status: str
    produced_at: str
    paths: list[str] = []
    validation_commands: list[str] = []


def evidence_path(
    *,
    track_id: str,
    milestone_id: str,
    evidence_kind: str,
    name: str,
    root: Path = EVIDENCE_ROOT,
) -> Path:
    safe_name = name.lower().replace(" ", "-").replace("/", "-")
    return root / track_id.lower() / milestone_id.lower() / f"{evidence_kind}-{safe_name}.yaml"


def write_evidence_record(record: EvidenceRecord, *, root: Path = EVIDENCE_ROOT) -> Path:
    path = evidence_path(
        track_id=record.track_id,
        milestone_id=record.milestone_id,
        evidence_kind=record.evidence_kind,
        name=record.name,
        root=root,
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(record.model_dump(mode="json"), sort_keys=False, width=4096),
        encoding="utf-8",
        newline="\n",
    )
    return path


def load_evidence_records(track_id: str, milestone_id: str, *, root: Path = EVIDENCE_ROOT) -> list[EvidenceRecord]:
    directory = root / track_id.lower() / milestone_id.lower()
    if not directory.exists():
        return []
    records: list[EvidenceRecord] = []
    for path in sorted(directory.glob("*.yaml")):
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
        try:
            records.append(EvidenceRecord.model_validate(data))
        except ValueError as error:
            raise WorkflowError(f"{repo_path(path)} is not a valid evidence record: {error}") from error
    return records


def missing_evidence_kinds(track_id: str, milestone_id: str, required: list[str], *, root: Path = EVIDENCE_ROOT) -> list[str]:
    records = load_evidence_records(track_id, milestone_id, root=root)
    present = {record.evidence_kind for record in records if record.status == "passed"}
    return sorted(kind for kind in required if kind not in present)


def passed_record(
    *,
    track_id: str,
    milestone_id: str,
    action_id: str,
    evidence_kind: EvidenceKind,
    name: str,
    paths: list[str],
    validation_commands: list[str],
) -> EvidenceRecord:
    return EvidenceRecord(
        track_id=track_id,
        milestone_id=milestone_id,
        action_id=action_id,
        evidence_kind=evidence_kind,
        name=name,
        status="passed",
        produced_at=now_utc_iso(),
        paths=paths,
        validation_commands=validation_commands,
    )


def evidence_paths_for_requirement(
    requirement,
    *,
    written_paths: list[Path],
    repo_root: Path,
) -> list[str]:
    if requirement.paths:
        resolved: list[str] = []
        for raw in requirement.paths:
            path = Path(raw)
            candidate = path if path.is_absolute() else repo_root / raw
            if not candidate.exists():
                raise WorkflowError(f"{requirement.name}: required evidence path is missing: {repo_path(candidate)}")
            resolved.append(repo_path(candidate))
        return resolved
    if requirement.kind == "runtime_test":
        return []
    raise WorkflowError(f"{requirement.name}: {requirement.kind} evidence requires explicit artifact paths")


def resolve_evidence_records(
    action: ActionContract,
    *,
    validation_results,
    written_paths: list[Path],
    evidence_root: Path = EVIDENCE_ROOT,
    repo_root: Path = REPO_ROOT,
) -> list[Path]:
    validation_refs = [str(result) for result in validation_results]
    if not validation_refs:
        raise WorkflowError(f"{action.action_id}: evidence cannot be recorded without validation results")
    paths: list[Path] = []
    for requirement in action.evidence_required:
        evidence_refs = evidence_paths_for_requirement(
            requirement,
            written_paths=written_paths,
            repo_root=repo_root,
        )
        paths.append(
            write_evidence_record(
                passed_record(
                    track_id=action.track_id,
                    milestone_id=action.milestone_id,
                    action_id=action.action_id,
                    evidence_kind=requirement.kind,
                    name=requirement.name,
                    paths=evidence_refs,
                    validation_commands=validation_refs,
                ),
                root=evidence_root,
            )
        )
    return paths
