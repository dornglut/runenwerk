from __future__ import annotations

from pathlib import Path

from roadmap_state import REPO_ROOT, repo_path

from truth.certificates import TruthFinding
from truth.conformance.design_coverage import verify_design_coverage
from truth.conformance.evidence import verify_evidence_records
from truth.conformance.rust_module_shape import read_text, verify_rust_module_shape
from truth.conformance.rust_module_shape import finding_slug
from truth.conformance.spec import ConformanceSpec, load_conformance_spec_file


def require_text(
    *,
    repo_root: Path,
    path: str,
    needles: list[str],
    check_id: str,
) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    subject = repo_root / path
    if not subject.exists():
        return [
            TruthFinding(
                finding_id=f"semantic-check-missing-subject-{finding_slug(path)}",
                message=f"Semantic check `{check_id}` cites missing subject {path}.",
                subject_paths=[path],
                remediation="Repair the subject path or update the conformance spec.",
            )
        ]
    text = read_text(subject)
    compact_text = "".join(text.split())
    for needle in needles:
        compact_needle = "".join(needle.split())
        if needle not in text and compact_needle not in compact_text:
            findings.append(
                TruthFinding(
                    finding_id=f"semantic-check-missing-behavior-{finding_slug(check_id)}-{finding_slug(needle)}",
                    message=f"Semantic check `{check_id}` requires `{needle}` in {path}.",
                    subject_paths=[path],
                    remediation="Restore the behavioral assertion or implementation path required by the conformance spec.",
                )
            )
    return findings


def verify_graph_family_lowering_to_runtime_tables(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_artifacts/src/tables/mod.rs",
            check_id="graph_family_lowering_to_runtime_tables",
            needles=[
                "ControlTable::from_program(program, source_map)",
                "LayoutPlanTable::from_program(program, source_map)",
                "StyleResolutionTable::from_program(program, source_map)",
                "StateTable::from_program(program, source_map)",
                "InteractionDispatchTable::from_program(program, source_map)",
                "BindingSnapshotTable::from_program(program, source_map)",
                "CollectionDiffPlan::from_program(program)",
                "VisualOperatorTable::from_program(program, source_map)",
                "TextLayoutRequestTable::from_program(program, source_map)",
                "AccessibilityTable::from_program(program, source_map)",
                "InspectionTable::from_program(program, source_map)",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_artifacts/src/tests.rs",
            check_id="graph_family_lowering_to_runtime_tables",
            needles=[
                "artifact_contract_splits_manifest_from_typed_runtime_tables",
                "artifact.tables.controls.rows.len()",
                "artifact.tables.layout.rows.len()",
                "artifact.tables.style.rows.len()",
                "artifact.tables.state.rows.len()",
                "artifact.tables.interaction.rows.len()",
                "artifact.tables.binding_snapshots.rows.len()",
                "artifact.tables.collection_diffs.rows.len()",
                "artifact.tables.visual.rows.len()",
                "artifact.tables.text_layout_requests.rows.len()",
                "artifact.tables.accessibility.rows.len()",
                "artifact.tables.inspection.rows.len()",
            ],
        )
    )
    return findings


