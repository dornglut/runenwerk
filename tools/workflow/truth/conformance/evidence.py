from __future__ import annotations

from pathlib import Path

import yaml

from roadmap_state import REPO_ROOT, repo_path

from execution.evidence import validation_result_digest
from truth.certificates import TruthFinding, digest_path
from truth.conformance.rust_module_shape import finding_slug
from truth.conformance.spec import ConformanceSpec


KNOWN_EVIDENCE_KINDS = {
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


def verify_evidence_records(
    spec: ConformanceSpec,
    *,
    repo_root: Path = REPO_ROOT,
) -> tuple[list[TruthFinding], list[str]]:
    findings: list[TruthFinding] = []
    checks: list[str] = []

    checks.append("conformance evidence records are resolver-backed and non-self-referential")
    for requirement in spec.evidence_requirements:
        if requirement.evidence_kind not in KNOWN_EVIDENCE_KINDS:
            findings.append(
                TruthFinding(
                    finding_id=f"unknown-evidence-kind-{finding_slug(requirement.evidence_kind)}",
                    message=f"Conformance spec declares unknown evidence kind `{requirement.evidence_kind}`.",
                    subject_paths=[],
                    remediation="Use a resolver-backed evidence kind before certification.",
                )
            )
            continue

        evidence_dir = (
            repo_root
            / "docs-site/src/content/docs/reports/execution-evidence"
            / spec.track_id.lower()
            / requirement.milestone_id.lower()
        )
        records = sorted(evidence_dir.glob(f"{requirement.evidence_kind}-*.yaml"))
        if not records:
            findings.append(
                TruthFinding(
                    finding_id=f"missing-evidence-{finding_slug(requirement.milestone_id)}-{finding_slug(requirement.evidence_kind)}",
                    message=f"Missing `{requirement.evidence_kind}` evidence for {requirement.milestone_id}.",
                    subject_paths=[repo_path(evidence_dir)],
                    remediation="Create resolver-backed evidence for the declared milestone and category.",
                )
            )
            continue

        for record in records:
            try:
                data = yaml.safe_load(record.read_text(encoding="utf-8")) or {}
            except (OSError, yaml.YAMLError) as error:
                findings.append(
                    TruthFinding(
                        finding_id=f"malformed-evidence-{finding_slug(repo_path(record))}",
                        message=f"Evidence record {repo_path(record)} is not valid YAML: {error}.",
                        subject_paths=[repo_path(record)],
                        remediation="Repair the machine-readable evidence record.",
                    )
                )
                continue
            findings.extend(validate_evidence_record(record, data, requirement, repo_root=repo_root))

    return findings, checks


def validate_evidence_record(record: Path, data: dict, requirement, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    raw_record = repo_path(record)
    if data.get("evidence_kind") != requirement.evidence_kind:
        findings.append(
            TruthFinding(
                finding_id=f"wrong-evidence-kind-{finding_slug(raw_record)}",
                message=f"{raw_record} declares evidence kind `{data.get('evidence_kind')}`, expected `{requirement.evidence_kind}`.",
                subject_paths=[raw_record],
                remediation="Keep evidence files and evidence_kind aligned.",
            )
        )
    if data.get("status") != "passed":
        findings.append(
            TruthFinding(
                finding_id=f"evidence-not-passed-{finding_slug(raw_record)}",
                message=f"{raw_record} is not passed evidence.",
                subject_paths=[raw_record],
                remediation="Only passed resolver-backed evidence may support truth certification.",
            )
        )

    subject_paths = data.get("subject_paths") or []
    if requirement.required_subject_paths and not subject_paths:
        findings.append(
            TruthFinding(
                finding_id=f"missing-subject-paths-{finding_slug(raw_record)}",
                message=f"{raw_record} does not name concrete subject_paths.",
                subject_paths=[raw_record],
                remediation="Evidence must name implementation, test, artifact, or diagnostic subjects; the evidence file cannot prove itself.",
            )
        )
    for subject in subject_paths:
        subject_text = str(subject)
        if subject_text == raw_record or subject_text.endswith(record.name):
            findings.append(
                TruthFinding(
                    finding_id=f"self-referential-evidence-{finding_slug(raw_record)}",
                    message=f"{raw_record} references itself as evidence subject.",
                    subject_paths=[raw_record],
                    remediation="Replace self-referential evidence with concrete code, test, or artifact subjects.",
                )
            )
        subject_path = repo_root / subject_text
        if not subject_path.exists():
            findings.append(
                TruthFinding(
                    finding_id=f"missing-evidence-subject-{finding_slug(subject_text)}",
                    message=f"{raw_record} references missing evidence subject {subject_text}.",
                    subject_paths=[raw_record, subject_text],
                    remediation="Evidence subjects must exist in the current repository state.",
                )
            )
            continue
        subject_digests = data.get("subject_digests") or {}
        if subject_text not in subject_digests:
            findings.append(
                TruthFinding(
                    finding_id=f"missing-subject-digest-{finding_slug(raw_record)}-{finding_slug(subject_text)}",
                    message=f"{raw_record} does not record a current digest for evidence subject {subject_text}.",
                    subject_paths=[raw_record, subject_text],
                    remediation="Regenerate resolver-backed evidence so every subject path has a recorded digest.",
                )
            )
            continue
        current_digest = digest_path(subject_path)
        if subject_digests[subject_text] != current_digest:
            findings.append(
                TruthFinding(
                    finding_id=f"stale-evidence-subject-{finding_slug(raw_record)}-{finding_slug(subject_text)}",
                    message=f"{raw_record} records stale evidence for {subject_text}.",
                    subject_paths=[raw_record, subject_text],
                    remediation="Refresh the resolver-backed evidence after changing the subject file.",
                )
            )

    provenance = data.get("validation_provenance") or []
    if not provenance:
        findings.append(
            TruthFinding(
                finding_id=f"missing-validation-result-{finding_slug(raw_record)}",
                message=f"{raw_record} does not record structured validation provenance.",
                subject_paths=[raw_record],
                remediation="Regenerate evidence through the execution kernel so it records command id, argv, return code, run ledger, action id, result digest, and subject digests.",
            )
        )
        return findings

    validation_text = "\n".join(" ".join(str(arg) for arg in item.get("argv") or []) for item in provenance)
    for index, item in enumerate(provenance):
        findings.extend(validate_validation_provenance(raw_record, index, item, data, repo_root=repo_root))
    for fragment in requirement.required_validation_fragments:
        if fragment not in validation_text:
            findings.append(
                TruthFinding(
                    finding_id=f"missing-validation-fragment-{finding_slug(raw_record)}-{finding_slug(fragment)}",
                    message=f"{raw_record} does not include required validation evidence `{fragment}`.",
                    subject_paths=[raw_record],
                    remediation="Record the exact validation command result that produced this evidence.",
                )
            )

    return findings


def validate_validation_provenance(raw_record: str, index: int, item: dict, data: dict, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    prefix = f"{raw_record}-validation-{index}"
    command_id = item.get("command_id")
    argv = item.get("argv")
    returncode = item.get("returncode")
    run_ledger_path = item.get("run_ledger_path")
    run_action_id = item.get("run_action_id")
    recorded_digest = item.get("validation_result_digest")
    subject_digests = item.get("subject_digests") or {}

    if not command_id or not isinstance(argv, list) or returncode is None or not run_ledger_path or not run_action_id or not recorded_digest:
        findings.append(
            TruthFinding(
                finding_id=f"malformed-validation-provenance-{finding_slug(prefix)}",
                message=f"{raw_record} has incomplete validation provenance entry {index}.",
                subject_paths=[raw_record],
                remediation="Validation provenance must include command_id, argv, returncode, run_ledger_path, run_action_id, validation_result_digest, and subject_digests.",
            )
        )
        return findings

    if returncode != 0:
        findings.append(
            TruthFinding(
                finding_id=f"validation-provenance-not-passed-{finding_slug(prefix)}",
                message=f"{raw_record} validation provenance `{command_id}` returned {returncode}.",
                subject_paths=[raw_record],
                remediation="Only successful typed validation commands may support truth evidence.",
            )
        )

    record_subject_digests = data.get("subject_digests") or {}
    if subject_digests != record_subject_digests:
        findings.append(
            TruthFinding(
                finding_id=f"validation-subject-digest-mismatch-{finding_slug(prefix)}",
                message=f"{raw_record} validation provenance subject digests do not match the evidence record subject digests.",
                subject_paths=[raw_record],
                remediation="Regenerate evidence so validation provenance and evidence subject digests describe the same current subjects.",
            )
        )

    ledger = repo_root / str(run_ledger_path)
    if not ledger.exists():
        findings.append(
            TruthFinding(
                finding_id=f"missing-validation-ledger-{finding_slug(prefix)}",
                message=f"{raw_record} references missing validation run ledger {run_ledger_path}.",
                subject_paths=[raw_record, str(run_ledger_path)],
                remediation="Evidence must resolve to a current execution run ledger action or an accepted typed validation-result artifact.",
            )
        )
        return findings
    try:
        ledger_data = yaml.safe_load(ledger.read_text(encoding="utf-8")) or {}
    except (OSError, yaml.YAMLError) as error:
        findings.append(
            TruthFinding(
                finding_id=f"malformed-validation-ledger-{finding_slug(prefix)}",
                message=f"Validation run ledger {run_ledger_path} cannot be read: {error}.",
                subject_paths=[raw_record, str(run_ledger_path)],
                remediation="Repair the machine-readable run ledger or regenerate evidence from a valid ledger action.",
            )
        )
        return findings

    action = next(
        (
            candidate
            for candidate in ledger_data.get("actions", [])
            if candidate.get("action_id") == run_action_id
            and candidate.get("status") == "passed"
        ),
        None,
    )
    if action is None:
        findings.append(
            TruthFinding(
                finding_id=f"missing-validation-ledger-action-{finding_slug(prefix)}",
                message=f"{raw_record} references run action {run_action_id}, but the ledger has no matching passed action.",
                subject_paths=[raw_record, str(run_ledger_path)],
                remediation="Evidence must point at the exact passed action that ran the validation.",
            )
        )
        return findings

    ledger_result = next(
        (
            result
            for result in action.get("validation_results", [])
            if result.get("command_id") == command_id
            and result.get("argv") == argv
            and result.get("returncode") == returncode
        ),
        None,
    )
    if ledger_result is None:
        findings.append(
            TruthFinding(
                finding_id=f"missing-validation-ledger-result-{finding_slug(prefix)}",
                message=f"{raw_record} validation provenance `{command_id}` does not match any result in {run_ledger_path}.",
                subject_paths=[raw_record, str(run_ledger_path)],
                remediation="Evidence provenance must match the ledger validation result exactly.",
            )
        )
        return findings

    expected_digest = validation_result_digest(
        command_id=command_id,
        argv=argv,
        returncode=returncode,
        files_changed=ledger_result.get("files_changed") or [],
        subject_digests=subject_digests,
    )
    if recorded_digest != expected_digest:
        findings.append(
            TruthFinding(
                finding_id=f"stale-validation-result-digest-{finding_slug(prefix)}",
                message=f"{raw_record} records a stale validation_result_digest for `{command_id}`.",
                subject_paths=[raw_record, str(run_ledger_path)],
                remediation="Regenerate evidence after validation output, subject digests, or ledger content changes.",
            )
        )

    return findings
