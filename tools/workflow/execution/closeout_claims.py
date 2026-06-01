from __future__ import annotations

from pathlib import Path

from execution.contracts import ActionContract
from execution.evidence import EVIDENCE_ROOT, load_evidence_records


def closeout_claim_errors(action: ActionContract, *, evidence_root: Path = EVIDENCE_ROOT) -> list[str]:
    errors: list[str] = []
    if action.execution_kind == "proof_aggregation":
        if not action.required_prior_milestones:
            return [f"{action.action_id}: proof aggregation requires prior milestones"]
        for milestone_id in action.required_prior_milestones:
            records = load_evidence_records(action.track_id, milestone_id, root=evidence_root)
            present = {record.evidence_kind for record in records if record.status == "passed"}
            for requirement in action.evidence_required:
                if requirement.required and requirement.kind not in present:
                    errors.append(
                        f"{action.action_id}: prior milestone {milestone_id} is missing {requirement.kind} evidence"
                    )
        return errors
    if action.closeout_contract.completion_quality in {"runtime_proven", "architecture_runtime_proven"}:
        records = load_evidence_records(action.track_id, action.milestone_id, root=evidence_root)
        present = {record.evidence_kind for record in records if record.status == "passed"}
        for requirement in action.closeout_contract.evidence_required:
            if requirement.required and requirement.kind not in present:
                errors.append(
                    f"{action.action_id}: closeout cannot claim {action.closeout_contract.completion_quality} "
                    f"without {requirement.kind} evidence"
                )
    return errors
