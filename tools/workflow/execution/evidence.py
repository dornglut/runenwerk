from __future__ import annotations

import hashlib
import json
from pathlib import Path

import yaml

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from execution.contracts import ActionContract, EvidenceKind, StrictModel, now_utc_iso
from truth.certificates import digest_path


EVIDENCE_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/execution-evidence"
RUN_LEDGER_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/track-execution-runs"


class ValidationProvenance(StrictModel):
    command_id: str
    argv: list[str]
    returncode: int
    run_ledger_path: str
    run_action_id: str
    validation_result_digest: str
    subject_digests: dict[str, str] = {}


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
    subject_paths: list[str] = []
    subject_digests: dict[str, str] = {}
    validation_commands: list[str] = []
    validation_provenance: list[ValidationProvenance] = []


def run_ledger_record_path(track_id: str, run_id: str) -> str:
    return repo_path(RUN_LEDGER_ROOT / track_id.lower() / f"{run_id}.yaml")


def validation_result_digest(
    *,
    command_id: str,
    argv: list[str] | tuple[str, ...],
    returncode: int,
    files_changed: list[str] | tuple[str, ...],
    subject_digests: dict[str, str],
) -> str:
    payload = {
        "argv": list(argv),
        "command_id": command_id,
        "files_changed": sorted(files_changed),
        "returncode": returncode,
        "subject_digests": {key: subject_digests[key] for key in sorted(subject_digests)},
    }
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def validation_provenance_for_results(
    *,
    validation_results,
    run_ledger_path: str,
    run_action_id: str,
    subject_digests: dict[str, str],
) -> list[ValidationProvenance]:
    return [
        ValidationProvenance(
            command_id=result.command_id,
            argv=list(result.argv),
            returncode=result.returncode,
            run_ledger_path=run_ledger_path,
            run_action_id=run_action_id,
            validation_result_digest=validation_result_digest(
                command_id=result.command_id,
                argv=result.argv,
                returncode=result.returncode,
                files_changed=result.files_changed,
                subject_digests=subject_digests,
            ),
            subject_digests=subject_digests,
        )
        for result in validation_results
    ]


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


def default_evidence_output_path(
    *,
    track_id: str,
    milestone_id: str,
    evidence_kind: str,
    name: str,
) -> str:
    return repo_path(evidence_path(track_id=track_id, milestone_id=milestone_id, evidence_kind=evidence_kind, name=name))


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
    subject_paths: list[str] | None = None,
    subject_digests: dict[str, str] | None = None,
    validation_commands: list[str],
    validation_provenance: list[ValidationProvenance] | None = None,
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
        subject_paths=subject_paths or [],
        subject_digests=subject_digests or {},
        validation_commands=validation_commands,
        validation_provenance=validation_provenance or [],
    )


def current_subject_digests(subject_paths: list[str], *, repo_root: Path) -> dict[str, str]:
    return {
        raw_path: digest_path(repo_root / raw_path)
        for raw_path in sorted(set(subject_paths))
    }


def evidence_artifact_refs_for_requirement(
    requirement,
    *,
    repo_root: Path,
    validation_results,
) -> list[str]:
    if not requirement.paths:
        raise WorkflowError(f"{requirement.name}: {requirement.kind} evidence requires declared evidence output path")
    for raw in requirement.paths:
        path = Path(raw)
        if path.is_absolute():
            raise WorkflowError(f"{requirement.name}: evidence path must be repository-relative: {raw}")
        candidate = repo_root / raw
        if candidate.exists() and candidate.is_dir():
            raise WorkflowError(f"{requirement.name}: evidence path must be a file, not a directory: {raw}")
    if requirement.kind == "runtime_test":
        if not requirement.validation_command_ids:
            raise WorkflowError(f"{requirement.name}: runtime_test evidence requires validation_command_ids")
        seen = {result.command_id for result in validation_results}
        missing = sorted(command_id for command_id in requirement.validation_command_ids if command_id not in seen)
        if missing:
            raise WorkflowError(f"{requirement.name}: runtime_test evidence is missing validation command results: {', '.join(missing)}")
        return list(requirement.subject_paths)
    if not requirement.subject_paths:
        raise WorkflowError(f"{requirement.name}: {requirement.kind} evidence requires subject_paths")
    for raw in requirement.subject_paths:
        path = Path(raw)
        if path.is_absolute():
            raise WorkflowError(f"{requirement.name}: subject path must be repository-relative: {raw}")
        candidate = repo_root / raw
        if not candidate.exists():
            raise WorkflowError(f"{requirement.name}: subject path does not exist: {raw}")
        if not candidate.is_file():
            raise WorkflowError(f"{requirement.name}: subject path must be an exact file, not a directory: {raw}")
    return list(requirement.subject_paths)


