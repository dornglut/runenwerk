from __future__ import annotations

import re
from pathlib import Path

from roadmap_state import REPO_ROOT, WorkflowError, repo_path

from truth.certificates import TruthCertificate, TruthFinding, now_utc_iso, source_digests
from truth.conformance import ui_program_architecture
from truth.registry import verifier_binding, verifier_source_paths as registry_verifier_source_paths


UI_PROGRAM_ARCHITECTURE_VERIFIER = "ui_program_architecture_conformance"
TRACK_EXECUTION_HARNESS_VERIFIER = "track_execution_harness_authority"


def verifier_source_paths(
    verifier: str,
    *,
    track_id: str | None = None,
    claim_id: str | None = None,
    repo_root: Path = REPO_ROOT,
) -> list[str]:
    return registry_verifier_source_paths(
        verifier,
        track_id=track_id,
        claim_id=claim_id,
        repo_root=repo_root,
    )


def run_verifier(
    *,
    track_id: str,
    claim_id: str,
    verifier: str,
    repo_root: Path = REPO_ROOT,
) -> TruthCertificate:
    binding = verifier_binding(
        verifier_id=verifier,
        track_id=track_id,
        claim_id=claim_id,
        repo_root=repo_root,
    )
    if binding.backend == "ui_program_architecture":
        findings, checks = verify_ui_program_architecture(
            track_id=track_id,
            claim_id=claim_id,
            spec_path=binding.conformance_spec_path,
            repo_root=repo_root,
        )
    elif binding.backend == "track_execution_harness":
        findings, checks = verify_track_execution_harness(repo_root=repo_root)
    else:
        raise WorkflowError(f"{track_id}: truth verifier {verifier!r} declares unsupported backend {binding.backend!r}")

    status = "passed" if not findings else "failed"
    digest_paths = registry_verifier_source_paths(
        verifier,
        track_id=track_id,
        claim_id=claim_id,
        repo_root=repo_root,
    )
    return TruthCertificate(
        track_id=track_id,
        claim_id=claim_id,
        verifier=verifier,
        status=status,
        produced_at=now_utc_iso(),
        source_digests=source_digests(digest_paths, repo_root=repo_root),
        findings=findings,
        checks=checks,
    )


def verify_ui_program_architecture(
    *,
    track_id: str,
    claim_id: str,
    spec_path: str | None,
    repo_root: Path,
) -> tuple[list[TruthFinding], list[str]]:
    if not spec_path:
        raise WorkflowError(f"{track_id}: UI architecture verifier for {claim_id} is missing conformance_spec_path")
    return ui_program_architecture.verify(
        track_id=track_id,
        claim_id=claim_id,
        spec_path=spec_path,
        repo_root=repo_root,
    )


