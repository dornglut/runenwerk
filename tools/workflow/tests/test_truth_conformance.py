from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from workflow_fixtures import *
from execution.evidence import validation_result_digest
from truth.certificates import digest_path
from truth.conformance.design_coverage import verify_design_coverage
from truth.conformance.evidence import validate_evidence_record
from truth.conformance.ui_program_architecture import verify_semantic_checks
from truth.registry import load_truth_verifier_registry


def test_manifest_backed_validation_requires_truth_claims(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data.pop("truth_claims")
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("manifest-backed tracks must declare truth_claims" in error for error in errors)

def test_completed_track_rejects_remaining_blocked_architecture_truth_claim(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["state"] = "completed"
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "retained-compatibility-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Retained compatibility is still blocked.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Retained compatibility remains unverified."],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any(
        "completed tracks must not retain blocked architecture_contract truth claim retained-compatibility-test"
        in error
        for error in errors
    )

def test_satisfied_strong_truth_claim_requires_current_certificate(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Architecture exists.",
            "truth_verifier": "ui_program_architecture_conformance",
            "truth_certificate_path": "docs-site/src/content/docs/reports/truth-certificates/pt-test/architecture-test.yaml",
            "required_docs": [
                {
                    "evidence_kind": "doc_exists",
                    "path": "tools/workflow/test_workflow.py",
                    "reason": "Fixture truth evidence path exists.",
                }
            ],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("requires current certificate" in error for error in errors)

def test_perfectionist_truth_certificate_must_be_zero_finding(tmp_path: Path) -> None:
    claim = ManifestTruthClaim.model_validate(
        {
            "claim_id": "perfect-test",
            "claim_kind": "architecture_contract",
            "claim_level": "perfectionist_verified",
            "claim_status": "satisfied",
            "claim_statement": "Everything is perfect.",
            "truth_verifier": "ui_program_architecture_conformance",
            "truth_certificate_path": "docs-site/src/content/docs/reports/truth-certificates/pt-test/perfect-test.yaml",
            "required_docs": [
                {
                    "evidence_kind": "doc_exists",
                    "path": "tools/workflow/test_workflow.py",
                    "reason": "Fixture truth evidence path exists.",
                }
            ],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    )
    write_certificate(
        TruthCertificate(
            track_id="PT-TEST",
            claim_id="perfect-test",
            verifier="ui_program_architecture_conformance",
            status="failed",
            produced_at="2026-06-01T00:00:00Z",
            source_digests={"tools/workflow/test_workflow.py": "not-current"},
            findings=[
                TruthFinding(
                    finding_id="gap",
                    message="A finding remains.",
                    remediation="Fix it before certification.",
                )
            ],
        ),
        root=tmp_path / "truth-certificates",
    )

    errors = certificate_errors_for_claim("PT-TEST", claim, root=tmp_path / "truth-certificates")

    assert any("status is failed" in error for error in errors)
    assert any("not zero-finding" in error for error in errors)

def test_satisfied_architecture_truth_claim_requires_code_contracts(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Architecture exists.",
            "required_docs": [],
            "required_code_contracts": [
                {
                    "evidence_kind": "module_path_exists",
                    "path": "domain/missing_test_architecture_module",
                    "reason": "Architecture module must exist.",
                }
            ],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("requires module path domain/missing_test_architecture_module" in error for error in errors)

def test_blocked_truth_claim_blocks_downstream_milestone(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"].append(
        {
            "claim_id": "blocked-architecture",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Architecture is not ready.",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": ["Architecture module missing."],
            "supersedes": [],
            "blocks_downstream": ["PM-TEST-002"],
        }
    )
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(valid_production_state())

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("truth claim blocked-architecture blocks downstream PM-TEST-002" in error for error in errors)

def test_manifest_next_surfaces_truth_claims(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())

    result = CliRunner().invoke(
        production_track_cli_app,
        [
            "next",
            "--track",
            "PT-TEST",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
            "--contract-pack-root",
            str(tmp_path / "contract-packs"),
        ],
    )

    assert result.exit_code == 0
    assert "Truth claims:" in result.stdout
    assert "test-track-proof" in result.stdout

def test_goal_output_surfaces_truth_claims(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    write_yaml(production_path, valid_production_state())
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-TEST",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0
    assert "Truth claims:" in result.stdout
    assert "test-track-proof" in result.stdout

def test_goal_output_surfaces_truth_certificate_status(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Architecture exists.",
            "truth_verifier": "ui_program_architecture_conformance",
            "truth_certificate_path": "docs-site/src/content/docs/reports/truth-certificates/pt-test/architecture-test.yaml",
            "required_docs": [
                {
                    "evidence_kind": "doc_exists",
                    "path": "tools/workflow/test_workflow.py",
                    "reason": "Fixture truth evidence path exists.",
                }
            ],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        production_goal_app,
        [
            "goal",
            "--track",
            "PT-TEST",
            "--production-source",
            str(production_path),
            "--roadmap-source",
            str(roadmap_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 1
    assert "truth claim errors" in result.stdout
    assert "architecture-test" in result.stdout
    assert "requires current certificate" in result.stdout

def test_truth_audit_surfaces_blocked_verifier_status(tmp_path: Path) -> None:
    production_path = tmp_path / "production.yaml"
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "architecture-test",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "blocked",
            "claim_statement": "Architecture remains blocked until code truth passes.",
            "truth_verifier": "ui_program_architecture_conformance",
            "required_docs": [],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
                "known_gaps": ["Claim intentionally remains blocked until a closeout satisfies it."],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(production_path, production_data)
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)

    result = CliRunner().invoke(
        truth_app,
        [
            "audit",
            "--track",
            "PT-TEST",
            "--production-source",
            str(production_path),
            "--manifest-source-root",
            str(manifest_root),
        ],
    )

    assert result.exit_code == 0, result.output
    assert "blocked claims remain uncertified" in result.stdout
    assert "Blocked verifier status:" in result.stdout
    assert "architecture-test: verifier error" in result.stdout
    assert "not registered for claim" in result.stdout

def test_truth_validation_commands_require_exact_argument_shape() -> None:
    valid = validation_command_from_string(
        "task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-architecture-implementation"
    )
    invalid = validation_command_from_string(
        "task truth:verify -- --track PT-UI-PROGRAM-ARCHITECTURE --claim ui-program-architecture-implementation --extra unsafe"
    )

    assert valid["command_id"] == "task:truth:verify"
    assert invalid["command_id"] == "blocked"
    assert invalid["blocked_reason"] == "validation command is not in the safe command registry"

def test_ui_program_architecture_verifier_reports_current_conformance_status() -> None:
    certificate = run_verifier(
        track_id="PT-UI-PROGRAM-ARCHITECTURE",
        claim_id="ui-program-architecture-implementation",
        verifier="ui_program_architecture_conformance",
    )

    assert certificate.status in {"passed", "failed"}
    assert "conformance required source files exist and expose declared symbols" in certificate.checks
    assert "conformance evidence records are resolver-backed and non-self-referential" in certificate.checks
    if certificate.status == "passed":
        assert certificate.findings == []
        assert certificate.known_gaps == []
        assert certificate.known_risks == []
        assert certificate.truth_drift == []
    else:
        assert certificate.findings

def test_ui_program_architecture_verifier_sources_include_conformance_spec() -> None:
    sources = verifier_source_paths(
        "ui_program_architecture_conformance",
        track_id="PT-UI-PROGRAM-ARCHITECTURE",
        claim_id="ui-program-architecture-implementation",
    )

    assert (
        "docs-site/src/content/docs/workspace/truth-conformance-specs/pt-ui-program-architecture/ui-program-architecture-implementation.yaml"
        in sources
    )
    assert (
        "docs-site/src/content/docs/workspace/design-conformance/pt-ui-program-architecture.requirements.yaml"
        in sources
    )
    assert "docs-site/src/content/docs/workspace/production-tracks.yaml" in sources
    assert "docs-site/src/content/docs/workspace/execution-contract-packs/pt-ui-program-architecture.yaml" in sources
    assert "tools/workflow/truth" in sources
    assert "docs-site/src/content/docs/workspace/truth-verifier-registry.yaml" in sources
    assert "docs-site/src/content/docs/reports/track-execution-runs/pt-ui-program-architecture" not in sources
    assert not any(source.startswith("docs-site/src/content/docs/reports/track-execution-runs/") for source in sources)


def test_certificate_errors_require_complete_registry_digest_closure(tmp_path: Path) -> None:
    claim = ManifestTruthClaim.model_validate(
        {
            "claim_id": "ui-program-architecture-implementation",
            "claim_kind": "architecture_contract",
            "claim_level": "architecture_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Architecture exists.",
            "truth_verifier": "ui_program_architecture_conformance",
            "truth_certificate_path": "docs-site/src/content/docs/reports/truth-certificates/pt-ui-program-architecture/ui-program-architecture-implementation.yaml",
            "required_docs": [
                {
                    "evidence_kind": "doc_exists",
                    "path": "docs-site/src/content/docs/design/active/ui-program-architecture.md",
                    "reason": "Architecture design must exist.",
                }
            ],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    )
    write_certificate(
        TruthCertificate(
            track_id="PT-UI-PROGRAM-ARCHITECTURE",
            claim_id="ui-program-architecture-implementation",
            verifier="ui_program_architecture_conformance",
            status="passed",
            produced_at="2026-06-01T00:00:00Z",
            source_digests={
                "docs-site/src/content/docs/design/active/ui-program-architecture.md": digest_path(
                    REPO_ROOT / "docs-site/src/content/docs/design/active/ui-program-architecture.md"
                )
            },
            findings=[],
            known_gaps=[],
            known_risks=[],
            truth_drift=[],
        ),
        root=tmp_path / "truth-certificates",
    )

    errors = certificate_errors_for_claim(
        "PT-UI-PROGRAM-ARCHITECTURE",
        claim,
        root=tmp_path / "truth-certificates",
    )

    assert any("missing required source digest" in error for error in errors)
    assert any("design-conformance/pt-ui-program-architecture.requirements.yaml" in error for error in errors)


def test_truth_verifier_registry_rejects_unregistered_track_claim() -> None:
    with pytest.raises(WorkflowError, match="not registered"):
        run_verifier(
            track_id="PT-UNKNOWN",
            claim_id="ui-program-architecture-implementation",
            verifier="ui_program_architecture_conformance",
        )


def test_truth_verifier_registry_has_unique_current_bindings() -> None:
    registry = load_truth_verifier_registry()

    bindings = {(entry.verifier_id, entry.track_id, entry.claim_id) for entry in registry.entries}

    assert (
        "ui_program_architecture_conformance",
        "PT-UI-PROGRAM-ARCHITECTURE",
        "ui-program-architecture-implementation",
    ) in bindings

def test_conformance_module_shape_rejects_missing_declared_file(tmp_path: Path) -> None:
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "test",
            "claim_ids": ["test"],
            "final_owner_dirs": [],
            "required_files": [
                {
                    "path": "domain/ui/ui_program/src/graphs/control.rs",
                    "role": "control graph",
                    "required_symbols": ["ControlGraphNode"],
                }
            ],
        }
    )

    findings, checks = verify_rust_module_shape(spec, repo_root=tmp_path)

    assert "conformance required source files exist and expose declared symbols" in checks
    assert [finding.finding_id for finding in findings] == [
        "missing-source-domain-ui-ui-program-src-graphs-control-rs"
    ]


def test_design_coverage_rejects_verified_requirement_without_complete_proof(tmp_path: Path) -> None:
    design_doc = tmp_path / "docs/design.md"
    design_doc.parent.mkdir(parents=True)
    design_doc.write_text("# Design\n", encoding="utf-8")
    subject = tmp_path / "domain/ui/ui_program/src/program.rs"
    subject.parent.mkdir(parents=True)
    subject.write_text("pub struct UiProgram;\n", encoding="utf-8")
    coverage = tmp_path / "coverage.yaml"
    coverage.write_text(
        yaml.safe_dump(
            {
                "version": 1,
                "track_id": "PT-TEST",
                "spec_id": "test",
                "source_design_docs": ["docs/design.md"],
                "requirements": [
                    {
                        "requirement_id": "typed-program",
                        "source_path": "docs/design.md",
                        "source_section": "program",
                        "summary": "UiProgram must be typed.",
                        "owner": "domain/ui/ui_program",
                        "status": "verified",
                        "code_subjects": ["domain/ui/ui_program/src/program.rs"],
                        "test_subjects": [],
                        "evidence_kinds": ["runtime_test"],
                        "semantic_check_ids": ["program_semantics"],
                    }
                ],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "test",
            "claim_ids": ["test"],
            "final_owner_dirs": [],
            "design_coverage_path": "coverage.yaml",
            "required_design_docs": ["docs/design.md"],
            "required_design_requirement_ids": ["typed-program"],
            "required_files": [],
            "evidence_requirements": [{"milestone_id": "PM-TEST", "evidence_kind": "runtime_test"}],
            "semantic_checks": [{"check_id": "program_semantics", "description": "program semantics"}],
        }
    )

    findings, checks = verify_design_coverage(spec, repo_root=tmp_path)

    assert "design requirement coverage matrix is machine-readable and bound to the conformance spec" in checks
    assert any(finding.finding_id == "design-requirement-incomplete-proof-typed-program" for finding in findings)

def test_conformance_module_shape_rejects_root_facade_ownership(tmp_path: Path) -> None:
    facade = tmp_path / "domain/ui/ui_program/src/lib.rs"
    facade.parent.mkdir(parents=True)
    facade.write_text("pub struct ControlGraphNode;\n", encoding="utf-8")
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "test",
            "claim_ids": ["test"],
            "final_owner_dirs": [],
            "required_files": [],
            "root_facades": [
                {
                    "path": "domain/ui/ui_program/src/lib.rs",
                    "forbidden_patterns": ["^pub struct "],
                }
            ],
        }
    )

    findings, _checks = verify_rust_module_shape(spec, repo_root=tmp_path)

    assert [finding.finding_id for finding in findings] == [
        "root-facade-owns-contract-domain-ui-ui-program-src-lib-rs"
    ]

def test_conformance_evidence_rejects_self_referential_records(tmp_path: Path) -> None:
    record = (
        tmp_path
        / "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test/runtime_test-runtime_test.yaml"
    )
    record.parent.mkdir(parents=True)
    record.write_text(
        yaml.safe_dump(
            {
                "evidence_kind": "runtime_test",
                "status": "passed",
                "subject_paths": [
                    "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test/runtime_test-runtime_test.yaml"
                ],
                "validation_commands": ["cargo:test (cargo test -p ui_program architecture_contract) -> exit 0"],
            }
        ),
        encoding="utf-8",
    )
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "test",
            "claim_ids": ["test"],
            "final_owner_dirs": [],
            "required_files": [],
            "evidence_requirements": [
                {
                    "milestone_id": "PM-TEST",
                    "evidence_kind": "runtime_test",
                    "required_validation_fragments": ["cargo test -p ui_program architecture_contract"],
                }
            ],
        }
    )

    findings, checks = verify_evidence_records(spec, repo_root=tmp_path)

    assert "conformance evidence records are resolver-backed and non-self-referential" in checks
    assert any(finding.finding_id.startswith("self-referential-evidence") for finding in findings)


def test_conformance_evidence_rejects_missing_subject_digest(tmp_path: Path) -> None:
    subject = tmp_path / "domain/ui/ui_program/src/program.rs"
    subject.parent.mkdir(parents=True)
    subject.write_text("pub struct UiProgram;\n", encoding="utf-8")
    record = (
        tmp_path
        / "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test/artifact-artifact.yaml"
    )
    record.parent.mkdir(parents=True)
    record.write_text(
        yaml.safe_dump(
            {
                "evidence_kind": "artifact",
                "status": "passed",
                "subject_paths": ["domain/ui/ui_program/src/program.rs"],
                "validation_commands": ["cargo:test (cargo test -p ui_program architecture_contract) -> exit 0"],
            }
        ),
        encoding="utf-8",
    )
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "test",
            "claim_ids": ["test"],
            "final_owner_dirs": [],
            "required_files": [],
            "evidence_requirements": [
                {
                    "milestone_id": "PM-TEST",
                    "evidence_kind": "artifact",
                }
            ],
        }
    )

    findings, _checks = verify_evidence_records(spec, repo_root=tmp_path)

    assert any(finding.finding_id.startswith("missing-subject-digest") for finding in findings)


def test_conformance_evidence_rejects_stale_subject_digest(tmp_path: Path) -> None:
    subject = tmp_path / "domain/ui/ui_program/src/program.rs"
    subject.parent.mkdir(parents=True)
    subject.write_text("pub struct UiProgram;\n", encoding="utf-8")
    record = (
        tmp_path
        / "docs-site/src/content/docs/reports/execution-evidence/pt-test/pm-test/artifact-artifact.yaml"
    )
    record.parent.mkdir(parents=True)
    record.write_text(
        yaml.safe_dump(
            {
                "evidence_kind": "artifact",
                "status": "passed",
                "subject_paths": ["domain/ui/ui_program/src/program.rs"],
                "subject_digests": {"domain/ui/ui_program/src/program.rs": "stale"},
                "validation_commands": ["cargo:test (cargo test -p ui_program architecture_contract) -> exit 0"],
            }
        ),
        encoding="utf-8",
    )
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "test",
            "claim_ids": ["test"],
            "final_owner_dirs": [],
            "required_files": [],
            "evidence_requirements": [
                {
                    "milestone_id": "PM-TEST",
                    "evidence_kind": "artifact",
                }
            ],
        }
    )

    findings, _checks = verify_evidence_records(spec, repo_root=tmp_path)

    assert any(finding.finding_id.startswith("stale-evidence-subject") for finding in findings)