def write_resolver_evidence_records(
    action: ActionContract,
    *,
    validation_results,
    workspace_root: Path,
    run_id: str | None = None,
) -> list[Path]:
    validation_refs = [str(result) for result in validation_results]
    if not validation_refs:
        raise WorkflowError(f"{action.action_id}: evidence cannot be recorded without validation results")
    if not run_id:
        raise WorkflowError(f"{action.action_id}: evidence requires a run_id so validation provenance can resolve to a run ledger")
    ledger_path = run_ledger_record_path(action.track_id, run_id)
    paths: list[Path] = []
    for requirement in action.evidence_required:
        evidence_refs = evidence_artifact_refs_for_requirement(
            requirement,
            repo_root=workspace_root,
            validation_results=validation_results,
        )
        subject_digests = current_subject_digests(evidence_refs, repo_root=workspace_root)
        record = passed_record(
            track_id=action.track_id,
            milestone_id=action.milestone_id,
            action_id=action.action_id,
            evidence_kind=requirement.kind,
            name=requirement.name,
            paths=evidence_refs,
            subject_paths=evidence_refs,
            subject_digests=subject_digests,
            validation_commands=validation_refs,
            validation_provenance=validation_provenance_for_results(
                validation_results=validation_results,
                run_ledger_path=ledger_path,
                run_action_id=action.action_id,
                subject_digests=subject_digests,
            ),
        )
        if len(requirement.paths) != 1:
            raise WorkflowError(f"{requirement.name}: evidence requires exactly one output record path")
        output = workspace_root / requirement.paths[0]
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(
            yaml.safe_dump(record.model_dump(mode="json"), sort_keys=False, width=4096),
            encoding="utf-8",
            newline="\n",
        )
        paths.append(output)
    return paths


def resolve_evidence_records(
    action: ActionContract,
    *,
    validation_results,
    written_paths: list[Path],
    evidence_root: Path = EVIDENCE_ROOT,
    repo_root: Path = REPO_ROOT,
) -> list[Path]:
    """Compatibility wrapper for older direct tests.

    New execution paths write evidence inside the action workspace and import it
    as declared output. This wrapper keeps non-kernel callers honest by using a
    temporary workspace rooted at ``repo_root`` and still requiring declared
    evidence output paths.
    """
    records: list[Path] = []
    for requirement in action.evidence_required:
        records.append(
            write_evidence_record(
                passed_record(
                    track_id=action.track_id,
                    milestone_id=action.milestone_id,
                    action_id=action.action_id,
                    evidence_kind=requirement.kind,
                    name=requirement.name,
                    paths=evidence_artifact_refs_for_requirement(
                        requirement,
                        repo_root=repo_root,
                        validation_results=validation_results,
                    ),
                    subject_paths=list(requirement.subject_paths),
                    subject_digests=current_subject_digests(list(requirement.subject_paths), repo_root=repo_root),
                    validation_commands=[str(result) for result in validation_results],
                    validation_provenance=[],
                ),
                root=evidence_root,
            )
        )
    return records