def verify_track_execution_harness(*, repo_root: Path) -> tuple[list[TruthFinding], list[str]]:
    findings: list[TruthFinding] = []
    checks: list[str] = []

    legacy = repo_root / "tools/workflow/track_execution_manifest.py"
    taskfile = repo_root / "Taskfile.yml"
    adapter = repo_root / "tools/workflow/production_track_cli.py"

    checks.append("legacy manifest runner is non-authoritative")
    legacy_text = read_text(legacy)
    if "legacy compatibility only" not in legacy_text or "raise SystemExit" not in legacy_text:
        findings.append(
            TruthFinding(
                finding_id="legacy-runner-not-disabled",
                message="track_execution_manifest.py is not clearly disabled as execution authority.",
                subject_paths=[repo_path(legacy)],
                remediation="Keep track_execution_manifest.py as a fail-closed compatibility shim only.",
            )
        )

    checks.append("public production commands route through clean adapter")
    taskfile_text = read_text(taskfile)
    for task_name in ("production:run-track", "production:audit-track", "production:next", "production:lock-track"):
        if f"{task_name}:" not in taskfile_text:
            findings.append(
                TruthFinding(
                    finding_id=f"{task_name}-missing",
                    message=f"{task_name} is missing from Taskfile.yml.",
                    subject_paths=[repo_path(taskfile)],
                    remediation="Route public production track commands through production_track_cli.py.",
                )
            )
    if "tools/workflow/production_track_cli.py" not in taskfile_text:
        findings.append(
            TruthFinding(
                finding_id="public-commands-not-adapter-backed",
                message="Taskfile.yml does not route production track commands through production_track_cli.py.",
                subject_paths=[repo_path(taskfile)],
                remediation="Use production_track_cli.py as the public adapter for production track commands.",
            )
        )

    checks.append("authority-critical modules do not import legacy execution path")
    authority_files = [
        repo_root / "tools/workflow/production_goal.py",
        repo_root / "tools/workflow/production_state.py",
        adapter,
        *sorted((repo_root / "tools/workflow/execution").glob("*.py")),
        *sorted((repo_root / "tools/workflow/track_sources").glob("*.py")),
    ]
    for path in authority_files:
        text = read_text(path)
        if "import track_execution_manifest" in text or "from track_execution_manifest" in text:
            findings.append(
                TruthFinding(
                    finding_id=f"legacy-import-{path.stem}",
                    message=f"{repo_path(path)} imports the legacy manifest runner.",
                    subject_paths=[repo_path(path)],
                    remediation="Import source models or execution kernel modules instead of track_execution_manifest.py.",
                )
            )

    checks.append("execution kernel and source model modules exist")
    for raw_path in ("tools/workflow/execution", "tools/workflow/track_sources", "tools/workflow/production_track_cli.py"):
        path = repo_root / raw_path
        if not path.exists():
            findings.append(
                TruthFinding(
                    finding_id=f"missing-{raw_path.replace('/', '-')}",
                    message=f"Required harness owner path {raw_path} is missing.",
                    subject_paths=[raw_path],
                    remediation="Restore the clean execution/source/public-adapter owner path.",
                )
            )

    checks.append("transactional execution kernel exposes required authority boundaries")
    required_harness_symbols = {
        "tools/workflow/execution/contracts.py": [
            "ActionContract",
            "AgentSubActionContract",
            "ContractPack",
            "ValidationCommand",
            "EvidenceRequirement",
            "ExecutorKind",
        ],
        "tools/workflow/track_sources/plan_metadata.py": [
            "AgentSubActionMetadata",
            "agent_subactions",
        ],
        "tools/workflow/execution/compiler.py": [
            "agent_subactions",
            "parent_action_id",
            "subaction_evidence_required",
        ],
        "tools/workflow/execution/locks.py": [
            "ExecutionLock",
            "execution_lock_errors",
            "contract_pack_freshness_errors",
        ],
        "tools/workflow/execution/workspace.py": [
            "create_full_snapshot",
            "import_scoped_changes",
            "validate_changed_files",
            "target digest drifted before import",
        ],
        "tools/workflow/execution/evidence.py": [
            "EvidenceRecord",
            "write_resolver_evidence_records",
            "runtime_test evidence requires validation_command_ids",
            "subject path must be an exact file",
        ],
        "tools/workflow/execution/runner.py": [
            "run_action",
            "run_validation_commands",
            "shell=False",
            "create_full_snapshot",
            "import_scoped_changes",
        ],
        "tools/workflow/execution/writers.py": [
            "CodexExecBackend",
            "subprocess.Popen",
            "transcript_dir",
            "transcript_paths",
            "AgentWriterError",
            "agent_writer requires a current execution lock",
            "action_prompt",
        ],
        "tools/workflow/execution/ledger.py": [
            "transcript_paths",
        ],
        "tools/workflow/production_track_cli.py": [
            "execution_lock_errors",
            "contract_pack_freshness_errors",
            "full-automation",
        ],
    }
    for raw_path, symbols in required_harness_symbols.items():
        text = read_text(repo_root / raw_path)
        for symbol in symbols:
            if symbol not in text:
                findings.append(
                    TruthFinding(
                        finding_id=f"missing-harness-{camel_to_kebab(symbol)}",
                        message=f"Track execution harness is missing required authority contract `{symbol}` in {raw_path}.",
                        subject_paths=[raw_path],
                        remediation="Implement and test the contract before certifying the harness as architecture/runtime proven.",
                    )
                )

    checks.append("harness has focused tests for safety-critical behavior")
    test_sources = workflow_test_sources(repo_root)
    test_text = "\n".join(read_text(path) for path in test_sources)
    required_test_names = [
        "test_authority_paths_do_not_import_legacy_manifest_runner",
        "test_full_automation_audit_requires_current_execution_lock_when_requested",
        "test_execution_agent_writer_requires_lock_and_imports_scoped_diff",
        "test_execution_agent_writer_result_records_transcript_paths",
        "test_execution_agent_writer_failure_carries_transcript_paths",
        "test_codex_exec_backend_writes_streamed_transcripts",
        "test_execution_contract_compiler_expands_agent_subactions_as_resumable_actions",
        "test_execution_agent_writer_rejects_out_of_scope_diff",
        "test_execution_transactional_writer_leaves_main_workspace_unchanged_on_validation_failure",
        "test_closeout_claim_rejects_non_runtime_evidence_without_subject_paths",
        "test_execution_proof_aggregation_requires_prior_machine_evidence",
        "test_satisfied_strong_truth_claim_requires_current_certificate",
    ]
    for test_name in required_test_names:
        if test_name not in test_text:
            findings.append(
                TruthFinding(
                    finding_id=f"missing-test-{test_name.removeprefix('test_').replace('_', '-')}",
                    message=f"Harness certification is missing focused regression test `{test_name}`.",
                    subject_paths=[repo_path(path) for path in test_sources],
                    remediation="Add a focused test that proves this safety property before certifying the harness.",
                )
            )

    return findings, checks


def workflow_test_sources(repo_root: Path) -> list[Path]:
    split_tests = sorted((repo_root / "tools/workflow/tests").glob("test_*.py"))
    legacy_test = repo_root / "tools/workflow/test_workflow.py"
    if legacy_test.exists():
        return [legacy_test, *split_tests]
    return split_tests


def camel_to_kebab(value: str) -> str:
    value = re.sub(r"(.)([A-Z][a-z]+)", r"\1-\2", value)
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", value)
    return value.replace("_", "-").lower()


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""


def contains(path: Path, needle: str) -> bool:
    return needle in read_text(path)