def test_track_execution_harness_verifier_requires_positive_contracts() -> None:
    certificate = run_verifier(
        track_id="PT-TRACK-EXECUTION-HARNESS",
        claim_id="track-execution-harness-authority",
        verifier="track_execution_harness_authority",
    )

    assert certificate.status == "passed"
    assert "transactional execution kernel exposes required authority boundaries" in certificate.checks
    assert "harness has focused tests for safety-critical behavior" in certificate.checks

def test_harness_truth_certificate_sources_exclude_global_production_metadata() -> None:
    sources = verifier_source_paths(
        TRACK_EXECUTION_HARNESS_VERIFIER,
        track_id="PT-TRACK-EXECUTION-HARNESS",
        claim_id="track-execution-harness-authority",
    )

    assert "docs-site/src/content/docs/workspace/production-tracks.yaml" not in sources
    assert "tools/workflow/execution" in sources
    assert "tools/workflow/tests" in sources
    assert "docs-site/src/content/docs/workspace/truth-verifier-registry.yaml" in sources

def test_pm011_declares_exact_crate_creation_authority() -> None:
    manifest = load_yaml(
        REPO_ROOT / "docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program-architecture.yaml"
    )
    pm011 = next(entry for entry in manifest["milestones"] if entry["milestone_id"] == "PM-UI-PROGRAM-ARCH-011")

    assert pm011["may_create_crates"] is True
    assert "crate_creation" in pm011["permission_classes_required"]
    assert pm011["implementation_writer"]["strategy"] == "agent_writer"
    assert "new: domain/ui/ui_controls/Cargo.toml" in pm011["write_scope"]
    assert "new: domain/ui/ui_accessibility/Cargo.toml" in pm011["write_scope"]
    assert "new: domain/ui/ui_geometry/Cargo.toml" in pm011["write_scope"]

