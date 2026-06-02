#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

import typer
from rich.console import Console

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from production_state import PRODUCTION_SOURCE, load_production_tracks
from roadmap_state import WorkflowError, repo_path
from track_sources.manifest import TRACK_EXECUTION_MANIFEST_ROOT, load_track_execution_manifest
from truth.certificates import (
    certificate_errors_for_claim,
    certificate_path,
    certificate_summary_lines,
    strong_claim_requires_certificate,
    write_certificate,
)
from truth.verifiers import run_verifier


console = Console()
app = typer.Typer(no_args_is_help=True, help="Independent truth certification commands.")


def load_track_and_manifest(track_id: str, *, production_source: Path, manifest_root: Path):
    planning = load_production_tracks(production_source)
    track = next((candidate for candidate in planning.tracks if candidate.id == track_id), None)
    if track is None:
        raise WorkflowError(f"{track_id}: not present in production tracks source")
    loaded = load_track_execution_manifest(track_id, root=manifest_root)
    if loaded is None:
        raise WorkflowError(f"{track_id}: missing Track Execution Manifest")
    return track, loaded.manifest


def claim_by_id(manifest, claim_id: str):
    claim = next((candidate for candidate in manifest.truth_claims if candidate.claim_id == claim_id), None)
    if claim is None:
        raise WorkflowError(f"{manifest.track_id}: missing truth claim {claim_id}")
    return claim


@app.command("audit")
def audit_command(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
) -> None:
    try:
        _, manifest = load_track_and_manifest(track, production_source=production_source, manifest_root=manifest_source_root)
        console.print("Truth claims:")
        for claim in manifest.truth_claims:
            verifier = f", verifier={claim.truth_verifier}" if claim.truth_verifier else ""
            console.print(f"- {claim.claim_id}: {claim.claim_status} {claim.claim_kind} at {claim.claim_level}{verifier}")
            if claim.known_gaps:
                console.print(f"  known gaps: {len(claim.known_gaps)}")
        lines = certificate_summary_lines(track, manifest.truth_claims)
        if lines:
            console.print("Truth certificate status:")
            for line in lines:
                console.print(line)
        else:
            console.print(
                "Truth certificate status: no satisfied strong truth claims currently require certificates; "
                "blocked claims remain uncertified."
            )
        render_blocked_verifier_status(track, manifest.truth_claims)
        errors: list[str] = []
        for claim in manifest.truth_claims:
            errors.extend(certificate_errors_for_claim(track, claim))
            if strong_claim_requires_certificate(claim):
                verifier = claim.truth_verifier
                if not verifier:
                    continue
                certificate = run_verifier(track_id=track, claim_id=claim.claim_id, verifier=verifier)
                if certificate.status != "passed":
                    errors.append(
                        f"{track}: satisfied strong truth claim {claim.claim_id} fails current verifier rerun "
                        f"with {len(certificate.findings)} findings"
                    )
        if errors:
            console.print("[red]truth:audit failed[/red]")
            for error in errors:
                console.print(f"- {error}")
            raise typer.Exit(1)
        console.print("[green]truth:audit passed[/green]")
    except WorkflowError as error:
        console.print("[red]truth:audit failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


def render_blocked_verifier_status(track: str, claims: list[object]) -> None:
    blocked_claims = [
        claim
        for claim in claims
        if getattr(claim, "claim_status", None) == "blocked" and getattr(claim, "truth_verifier", None)
    ]
    if not blocked_claims:
        return
    console.print("Blocked verifier status:")
    for claim in blocked_claims:
        claim_id = getattr(claim, "claim_id")
        verifier = getattr(claim, "truth_verifier")
        try:
            certificate = run_verifier(track_id=track, claim_id=claim_id, verifier=verifier)
        except WorkflowError as error:
            console.print(f"- {claim_id}: verifier error: {error}")
            continue
        console.print(
            f"- {claim_id}: {certificate.status} "
            f"({len(certificate.findings)} findings, {len(certificate.known_gaps)} gaps, "
            f"{len(certificate.known_risks)} risks, {len(certificate.truth_drift)} drift)"
        )


@app.command("verify")
def verify_command(
    track: str = typer.Option(..., "--track"),
    claim: str = typer.Option(..., "--claim"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
) -> None:
    try:
        _, manifest = load_track_and_manifest(track, production_source=production_source, manifest_root=manifest_source_root)
        truth_claim = claim_by_id(manifest, claim)
        verifier = truth_claim.truth_verifier
        if not verifier:
            raise WorkflowError(f"{track}: truth claim {claim} does not declare truth_verifier")
        certificate = run_verifier(track_id=track, claim_id=claim, verifier=verifier)
        render_certificate_result(certificate)
        if certificate.status != "passed":
            raise typer.Exit(1)
    except WorkflowError as error:
        console.print("[red]truth:verify failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("certify")
def certify_command(
    track: str = typer.Option(..., "--track"),
    claim: str = typer.Option(..., "--claim"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
) -> None:
    try:
        _, manifest = load_track_and_manifest(track, production_source=production_source, manifest_root=manifest_source_root)
        truth_claim = claim_by_id(manifest, claim)
        verifier = truth_claim.truth_verifier
        if not verifier:
            raise WorkflowError(f"{track}: truth claim {claim} does not declare truth_verifier")
        certificate = run_verifier(track_id=track, claim_id=claim, verifier=verifier)
        path = write_certificate(certificate)
        render_certificate_result(certificate)
        console.print(f"Certificate: {repo_path(path)}")
        if certificate.status != "passed":
            raise typer.Exit(1)
    except WorkflowError as error:
        console.print("[red]truth:certify failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("post-completion-audit")
def post_completion_audit_command(
    track: str = typer.Option(..., "--track"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
) -> None:
    try:
        track_model, manifest = load_track_and_manifest(track, production_source=production_source, manifest_root=manifest_source_root)
        errors: list[str] = []
        if track_model.state == "completed":
            for claim in manifest.truth_claims:
                errors.extend(certificate_errors_for_claim(track, claim))
                if strong_claim_requires_certificate(claim) and claim.truth_verifier:
                    certificate = run_verifier(track_id=track, claim_id=claim.claim_id, verifier=claim.truth_verifier)
                    if certificate.status != "passed":
                        errors.append(
                            f"{track}: completed track strong truth claim {claim.claim_id} fails current verifier rerun "
                            f"with {len(certificate.findings)} findings"
                        )
        else:
            console.print(f"{track}: track is {track_model.state}; post-completion audit checks active truth blockers only.")
        if errors:
            console.print("[red]truth:post-completion-audit failed[/red]")
            for error in errors:
                console.print(f"- {error}")
            raise typer.Exit(1)
        console.print("[green]truth:post-completion-audit passed[/green]")
    except WorkflowError as error:
        console.print("[red]truth:post-completion-audit failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


def render_certificate_result(certificate) -> None:
    style = "green" if certificate.status == "passed" else "red"
    console.print(f"[{style}]Verifier {certificate.verifier}: {certificate.status}[/{style}]")
    console.print(f"Checks: {len(certificate.checks)}")
    for check in certificate.checks:
        console.print(f"- {check}")
    if certificate.findings:
        console.print("Findings:")
        for finding in certificate.findings:
            console.print(f"- {finding.finding_id}: {finding.message}")
            console.print(f"  Remediation: {finding.remediation}")
            if finding.subject_paths:
                console.print(f"  Subjects: {', '.join(finding.subject_paths)}")
    else:
        console.print("Findings: none")
    console.print(f"Expected certificate path: {repo_path(certificate_path(certificate.track_id, certificate.claim_id))}")


if __name__ == "__main__":
    app()
