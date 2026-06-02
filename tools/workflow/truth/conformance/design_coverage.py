from __future__ import annotations

from pathlib import Path

from roadmap_state import REPO_ROOT, repo_path

from truth.certificates import TruthFinding
from truth.conformance.rust_module_shape import finding_slug
from truth.conformance.spec import ConformanceSpec, load_design_coverage_matrix_file


VERIFIED_STATUSES = {"verified"}
DEFERRED_STATUSES = {"deferred", "blocked"}


def verify_design_coverage(spec: ConformanceSpec, *, repo_root: Path = REPO_ROOT) -> tuple[list[TruthFinding], list[str]]:
    findings: list[TruthFinding] = []
    checks: list[str] = []
    if not spec.design_coverage_path:
        return [
            TruthFinding(
                finding_id="missing-design-coverage-matrix",
                message="Conformance spec does not declare a requirement-by-requirement design coverage matrix.",
                subject_paths=[],
                remediation="Add design_coverage_path and bind every accepted design requirement to code, tests, evidence, and semantic verifier IDs.",
            )
        ], checks

    matrix = load_design_coverage_matrix_file(spec.design_coverage_path, repo_root=repo_root)
    checks.append("design requirement coverage matrix is machine-readable and bound to the conformance spec")
    if matrix.track_id != spec.track_id or matrix.spec_id != spec.spec_id:
        findings.append(
            TruthFinding(
                finding_id="design-coverage-spec-mismatch",
                message=(
                    f"Design coverage matrix is for {matrix.track_id}/{matrix.spec_id}, "
                    f"not {spec.track_id}/{spec.spec_id}."
                ),
                subject_paths=[spec.design_coverage_path],
                remediation="Bind the requirement coverage matrix to the exact track and conformance spec.",
            )
        )

    matrix_docs = set(matrix.source_design_docs)
    for design_doc in spec.required_design_docs:
        if design_doc not in matrix_docs:
            findings.append(
                TruthFinding(
                    finding_id=f"design-coverage-missing-design-doc-{finding_slug(design_doc)}",
                    message=f"Design coverage matrix does not cite required design doc {design_doc}.",
                    subject_paths=[spec.design_coverage_path],
                    remediation="Add the design doc to source_design_docs so coverage is anchored in accepted design authority.",
                )
            )

    requirement_ids = [requirement.requirement_id for requirement in matrix.requirements]
    duplicate_ids = sorted({requirement_id for requirement_id in requirement_ids if requirement_ids.count(requirement_id) > 1})
    for requirement_id in duplicate_ids:
        findings.append(
            TruthFinding(
                finding_id=f"duplicate-design-requirement-{finding_slug(requirement_id)}",
                message=f"Design requirement {requirement_id} appears more than once.",
                subject_paths=[spec.design_coverage_path],
                remediation="Keep design requirement IDs stable and unique.",
            )
        )

    coverage_by_id = {requirement.requirement_id: requirement for requirement in matrix.requirements}
    for requirement_id in spec.required_design_requirement_ids:
        if requirement_id not in coverage_by_id:
            findings.append(
                TruthFinding(
                    finding_id=f"missing-design-requirement-{finding_slug(requirement_id)}",
                    message=f"Required accepted design requirement {requirement_id} is missing from the coverage matrix.",
                    subject_paths=[spec.design_coverage_path],
                    remediation="Add the requirement with code subjects, tests, evidence, and semantic verifier coverage.",
                )
            )

    declared_semantic_checks = {semantic_check.check_id for semantic_check in spec.semantic_checks}
    declared_evidence_kinds = {requirement.evidence_kind for requirement in spec.evidence_requirements}
    for requirement in matrix.requirements:
        source_path = repo_root / requirement.source_path
        if not source_path.exists():
            findings.append(
                TruthFinding(
                    finding_id=f"design-requirement-missing-source-{finding_slug(requirement.requirement_id)}",
                    message=f"Design requirement {requirement.requirement_id} cites missing source doc {requirement.source_path}.",
                    subject_paths=[spec.design_coverage_path, requirement.source_path],
                    remediation="Point the requirement at an accepted design source that exists.",
                )
            )
        if requirement.status in VERIFIED_STATUSES:
            findings.extend(_verified_requirement_findings(requirement, spec.design_coverage_path, repo_root=repo_root))
        elif requirement.status in DEFERRED_STATUSES:
            if not requirement.deferral_authority:
                findings.append(
                    TruthFinding(
                        finding_id=f"design-requirement-missing-deferral-{finding_slug(requirement.requirement_id)}",
                        message=f"Deferred or blocked requirement {requirement.requirement_id} lacks deferral_authority.",
                        subject_paths=[spec.design_coverage_path],
                        remediation="Attach accepted deferral authority or do not mark the requirement deferred.",
                    )
                )
        else:
            findings.append(
                TruthFinding(
                    finding_id=f"design-requirement-invalid-status-{finding_slug(requirement.requirement_id)}",
                    message=f"Design requirement {requirement.requirement_id} has unsupported status {requirement.status!r}.",
                    subject_paths=[spec.design_coverage_path],
                    remediation="Use verified, deferred, or blocked with explicit authority.",
                )
            )

        for semantic_check_id in requirement.semantic_check_ids:
            if semantic_check_id not in declared_semantic_checks:
                findings.append(
                    TruthFinding(
                        finding_id=f"design-requirement-missing-semantic-check-{finding_slug(requirement.requirement_id)}-{finding_slug(semantic_check_id)}",
                        message=f"Design requirement {requirement.requirement_id} cites unknown semantic check {semantic_check_id}.",
                        subject_paths=[spec.design_coverage_path],
                        remediation="Bind the requirement to a declared semantic check with an executable verifier.",
                    )
                )
        for evidence_kind in requirement.evidence_kinds:
            if evidence_kind not in declared_evidence_kinds:
                findings.append(
                    TruthFinding(
                        finding_id=f"design-requirement-missing-evidence-kind-{finding_slug(requirement.requirement_id)}-{finding_slug(evidence_kind)}",
                        message=f"Design requirement {requirement.requirement_id} cites undeclared evidence kind {evidence_kind}.",
                        subject_paths=[spec.design_coverage_path],
                        remediation="Declare resolver-backed evidence for the requirement's proof category.",
                    )
                )

    checks.append("verified design requirements cite concrete code subjects, tests, evidence, and semantic verifiers")
    return findings, checks