def test_generated_roadmap_diagrams_separate_dependency_truth_from_candidates() -> None:
    state = valid_state()
    active_item = item(
        "WR-002",
        blocker=2,
        dependencies=["WR-001"],
        write_scopes=["docs-site"],
    )
    active_item["gate"] = "Ready next"
    state["items"] = [active_item]
    state["edges"] = [{"source": "WR-001", "target": "WR-002", "label": "depends"}]
    archived = {
        "version": state["version"],
        "roadmap": state["roadmap"],
        "items": [item("WR-001", planning_state="completed", dependencies=[], write_scopes=["tools/workflow"])],
    }
    roadmap = RoadmapState.model_validate(
        combine_roadmap_data(
            state,
            archive_data=archived,
            deferred_data={"version": state["version"], "roadmap": state["roadmap"], "items": []},
        )
    )

    dependency = render_dependency_roadmap(roadmap)
    candidates = render_current_candidates_roadmap(roadmap)

    assert "Level 0 - Support Substrate" in dependency
    assert "Parallel" + " Now" not in dependency
    assert "state=completed" not in dependency
    assert "WR-001" not in dependency
    assert "Current Implementation Candidates" in candidates
    assert "state=current_candidate" in candidates
    assert "state=completed" not in candidates
    assert "Immediate Dependency Context" not in candidates
    assert "WR-001" not in candidates