def verify_graph_family_semantic_contracts(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    graph_contracts = {
        "domain/ui/ui_program/src/graphs/control.rs": [
            "pub struct ControlGraph",
            "pub struct ControlGraphNode",
            "pub required_capabilities: Vec<RouteCapability>",
        ],
        "domain/ui/ui_program/src/graphs/layout.rs": [
            "pub struct LayoutGraph",
            "pub struct LayoutGraphNode",
            "pub layout_kernel: Option<ControlKernelRef>",
        ],
        "domain/ui/ui_program/src/graphs/style.rs": [
            "pub struct StyleGraph",
            "pub struct StyleRule",
            "pub style_slot: StyleSlotId",
            "pub property_schema: UiSchemaRef",
        ],
        "domain/ui/ui_program/src/graphs/state.rs": [
            "pub struct StateGraph",
            "pub struct StateRequirement",
            "pub enum StateRequirementLifecycle",
        ],
        "domain/ui/ui_program/src/graphs/interaction.rs": [
            "pub struct InteractionGraph",
            "pub struct InteractionHandler",
            "pub trigger: InteractionTrigger",
            "pub route: RouteId",
            "pub payload_schema: UiSchemaRef",
            "pub required_capabilities: Vec<RouteCapability>",
        ],
        "domain/ui/ui_program/src/graphs/binding.rs": [
            "pub struct BindingGraph",
            "pub struct BindingEdge",
            "pub enum BindingEndpoint",
            "ControlProperty",
            "UiState",
            "HostData",
        ],
        "domain/ui/ui_program/src/graphs/visual.rs": [
            "pub struct VisualGraph",
            "pub struct VisualOperator",
            "pub visual_kernel: ControlKernelRef",
            "pub input_dependencies: Vec<ControlNodeId>",
        ],
        "domain/ui/ui_program/src/graphs/accessibility.rs": [
            "pub struct AccessibilityGraph",
            "pub struct AccessibilityNode",
            "pub enum AccessibilityRole",
            "pub label_source: Option<BindingEndpointId>",
        ],
        "domain/ui/ui_program/src/graphs/inspection.rs": [
            "pub struct InspectionGraph",
            "pub struct InspectionEntry",
            "pub value_schema: UiSchemaRef",
            "pub binding: Option<BindingEndpointId>",
        ],
    }
    for path, needles in graph_contracts.items():
        findings.extend(
            require_text(
                repo_root=repo_root,
                path=path,
                check_id="graph_family_semantic_contracts",
                needles=needles,
            )
        )
    return findings


def verify_control_package_design_truth(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_controls/src/package.rs",
            check_id="control_package_design_truth",
            needles=[
                "pub property_schema: UiSchemaRef",
                "pub state_schema: UiSchemaRef",
                "pub event_payload_schema: UiSchemaRef",
                "pub kernels: ControlKernelSet",
                "pub fixture_ids: Vec<ControlFixtureId>",
                "pub diagnostic_ids: Vec<ControlDiagnosticId>",
                "pub migration_ids: Vec<ControlMigrationId>",
                "pub fn with_fixture",
                "pub fn with_diagnostic",
                "pub fn with_migration",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_controls/src/registry.rs",
            check_id="control_package_design_truth",
            needles=[
                "pub struct ControlPackageRegistry",
                "pub fn register",
                "pub fn snapshot",
                "ControlPackageRegistrySnapshot",
                "DuplicateControlKind",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_controls/src/color_picker/mod.rs",
            check_id="control_package_design_truth",
            needles=[
                "COLOR_PICKER_CONTROL_KIND_ID",
                "committed_rgba",
                "triangle_saturation",
                "triangle_value",
                "preview_rgba",
                "UiSchemaShape::RouteRef",
                "runenwerk.ui.controls.color.write",
            ],
        )
    )
    return findings


def verify_route_event_schema_payload_validation(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_program/src/events/payload.rs",
            check_id="route_event_schema_payload_validation",
            needles=[
                "pub value: UiSchemaValue",
                "pub fn validation_report(&self, schema: &UiSchema)",
                "schema.validate(&self.value)",
                "ui.event.payload_schema_mismatch",
                "pub fn is_valid_for(&self, schema: &UiSchema) -> bool",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_program/src/events/packet.rs",
            check_id="route_event_schema_payload_validation",
            needles=[
                "pub route: RouteId",
                "pub schema_version: RouteSchemaVersion",
                "pub payload: UiEventPayload",
                "pub fn payload_schema(&self) -> &UiSchemaRef",
                "pub fn with_payload_validation(mut self, schema: &UiSchema)",
                "pub fn requires_capability(&self, capability: &RouteCapability) -> bool",
            ],
        )
    )
    return findings


def verify_source_map_and_diagnostics_preservation(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_artifacts/src/source_map.rs",
            check_id="source_map_and_diagnostics_preservation",
            needles=[
                "pub fn from_program(program: &UiProgram) -> Self",
                "RuntimeTableKind::Control",
                "RuntimeTableKind::Layout",
                "RuntimeTableKind::BindingSnapshot",
                "RuntimeTableKind::Accessibility",
                "RuntimeTableKind::Inspection",
                "pub fn index_for_entry(&self, source_map: &UiProgramSourceMapEntry)",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_artifacts/src/tests.rs",
            check_id="source_map_and_diagnostics_preservation",
            needles=[
                "artifact_contract_preserves_source_maps_routes_and_collection_diffs",
                "RuntimeTableKind::Control",
                "definition.title",
                "CollectionDiffStrategy::ReplaceValue",
            ],
        )
    )
    return findings


def verify_evaluator_consumes_artifact_tables_and_state_binding(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_evaluator/src/evaluator.rs",
            check_id="evaluator_consumes_artifact_tables_and_state_binding",
            needles=[
                "artifact.tables.controls",
                "artifact.tables.state",
                "artifact.tables.binding_snapshots",
                "apply_dirty_bindings_to_state",
                "UiOutput",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_evaluator/src/tests.rs",
            check_id="evaluator_consumes_artifact_tables_and_state_binding",
            needles=[
                "evaluator_contract_projects_typed_artifact_tables_and_updates_boundary_state",
                "state.value(\"state.score.value\")",
                "output.binding.dirty_report.dirty_bindings",
                "output.visual.text_layout_requests",
                "output.accessibility.rows",
                "output.inspection.rows",
            ],
        )
    )
    return findings


def verify_retained_ui_compatibility_and_render_boundary(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_evaluator/src/output.rs",
            check_id="retained_ui_compatibility_and_render_boundary",
            needles=[
                "pub struct UiOutput",
                "pub controls: ControlEvaluationPass",
                "pub visual: VisualEvaluationPass",
                "pub accessibility: AccessibilityEvaluationPass",
                "pub inspection: InspectionEvaluationPass",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_testing/src/tests.rs",
            check_id="retained_ui_compatibility_and_render_boundary",
            needles=[
                "run.accessibility.source_mapped_count()",
                "run.geometry.source_mapped_count()",
                "run.output",
            ],
        )
    )
    return findings


def verify_headless_fixture_reproducibility(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_testing/src/tests.rs",
            check_id="headless_fixture_reproducibility",
            needles=[
                "architecture_fixtures_compile_evaluate_assert_and_reproduce",
                "run.source_map_assertion",
                "run.diagnostic_assertion",
                "run.reproducibility_assertion.passed()",
                "run.passed()",
            ],
        )
    )
    findings.extend(
        require_text(
            repo_root=repo_root,
            path="domain/ui/ui_testing/src/headless_fixture.rs",
            check_id="headless_fixture_reproducibility",
            needles=[
                "SourceMapAssertion::target_in_table",
                "DiagnosticAssertion::code_absent",
                "ReproducibilityAssertion::from_fixture",
            ],
        )
    )
    return findings


SEMANTIC_VERIFIERS = {
    "graph_family_lowering_to_runtime_tables": verify_graph_family_lowering_to_runtime_tables,
    "graph_family_semantic_contracts": verify_graph_family_semantic_contracts,
    "control_package_design_truth": verify_control_package_design_truth,
    "route_event_schema_payload_validation": verify_route_event_schema_payload_validation,
    "source_map_and_diagnostics_preservation": verify_source_map_and_diagnostics_preservation,
    "evaluator_consumes_artifact_tables_and_state_binding": verify_evaluator_consumes_artifact_tables_and_state_binding,
    "retained_ui_compatibility_and_render_boundary": verify_retained_ui_compatibility_and_render_boundary,
    "headless_fixture_reproducibility": verify_headless_fixture_reproducibility,
}


def verify(
    *,
    track_id: str,
    claim_id: str,
    spec_path: str,
    repo_root: Path = REPO_ROOT,
) -> tuple[list[TruthFinding], list[str]]:
    spec = load_conformance_spec_file(spec_path, repo_root=repo_root)
    findings: list[TruthFinding] = []
    checks: list[str] = []
    findings.extend(verify_spec_binding(spec, track_id=track_id, claim_id=claim_id, spec_path=spec_path))
    checks.append("conformance spec is registered for the requested track and truth claim")

    module_findings, module_checks = verify_rust_module_shape(spec, repo_root=repo_root)
    findings.extend(module_findings)
    checks.extend(module_checks)

    findings.extend(verify_owner_map(spec, repo_root=repo_root))
    checks.append("final UI owner map directories exist and are reconciled with design docs")

    findings.extend(verify_design_terms(spec, repo_root=repo_root))
    checks.append("accepted UI architecture design declares every conformance owner and boundary term")

    coverage_findings, coverage_checks = verify_design_coverage(spec, repo_root=repo_root)
    findings.extend(coverage_findings)
    checks.extend(coverage_checks)

    evidence_findings, evidence_checks = verify_evidence_records(spec, repo_root=repo_root)
    findings.extend(evidence_findings)
    checks.extend(evidence_checks)

    semantic_findings, semantic_checks = verify_semantic_checks(spec, repo_root=repo_root)
    findings.extend(semantic_findings)
    checks.extend(semantic_checks)

    findings.extend(verify_no_catch_all_owner_files(spec, repo_root=repo_root))
    checks.append("final UI owner crates do not use catch-all helpers/utils/misc files")

    findings.extend(verify_forbidden_code_terms(spec, repo_root=repo_root))
    checks.append("domain UI code avoids forbidden ownership and extraction terms")

    if spec.zero_gap_criteria:
        checks.append("zero-gap criteria are declared by the conformance spec")

    return findings, checks


def verify_spec_binding(spec: ConformanceSpec, *, track_id: str, claim_id: str, spec_path: str) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    if spec.track_id != track_id:
        findings.append(
            TruthFinding(
                finding_id=f"spec-track-mismatch-{finding_slug(track_id)}",
                message=f"Conformance spec {spec_path} is for {spec.track_id}, not {track_id}.",
                subject_paths=[spec_path],
                remediation="Register a conformance spec whose track_id matches the requested truth claim.",
            )
        )
    if claim_id not in spec.claim_ids:
        findings.append(
            TruthFinding(
                finding_id=f"spec-claim-missing-{finding_slug(claim_id)}",
                message=f"Conformance spec {spec_path} does not cover claim {claim_id}.",
                subject_paths=[spec_path],
                remediation="Add the claim to the conformance spec or bind the claim to the correct spec.",
            )
        )
    return findings


def verify_owner_map(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    for raw_path in spec.final_owner_dirs:
        path = repo_root / raw_path
        cargo = path / "Cargo.toml"
        if not path.exists() or not cargo.exists():
            findings.append(
                TruthFinding(
                    finding_id=f"missing-owner-{raw_path.replace('/', '-')}",
                    message=f"Final UI owner `{raw_path}` is missing or lacks Cargo.toml.",
                    subject_paths=[raw_path],
                    remediation="Create the exact owner crate through a locked WR or explicitly reconcile why the owner is not part of this truth claim.",
                )
            )
    return findings


def verify_semantic_checks(spec: ConformanceSpec, *, repo_root: Path) -> tuple[list[TruthFinding], list[str]]:
    findings: list[TruthFinding] = []
    checks: list[str] = []
    if not spec.semantic_checks:
        findings.append(
            TruthFinding(
                finding_id="missing-semantic-conformance-checks",
                message="Conformance spec declares no semantic checks beyond file and symbol presence.",
                subject_paths=[],
                remediation="Declare semantic checks that cite code subjects, validation fragments, and resolver-backed evidence categories.",
            )
        )
        return findings, checks

    checks.append("semantic conformance checks dispatch through registered verifier functions")
    declared_evidence_kinds = {
        (requirement.milestone_id, requirement.evidence_kind)
        for requirement in spec.evidence_requirements
    }
    all_validation_fragments = {
        fragment
        for requirement in spec.evidence_requirements
        for fragment in requirement.required_validation_fragments
    }
    for semantic_check in spec.semantic_checks:
        verifier = SEMANTIC_VERIFIERS.get(semantic_check.check_id)
        if verifier is None:
            findings.append(
                TruthFinding(
                    finding_id=f"semantic-check-missing-verifier-{finding_slug(semantic_check.check_id)}",
                    message=f"Semantic check `{semantic_check.check_id}` has no registered verifier.",
                    subject_paths=[],
                    remediation="Bind every semantic_check_id to an executable verifier function.",
                )
            )
        else:
            findings.extend(verifier(spec, repo_root=repo_root))
        subject_text = ""
        if not semantic_check.subject_paths:
            findings.append(
                TruthFinding(
                    finding_id=f"semantic-check-missing-subjects-{finding_slug(semantic_check.check_id)}",
                    message=f"Semantic check `{semantic_check.check_id}` does not cite code subjects.",
                    subject_paths=[],
                    remediation="Every semantic check must name concrete source/test files that carry the behavior.",
                )
            )
        for raw_path in semantic_check.subject_paths:
            path = repo_root / raw_path
            if not path.exists():
                findings.append(
                    TruthFinding(
                        finding_id=f"semantic-check-missing-subject-{finding_slug(raw_path)}",
                        message=f"Semantic check `{semantic_check.check_id}` cites missing subject {raw_path}.",
                        subject_paths=[raw_path],
                        remediation="Repair the subject path or update the conformance spec.",
                    )
                )
                continue
            subject_text += "\n" + read_text(path)
        for symbol in semantic_check.required_symbols:
            if symbol not in subject_text:
                findings.append(
                    TruthFinding(
                        finding_id=f"semantic-check-missing-symbol-{finding_slug(semantic_check.check_id)}-{finding_slug(symbol)}",
                        message=f"Semantic check `{semantic_check.check_id}` requires cited subjects to expose `{symbol}`.",
                        subject_paths=list(semantic_check.subject_paths),
                        remediation="Keep semantic behavior in the cited owner files or update the accepted spec.",
                    )
                )
        for evidence_kind in semantic_check.evidence_kinds:
            if not any(kind == evidence_kind for _milestone, kind in declared_evidence_kinds):
                findings.append(
                    TruthFinding(
                        finding_id=f"semantic-check-missing-evidence-kind-{finding_slug(semantic_check.check_id)}-{finding_slug(evidence_kind)}",
                        message=f"Semantic check `{semantic_check.check_id}` requires undeclared evidence kind `{evidence_kind}`.",
                        subject_paths=[],
                        remediation="Declare resolver-backed evidence for this semantic behavior.",
                    )
                )
        for fragment in semantic_check.required_validation_fragments:
            if fragment not in all_validation_fragments:
                findings.append(
                    TruthFinding(
                        finding_id=f"semantic-check-missing-validation-{finding_slug(semantic_check.check_id)}-{finding_slug(fragment)}",
                        message=f"Semantic check `{semantic_check.check_id}` is missing validation fragment `{fragment}` in evidence requirements.",
                        subject_paths=[repo_path(repo_root / spec.required_design_docs[0])] if spec.required_design_docs else [],
                        remediation="Add exact validation command evidence for the semantic check.",
                    )
                )
    return findings, checks


def verify_design_terms(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    design_text = "\n".join(read_text(repo_root / path) for path in spec.required_design_docs)
    for term in spec.required_design_terms:
        if term not in design_text:
            findings.append(
                TruthFinding(
                    finding_id=f"missing-design-term-{term.replace('_', '-').replace('/', '-')}",
                    message=f"Accepted UI architecture docs do not declare required term `{term}`.",
                    subject_paths=spec.required_design_docs,
                    remediation="Update the accepted design or conformance spec so design intent and verification target agree.",
                )
            )
    return findings


def verify_forbidden_code_terms(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    rust_files = sorted((repo_root / "domain/ui").glob("**/*.rs"))
    for path in rust_files:
        text = read_text(path)
        for term in spec.forbidden_code_terms:
            if term in text:
                findings.append(
                    TruthFinding(
                        finding_id=f"forbidden-code-term-{term.replace('/', '-')}-{path.stem}",
                        message=f"{path.relative_to(repo_root)} contains forbidden architecture term `{term}`.",
                        subject_paths=[str(path.relative_to(repo_root))],
                        remediation="Remove forbidden ownership/extraction coupling before architecture certification.",
                    )
                )
    return findings


def verify_no_catch_all_owner_files(spec, *, repo_root: Path) -> list[TruthFinding]:
    findings: list[TruthFinding] = []
    forbidden_names = {"helpers.rs", "utils.rs", "misc.rs"}
    for owner in spec.final_owner_dirs:
        src = repo_root / owner / "src"
        if not src.exists():
            continue
        for path in sorted(src.rglob("*.rs")):
            if path.name not in forbidden_names:
                continue
            raw_path = repo_path(path)
            findings.append(
                TruthFinding(
                    finding_id=f"catch-all-owner-file-{finding_slug(raw_path)}",
                    message=f"{raw_path} is a catch-all owner file in a final UI owner crate.",
                    subject_paths=[raw_path],
                    remediation="Split the file by subdomain responsibility or add an explicit accepted design waiver to the conformance spec.",
                )
            )
    return findings