def _verified_requirement_findings(requirement, coverage_path: str, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    missing_fields = []
    if not requirement.code_subjects:
        missing_fields.append("code_subjects")
    if not requirement.test_subjects:
        missing_fields.append("test_subjects")
    if not requirement.evidence_kinds:
        missing_fields.append("evidence_kinds")
    if not requirement.semantic_check_ids:
        missing_fields.append("semantic_check_ids")
    if missing_fields:
        findings.append(
            TruthFinding(
                finding_id=f"design-requirement-incomplete-proof-{finding_slug(requirement.requirement_id)}",
                message=(
                    f"Verified requirement {requirement.requirement_id} lacks "
                    f"{', '.join(missing_fields)}."
                ),
                subject_paths=[coverage_path],
                remediation="Verified design requirements must bind to code, tests, evidence, and semantic verifier IDs.",
            )
        )
    for raw_path in [*requirement.code_subjects, *requirement.test_subjects]:
        path = repo_root / raw_path
        if not path.exists():
            findings.append(
                TruthFinding(
                    finding_id=f"design-requirement-missing-subject-{finding_slug(requirement.requirement_id)}-{finding_slug(raw_path)}",
                    message=f"Requirement {requirement.requirement_id} cites missing subject {raw_path}.",
                    subject_paths=[coverage_path, raw_path],
                    remediation="Repair the subject path or do not claim the requirement is verified.",
                )
            )
    return findings