def test_current_repository_roadmap_truth_and_generated_diagrams_conform() -> None:
    roadmap = load_roadmap()
    selected = select_batch_candidates(roadmap)
    dependency_puml = (REPO_ROOT / "docs-site/src/content/docs/workspace/diagrams/value-weighted-dependency-roadmap.puml").read_text(encoding="utf-8")
    candidates_puml = (REPO_ROOT / "docs-site/src/content/docs/workspace/diagrams/current-roadmap-candidates.puml").read_text(encoding="utf-8")

    current_ids = [item.id for item in selected]
    expected_current_ids = [
        item.id
        for item in sorted(
            (item for item in roadmap.active_items if item.can_enter_implementation_batch),
            key=lambda item: (item.level_number, item.lane, -item.score, item.id),
        )
    ]

    assert current_ids == expected_current_ids
    for item_id in current_ids:
        assert item_id in candidates_puml
    wr170 = roadmap.by_id.get("WR-170")
    if wr170 is not None and wr170.planning_state == "completed":
        assert wr170.completion_quality == "perfectionist_verified"
    assert roadmap.by_id["WR-169"].planning_state == "completed"
    assert roadmap.by_id["WR-169"].blocker_label == "B2"
    assert roadmap.by_id["WR-169"].completion_quality == "perfectionist_verified"
    assert roadmap.by_id["WR-146"].planning_state == "completed"
    assert roadmap.by_id["WR-089"].planning_state == "completed"
    assert roadmap.by_id["WR-029"].planning_state == "ready_next"
    assert "state=completed" not in dependency_puml
    assert "state=completed" not in candidates_puml

