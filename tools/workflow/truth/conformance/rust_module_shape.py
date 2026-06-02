from __future__ import annotations

import re
from pathlib import Path

from roadmap_state import REPO_ROOT, repo_path

from truth.certificates import TruthFinding
from truth.conformance.spec import ConformanceSpec


def verify_rust_module_shape(
    spec: ConformanceSpec,
    *,
    repo_root: Path = REPO_ROOT,
) -> tuple[list[TruthFinding], list[str]]:
    findings: list[TruthFinding] = []
    checks: list[str] = []

    checks.append("conformance required source files exist and expose declared symbols")
    for file_spec in spec.required_files:
        path = repo_root / file_spec.path
        if not path.exists():
            findings.append(
                TruthFinding(
                    finding_id=f"missing-source-{finding_slug(file_spec.path)}",
                    message=f"Required {file_spec.role} source file is missing: {file_spec.path}.",
                    subject_paths=[file_spec.path],
                    remediation="Create or reconcile the owner file before certifying this truth claim.",
                )
            )
            continue
        text = read_text(path)
        for symbol in file_spec.required_symbols:
            if symbol not in text:
                findings.append(
                    TruthFinding(
                        finding_id=f"missing-symbol-{finding_slug(file_spec.path)}-{finding_slug(symbol)}",
                        message=f"{file_spec.path} does not expose required {file_spec.role} symbol `{symbol}`.",
                        subject_paths=[file_spec.path],
                        remediation="Move or implement the symbol in the owning module declared by the conformance spec.",
                    )
                )
        for pattern in file_spec.forbidden_patterns:
            if re.search(pattern, text, flags=re.MULTILINE):
                findings.append(
                    TruthFinding(
                        finding_id=f"forbidden-pattern-{finding_slug(file_spec.path)}-{finding_slug(pattern)}",
                        message=f"{file_spec.path} matches forbidden responsibility pattern `{pattern}`.",
                        subject_paths=[file_spec.path],
                        remediation="Move the responsibility to the owning module before certification.",
                    )
                )

    checks.append("root facade modules remain facades instead of owning broad contracts")
    for facade in spec.root_facades:
        path = repo_root / facade.path
        text = read_text(path)
        for pattern in facade.forbidden_patterns:
            if re.search(pattern, text, flags=re.MULTILINE):
                findings.append(
                    TruthFinding(
                        finding_id=f"root-facade-owns-contract-{finding_slug(facade.path)}",
                        message=f"{facade.path} still owns implementation contract text matching `{pattern}` instead of delegating to owner modules.",
                        subject_paths=[facade.path],
                        remediation="Keep crate root files as module declarations and re-exports; move contracts into responsibility modules.",
                    )
                )

    return findings, checks


def finding_slug(value: str) -> str:
    return re.sub(r"[^a-zA-Z0-9]+", "-", value).strip("-").lower()


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""

