from __future__ import annotations

from pathlib import Path

from execution.contracts import ActionContract
from execution.evidence import EVIDENCE_ROOT, EvidenceRecord, load_evidence_records
from roadmap_state import REPO_ROOT


def evidence_repo_root(evidence_root: Path) -> Path:
    marker = Path("docs-site/src/content/docs/reports/execution-evidence").parts
    parts = evidence_root.resolve().parts
    if len(parts) >= len(marker) and tuple(parts[-len(marker) :]) == marker:
        root = evidence_root.resolve()
        for _part in marker:
            root = root.parent
        return root
    return REPO_ROOT


def evidence_record_errors(record: EvidenceRecord, *, evidence_root: Path) -> list[str]:
    errors: list[str] = []
    if record.status != "passed":
        errors.append(f"{record.milestone_id}: {record.evidence_kind} evidence {record.name} is not passed")
    if not record.validation_commands:
        errors.append(f"{record.milestone_id}: {record.evidence_kind} evidence {record.name} has no validation commands")
    if record.evidence_kind == "runtime_test":
        return errors
    if not record.subject_paths:
        errors.append(f"{record.milestone_id}: {record.evidence_kind} evidence {record.name} has no subject_paths")
        return errors
    root = evidence_repo_root(evidence_root)
    for subject_path in record.subject_paths:
        candidate = root / subject_path
        if not candidate.exists():
            errors.append(f"{record.milestone_id}: {record.evidence_kind} evidence subject is missing: {subject_path}")
        elif not candidate.is_file():
            errors.append(f"{record.milestone_id}: {record.evidence_kind} evidence subject is not an exact file: {subject_path}")
    return errors


def closeout_claim_errors(action: ActionContract, *, evidence_root: Path = EVIDENCE_ROOT) -> list[str]:
    errors: list[str] = []
    if action.execution_kind == "proof_aggregation":
        if not action.required_prior_milestones:
            return [f"{action.action_id}: proof aggregation requires prior milestones"]
        for milestone_id in action.required_prior_milestones:
            records = load_evidence_records(action.track_id, milestone_id, root=evidence_root)
            valid_records = []
            for record in records:
                record_errors = evidence_record_errors(record, evidence_root=evidence_root)
                if record_errors:
                    errors.extend(record_errors)
                else:
                    valid_records.append(record)
            present = {record.evidence_kind for record in valid_records}
            for requirement in action.evidence_required:
                if requirement.required and requirement.kind not in present:
                    errors.append(
                        f"{action.action_id}: prior milestone {milestone_id} is missing {requirement.kind} evidence"
                    )
        return errors
    if action.closeout_contract.completion_quality in {
        "runtime_proven",
        "proof_slice_runtime_proven",
        "architecture_runtime_proven",
        "perfectionist_verified",
    }:
        records = load_evidence_records(action.track_id, action.milestone_id, root=evidence_root)
        valid_records = []
        for record in records:
            record_errors = evidence_record_errors(record, evidence_root=evidence_root)
            if record_errors:
                errors.extend(record_errors)
            else:
                valid_records.append(record)
        present = {record.evidence_kind for record in valid_records}
        for requirement in action.closeout_contract.evidence_required:
            if requirement.required and requirement.kind not in present:
                errors.append(
                    f"{action.action_id}: closeout cannot claim {action.closeout_contract.completion_quality} "
                    f"without {requirement.kind} evidence"
                )
    return errors