def test_completed_architecture_quality_requires_satisfied_architecture_claim(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["state"] = "completed"
    production_data["tracks"][0]["target_completion_quality"] = "architecture_runtime_proven"
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", valid_track_manifest_state())
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("architecture_runtime_proven requires a satisfied architecture_contract truth claim" in error for error in errors)

def test_proof_slice_runtime_quality_rejects_overclaiming_wording(tmp_path: Path) -> None:
    roadmap_path = tmp_path / "roadmap.yaml"
    manifest_root = tmp_path / "manifests"
    manifest_root.mkdir()
    production_data = valid_production_state()
    production_data["tracks"][0]["target_completion_quality"] = "proof_slice_runtime_proven"
    production_data["tracks"][0]["strategic_goal"] = "Prove the final architecture platform."
    manifest_data = valid_track_manifest_state()
    manifest_data["truth_claims"] = [
        {
            "claim_id": "proof-slice-test",
            "claim_kind": "proof_slice",
            "claim_level": "proof_slice_runtime_proven",
            "claim_status": "satisfied",
            "claim_statement": "Bounded proof slices passed.",
            "required_docs": [
                {
                    "evidence_kind": "doc_exists",
                    "path": "tools/workflow/test_workflow.py",
                    "reason": "Fixture truth evidence path exists.",
                }
            ],
            "required_code_contracts": [],
            "required_validations": [],
            "required_closeout_evidence": [],
            "known_gaps": [],
            "supersedes": [],
            "blocks_downstream": [],
        }
    ]
    write_yaml(roadmap_path, valid_state())
    write_yaml(manifest_root / "pt-test.yaml", manifest_data)
    planning = ProductionPlanningState.model_validate(production_data)

    errors = validate_manifest_backed_tracks(planning, roadmap_path=roadmap_path, manifest_root=manifest_root)

    assert any("production wording claims stronger truth than proof_slice_runtime_proven" in error for error in errors)


def test_old_single_file_workflow_command_is_not_full_suite_authority() -> None:
    assert validation_command_from_string("uv run pytest tools/workflow/test_workflow.py -q")["command_id"] == "blocked"
    assert validation_command_from_string("task workflow:test")["command_id"] == "task:workflow:test"


def test_evidence_without_structured_validation_provenance_fails(tmp_path: Path) -> None:
    record_path = tmp_path / "runtime_test-runtime_test.yaml"
    record_path.write_text("", encoding="utf-8")
    data = {
        "evidence_kind": "runtime_test",
        "status": "passed",
        "subject_paths": ["subject.rs"],
        "subject_digests": {},
        "validation_commands": ["task workflow:test -> exit 0"],
    }
    requirement = SimpleNamespace(
        evidence_kind="runtime_test",
        required_subject_paths=True,
        required_validation_fragments=["task workflow:test"],
    )

    findings = validate_evidence_record(record_path, data, requirement, repo_root=tmp_path)

    assert any("structured validation provenance" in finding.message for finding in findings)


def test_evidence_missing_run_ledger_fails(tmp_path: Path) -> None:
    subject = tmp_path / "subject.rs"
    subject.write_text("pub struct Subject;", encoding="utf-8")
    subject_digest = digest_path(subject)
    argv = ["task", "workflow:test"]
    result_digest = validation_result_digest(
        command_id="task:workflow:test",
        argv=argv,
        returncode=0,
        files_changed=[],
        subject_digests={"subject.rs": subject_digest},
    )
    record_path = tmp_path / "runtime_test-runtime_test.yaml"
    record_path.write_text("", encoding="utf-8")
    data = {
        "evidence_kind": "runtime_test",
        "status": "passed",
        "subject_paths": ["subject.rs"],
        "subject_digests": {"subject.rs": subject_digest},
        "validation_provenance": [
            {
                "command_id": "task:workflow:test",
                "argv": argv,
                "returncode": 0,
                "run_ledger_path": "missing-ledger.yaml",
                "run_action_id": "ACTION-1",
                "validation_result_digest": result_digest,
                "subject_digests": {"subject.rs": subject_digest},
            }
        ],
    }
    requirement = SimpleNamespace(
        evidence_kind="runtime_test",
        required_subject_paths=True,
        required_validation_fragments=["task workflow:test"],
    )

    findings = validate_evidence_record(record_path, data, requirement, repo_root=tmp_path)

    assert any("missing validation run ledger" in finding.message for finding in findings)


def test_unknown_semantic_check_requires_registered_verifier() -> None:
    spec = ConformanceSpec.model_validate(
        {
            "track_id": "PT-TEST",
            "spec_id": "spec",
            "claim_ids": ["claim"],
            "final_owner_dirs": [],
            "required_files": [],
            "semantic_checks": [
                {
                    "check_id": "unknown_semantic_check",
                    "description": "Unknown semantic check must fail closed.",
                    "subject_paths": [],
                    "behavior_probe_paths": [],
                    "behavior_probe_ids": [],
                    "evidence_kinds": [],
                    "required_symbols": [],
                    "required_validation_fragments": [],
                }
            ],
        }
    )

    findings, _checks = verify_semantic_checks(spec, repo_root=REPO_ROOT)

    assert any("has no registered verifier" in finding.message for finding in findings)
