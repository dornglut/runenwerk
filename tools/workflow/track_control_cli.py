#!/usr/bin/env python3
from __future__ import annotations

import contextlib
import io
import json
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import typer
import yaml
from rich.console import Console

sys.path.insert(0, str(Path(__file__).resolve().parent))

from execution.cli import run_command as execution_run_command
from execution.compiler import CONTRACT_PACK_ROOT, compile_contract_pack, contract_pack_path, load_contract_pack, write_contract_pack
from execution.evidence import EVIDENCE_ROOT
from execution.ledger import RUN_LEDGER_ROOT
from execution.locks import (
    EXECUTION_LOCK_ROOT,
    build_execution_lock,
    contract_pack_freshness_errors,
    execution_lock_errors,
    load_execution_lock,
    write_execution_lock,
)
from execution.preflight import preflight_for_mode
from production_state import PRODUCTION_SOURCE, load_production_tracks, production_validation_errors
from roadmap_state import ROADMAP_SOURCE, REPO_ROOT, WorkflowError, repo_path
from track_sources.manifest import TRACK_EXECUTION_MANIFEST_ROOT, load_track_execution_manifest, truth_claim_summary_lines
from truth.certificates import certificate_errors_for_claim, certificate_summary_lines
from prompt_doctrine import QUALITY_DOCTRINE_ID, quality_doctrine_lines


LOCAL_STATE_PATH = REPO_ROOT / ".runenwerk/workflow-state.yaml"
TRACK_INTENT_LOCK_ROOT = REPO_ROOT / "docs-site/src/content/docs/workspace/track-intent-locks"
STRATEGIC_PERMISSIONS = {"crate_creation", "foundation_extraction"}
DEFAULT_DENIED_PERMISSIONS = {"foundation_extraction"}
RUN_LEDGER_RETENTION_POLICY = REPO_ROOT / "docs-site/src/content/docs/workspace/run-ledger-retention-policy.yaml"


console = Console()
app = typer.Typer(no_args_is_help=True, help="Ergonomic production-track control plane.")


def now_utc_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


class ActiveTrackState:
    def __init__(self, *, track_id: str, set_by: str, set_at: str, note: str = "") -> None:
        self.track_id = clean_text(track_id, "track_id")
        self.set_by = clean_text(set_by, "set_by")
        self.set_at = clean_text(set_at, "set_at")
        self.note = note.strip()

    @classmethod
    def from_mapping(cls, data: object) -> ActiveTrackState:
        if not isinstance(data, dict):
            raise WorkflowError(f"{repo_path(LOCAL_STATE_PATH)} must contain a YAML mapping")
        return cls(
            track_id=str(data.get("track_id", "")),
            set_by=str(data.get("set_by", "")),
            set_at=str(data.get("set_at", "")),
            note=str(data.get("note", "")),
        )

    def to_mapping(self) -> dict[str, str]:
        data = {"track_id": self.track_id, "set_by": self.set_by, "set_at": self.set_at}
        if self.note:
            data["note"] = self.note
        return data


class PermissionAuthorization:
    def __init__(self, *, permission: str, by: str, at: str, reason: str) -> None:
        self.permission = clean_text(permission, "permission")
        self.by = clean_text(by, "by")
        self.at = clean_text(at, "at")
        self.reason = clean_text(reason, "reason")

    @classmethod
    def from_mapping(cls, permission: str, data: object) -> PermissionAuthorization:
        if not isinstance(data, dict):
            raise WorkflowError(f"intent lock permission {permission} must be a YAML mapping")
        return cls(
            permission=permission,
            by=str(data.get("by", "")),
            at=str(data.get("at", "")),
            reason=str(data.get("reason", "")),
        )

    def to_mapping(self) -> dict[str, str]:
        return {"by": self.by, "at": self.at, "reason": self.reason}


class TrackIntentLock:
    def __init__(
        self,
        *,
        track_id: str,
        updated_at: str,
        granted_permissions: dict[str, PermissionAuthorization] | None = None,
        denied_permissions: dict[str, PermissionAuthorization] | None = None,
    ) -> None:
        self.track_id = clean_text(track_id, "track_id")
        self.updated_at = clean_text(updated_at, "updated_at")
        self.granted_permissions = granted_permissions or {}
        self.denied_permissions = denied_permissions or {}
        overlap = sorted(set(self.granted_permissions) & set(self.denied_permissions))
        if overlap:
            raise WorkflowError(f"{self.track_id}: intent lock grants and denies the same permission: {', '.join(overlap)}")

    @classmethod
    def from_mapping(cls, data: object) -> TrackIntentLock:
        if not isinstance(data, dict):
            raise WorkflowError("intent lock must contain a YAML mapping")
        grants = {
            permission: PermissionAuthorization.from_mapping(permission, value)
            for permission, value in (data.get("granted_permissions") or {}).items()
        }
        denials = {
            permission: PermissionAuthorization.from_mapping(permission, value)
            for permission, value in (data.get("denied_permissions") or {}).items()
        }
        return cls(
            track_id=str(data.get("track_id", "")),
            updated_at=str(data.get("updated_at", "")),
            granted_permissions=grants,
            denied_permissions=denials,
        )

    def to_mapping(self) -> dict[str, object]:
        return {
            "version": 1,
            "track_id": self.track_id,
            "updated_at": self.updated_at,
            "granted_permissions": {
                permission: grant.to_mapping() for permission, grant in sorted(self.granted_permissions.items())
            },
            "denied_permissions": {
                permission: denial.to_mapping() for permission, denial in sorted(self.denied_permissions.items())
            },
        }


def clean_text(value: str, field_name: str) -> str:
    cleaned = value.strip()
    if not cleaned:
        raise WorkflowError(f"{field_name} must not be empty")
    return cleaned


def yaml_load_mapping(path: Path) -> dict[str, Any]:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise WorkflowError(f"{repo_path(path)} must contain a YAML mapping")
    return data


def run_ledger_retention_policy(path: Path = RUN_LEDGER_RETENTION_POLICY) -> dict[str, Any]:
    if not path.exists():
        raise WorkflowError(f"missing run-ledger retention policy: {repo_path(path)}")
    policy = yaml_load_mapping(path)
    if policy.get("policy_id") != "runenwerk-run-ledger-retention-v1":
        raise WorkflowError(f"{repo_path(path)} has unsupported policy_id {policy.get('policy_id')!r}")
    failed_ledgers = policy.get("failed_ledgers")
    if not isinstance(failed_ledgers, dict) or failed_ledgers.get("current_authority") is not False:
        raise WorkflowError(f"{repo_path(path)} must mark failed ledgers as diagnostic history, not current authority")
    return policy


def active_track_path(*, local_state_path: Path = LOCAL_STATE_PATH) -> Path:
    return local_state_path


def read_active_track(*, local_state_path: Path = LOCAL_STATE_PATH) -> ActiveTrackState | None:
    path = active_track_path(local_state_path=local_state_path)
    if not path.exists():
        return None
    return ActiveTrackState.from_mapping(yaml_load_mapping(path))


def write_active_track(state: ActiveTrackState, *, local_state_path: Path = LOCAL_STATE_PATH) -> Path:
    path = active_track_path(local_state_path=local_state_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(state.to_mapping(), sort_keys=False), encoding="utf-8", newline="\n")
    return path


def resolve_track(track: str | None, *, local_state_path: Path = LOCAL_STATE_PATH) -> str:
    if track:
        return track
    active = read_active_track(local_state_path=local_state_path)
    if active is None:
        raise WorkflowError("no active track set; run task track:use -- --track <TRACK_ID> or pass --track")
    return active.track_id


def intent_lock_path(track_id: str, *, root: Path = TRACK_INTENT_LOCK_ROOT) -> Path:
    return root / f"{track_id.lower()}.yaml"


def load_intent_lock(track_id: str, *, root: Path = TRACK_INTENT_LOCK_ROOT) -> TrackIntentLock | None:
    path = intent_lock_path(track_id, root=root)
    if not path.exists():
        return None
    lock = TrackIntentLock.from_mapping(yaml_load_mapping(path))
    if lock.track_id != track_id:
        raise WorkflowError(f"{repo_path(path)} declares track_id={lock.track_id}, expected {track_id}")
    return lock


def write_intent_lock(lock: TrackIntentLock, *, root: Path = TRACK_INTENT_LOCK_ROOT) -> Path:
    path = intent_lock_path(lock.track_id, root=root)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(lock.to_mapping(), sort_keys=False, width=4096), encoding="utf-8", newline="\n")
    return path


def ensure_default_denials(lock: TrackIntentLock, *, by: str) -> None:
    for permission in sorted(DEFAULT_DENIED_PERMISSIONS):
        if permission not in lock.denied_permissions and permission not in lock.granted_permissions:
            lock.denied_permissions[permission] = PermissionAuthorization(
                permission=permission,
                by=by,
                at=now_utc_iso(),
                reason="Default strategic gate; requires a separate accepted extraction workflow.",
            )


def required_permissions(pack) -> set[str]:
    permissions: set[str] = set()
    for action in pack.actions:
        permissions.update(action.permissions_required)
    return permissions


def status_command_for(track_id: str) -> str:
    return f"task track -- --track {track_id}"


def go_command_for(track_id: str) -> str:
    return f"task track:go -- --track {track_id}"


def allow_command_for(track_id: str, permission: str) -> str:
    return f'task track:allow -- --track {track_id} --permission {permission} --reason "<why this exact scope is approved>"'


def doctrine_payload_fields() -> dict[str, object]:
    return {
        "quality_doctrine_id": QUALITY_DOCTRINE_ID,
        "quality_doctrine": quality_doctrine_lines(),
    }


def pack_status(
    track_id: str,
    *,
    contract_pack_root: Path,
) -> tuple[object | None, list[str]]:
    pack = load_contract_pack(track_id, root=contract_pack_root)
    if pack is None:
        return None, [f"{track_id}: missing Execution Contract Pack at {repo_path(contract_pack_path(track_id, root=contract_pack_root))}"]
    return pack, contract_pack_freshness_errors(pack)


def compile_or_refresh_pack(
    track_id: str,
    *,
    production_source: Path,
    roadmap_source: Path,
    manifest_source_root: Path,
    contract_pack_root: Path,
) -> object:
    pack, freshness = pack_status(track_id, contract_pack_root=contract_pack_root)
    if pack is None or freshness:
        pack = compile_contract_pack(
            track_id,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_root=manifest_source_root,
            contract_pack_root=contract_pack_root,
        )
        write_contract_pack(pack, root=contract_pack_root)
    return pack


def intent_errors(
    track_id: str,
    *,
    permissions: set[str],
    intent_lock_root: Path,
) -> tuple[TrackIntentLock | None, list[str], str | None]:
    strategic_required = permissions & STRATEGIC_PERMISSIONS
    if "foundation_extraction" in strategic_required:
        return None, [f"{track_id}: foundation_extraction is a strategic human gate and cannot be authorized by task track:allow"], None
    lock = load_intent_lock(track_id, root=intent_lock_root)
    if not strategic_required:
        return lock, [], None
    if lock is None:
        permission = sorted(strategic_required)[0]
        return None, [f"{track_id}: strategic permission {permission} requires a Track Intent Lock"], permission
    denied = sorted(permissions & set(lock.denied_permissions))
    if denied:
        return lock, [f"{track_id}: required permission is denied by Track Intent Lock: {', '.join(denied)}"], denied[0]
    missing = sorted(strategic_required - set(lock.granted_permissions))
    if missing:
        return lock, [f"{track_id}: strategic permission {missing[0]} is not granted by Track Intent Lock"], missing[0]
    return lock, [], None


def deny_permissions(intent_lock: TrackIntentLock | None) -> set[str]:
    denied = set(DEFAULT_DENIED_PERMISSIONS)
    if intent_lock is not None:
        denied.update(intent_lock.denied_permissions)
    return denied


def prepare_execution(
    track_id: str,
    *,
    production_source: Path,
    roadmap_source: Path,
    manifest_source_root: Path,
    contract_pack_root: Path,
    lock_root: Path,
    intent_lock_root: Path,
    locked_by: str,
) -> dict[str, Any]:
    pack = compile_or_refresh_pack(
        track_id,
        production_source=production_source,
        roadmap_source=roadmap_source,
        manifest_source_root=manifest_source_root,
        contract_pack_root=contract_pack_root,
    )
    permissions = required_permissions(pack)
    intent_lock, blockers, blocked_permission = intent_errors(
        track_id,
        permissions=permissions,
        intent_lock_root=intent_lock_root,
    )
    if blockers:
        return {
            "verdict": "needs_authorization",
            "track_id": track_id,
            **doctrine_payload_fields(),
            "blockers": blockers,
            "blocked_permission": blocked_permission,
            "next_command": allow_command_for(track_id, blocked_permission or "crate_creation"),
            "required_permissions": sorted(permissions),
        }
    preflight_errors = preflight_for_mode(pack, mode="full-track", allow=permissions)
    if preflight_errors:
        return {
            "verdict": "blocked",
            "track_id": track_id,
            **doctrine_payload_fields(),
            "blockers": preflight_errors,
            "next_command": status_command_for(track_id),
            "required_permissions": sorted(permissions),
        }
    deny_set = deny_permissions(intent_lock)
    if permissions & deny_set:
        overlap = sorted(permissions & deny_set)
        return {
            "verdict": "needs_authorization",
            "track_id": track_id,
            **doctrine_payload_fields(),
            "blockers": [f"{track_id}: required permissions are denied: {', '.join(overlap)}"],
            "blocked_permission": overlap[0],
            "next_command": status_command_for(track_id),
            "required_permissions": sorted(permissions),
        }
    lock_errors = execution_lock_errors(
        track_id,
        contract_pack_root=contract_pack_root,
        lock_root=lock_root,
        requested_permissions=permissions,
        run_mode="full-track",
    )
    refreshed_lock = False
    if lock_errors:
        lock = build_execution_lock(
            track_id,
            locked_by=locked_by,
            lock_scope="full-track",
            contract_pack_root=contract_pack_root,
            granted_permissions=sorted(permissions),
            denied_permissions=sorted(deny_set),
        )
        write_execution_lock(lock, root=lock_root)
        refreshed_lock = True
        lock_errors = execution_lock_errors(
            track_id,
            contract_pack_root=contract_pack_root,
            lock_root=lock_root,
            requested_permissions=permissions,
            run_mode="full-track",
        )
    if lock_errors:
        return {
            "verdict": "drift",
            "track_id": track_id,
            **doctrine_payload_fields(),
            "blockers": lock_errors,
            "next_command": status_command_for(track_id),
            "required_permissions": sorted(permissions),
        }
    return {
        "verdict": "ready",
        "track_id": track_id,
        **doctrine_payload_fields(),
        "blockers": [],
        "next_command": go_command_for(track_id),
        "required_permissions": sorted(permissions),
        "denied_permissions": sorted(deny_set),
        "lock_refreshed": refreshed_lock,
        "remaining_actions": len(pack.actions),
    }


def truth_status(track_id: str, *, manifest_source_root: Path) -> tuple[list[str], list[str]]:
    loaded = load_track_execution_manifest(track_id, root=manifest_source_root)
    if loaded is None:
        return [], []
    truth_lines = [line for line in truth_claim_summary_lines(loaded.manifest) if line != "Truth claims:"]
    certificate_lines = certificate_summary_lines(track_id, loaded.manifest.truth_claims)
    return truth_lines, certificate_lines


def post_completion_errors(track_id: str, *, production_source: Path, manifest_source_root: Path) -> list[str]:
    planning = load_production_tracks(production_source)
    track = next((candidate for candidate in planning.tracks if candidate.id == track_id), None)
    if track is None or track.state != "completed":
        return []
    loaded = load_track_execution_manifest(track_id, root=manifest_source_root)
    if loaded is None:
        return []
    errors: list[str] = []
    for claim in loaded.manifest.truth_claims:
        errors.extend(certificate_errors_for_claim(track_id, claim))
    return errors


def repo_visible_completion_drift(*, repo_root: Path = REPO_ROOT) -> list[str]:
    result = subprocess.run(
        ["git", "status", "--porcelain=v1", "--untracked-files=normal"],
        cwd=repo_root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return [f"git status failed while checking durable completion: {result.stderr.strip()}"]

    drift: list[str] = []
    for line in result.stdout.splitlines():
        if not line.strip():
            continue
        path = line[3:].strip()
        if path.startswith(".runenwerk/"):
            continue
        drift.append(line)
    return drift


def latest_run_failure_for_action(
    track_id: str,
    action,
    pack,
    *,
    run_ledger_root: Path,
    production_source: Path = PRODUCTION_SOURCE,
    roadmap_source: Path = ROADMAP_SOURCE,
    manifest_source_root: Path = TRACK_EXECUTION_MANIFEST_ROOT,
) -> dict[str, Any] | None:
    run_ledger_retention_policy()
    ledger_dir = run_ledger_root / track_id.lower()
    if not ledger_dir.exists():
        return None
    for path in sorted(ledger_dir.glob("*.yaml"), reverse=True):
        data = yaml_load_mapping(path)
        actions = data.get("actions")
        if not isinstance(actions, list) or not actions:
            continue
        last_action = actions[-1]
        if not isinstance(last_action, dict):
            continue
        if last_action.get("action_id") != action.action_id:
            continue
        if last_action.get("status") != "failed":
            return None
        error = str(last_action.get("error", ""))
        if "ungranted permissions" in error:
            return None
        if "truth:audit" in error:
            return None
        if last_action.get("executor_kind") != action.executor_kind:
            continue
        if last_action.get("writer_strategy") != action.writer_strategy:
            continue
        if "validation failed: task production:validate" in error and not production_validation_errors(
            source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
        ):
            return None
        if "validation failed: task workflow:test" in error and not readonly_validation_command_fails(
            ["task", "workflow:test"]
        ):
            return None
        action_digests = last_action.get("post_action_digests") or last_action.get("pre_action_digests") or {}
        if action_digests != pack.source_digests:
            return None
        return {"path": path, "action": last_action}
    return None


def readonly_validation_command_fails(argv: list[str]) -> bool:
    result = subprocess.run(
        argv,
        cwd=REPO_ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
        timeout=900,
    )
    return result.returncode != 0


def repair_command_for_failure(track_id: str, error: str) -> str:
    if "validation command changed undeclared file" in error:
        return status_command_for(track_id)
    if "changed undeclared file" in error:
        return status_command_for(track_id)
    return status_command_for(track_id)


def repair_blocker_for_failure(error: str) -> str:
    if "validation command changed undeclared file" in error and "Cargo.lock" in error:
        return "repair validation_commands[].allowed_outputs for Cargo.lock in the active plan.contract.yaml"
    if "changed undeclared file Cargo.lock" in error:
        return "repair Cargo.lock authority; validation-produced lockfile updates must be declared in validation_commands[].allowed_outputs"
    return "repair the latest failed ActionContract before rerunning track:go"


def status_payload(
    track_id: str,
    *,
    contract_pack_root: Path,
    lock_root: Path,
    intent_lock_root: Path,
    manifest_source_root: Path,
    run_ledger_root: Path = RUN_LEDGER_ROOT,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "verdict": "blocked",
        "track_id": track_id,
        **doctrine_payload_fields(),
        "next_action": None,
        "next_command": go_command_for(track_id),
        "blockers": [],
        "files_changed": [],
        "ledger_paths": [],
        "truth_findings": [],
    }
    pack, freshness = pack_status(track_id, contract_pack_root=contract_pack_root)
    truth_lines, certificate_lines = truth_status(track_id, manifest_source_root=manifest_source_root)
    payload["truth_claims"] = truth_lines
    payload["truth_certificates"] = certificate_lines
    if pack is None:
        payload["verdict"] = "blocked"
        payload["blockers"] = freshness
        payload["next_command"] = go_command_for(track_id)
        return payload
    payload["remaining_actions"] = len(pack.actions)
    if freshness:
        payload["verdict"] = "drift"
        payload["blockers"].extend(freshness)
        payload["next_command"] = go_command_for(track_id)
        return payload
    if pack.actions:
        action = pack.actions[0]
        payload["next_action"] = action.action_id
        payload["execution_kind"] = action.execution_kind
        payload["executor_kind"] = action.executor_kind
        payload["writer_strategy"] = action.writer_strategy
        latest_failure = latest_run_failure_for_action(
            track_id,
            action,
            pack,
            run_ledger_root=run_ledger_root,
            manifest_source_root=manifest_source_root,
        )
        if latest_failure is not None:
            error = str(latest_failure["action"].get("error", "latest run failed for current action"))
            payload["verdict"] = "blocked"
            payload["blockers"].extend(
                [
                    f"latest run failed for current action: {error}",
                    repair_blocker_for_failure(error),
                    f"run ledger: {repo_path(latest_failure['path'])}",
                ]
            )
            payload["next_command"] = repair_command_for_failure(track_id, error)
            return payload
    else:
        completion_drift = repo_visible_completion_drift()
        if completion_drift:
            payload["verdict"] = "complete_uncheckpointed"
            payload["blockers"].extend(
                [
                    "track has no remaining ActionContracts, but repository-visible files are not checkpointed",
                    *completion_drift[:25],
                ]
            )
            if len(completion_drift) > 25:
                payload["blockers"].append(f"... {len(completion_drift) - 25} more repo-visible changes")
            payload["next_command"] = "checkpoint the current validated repository state, then rerun task track"
            return payload
        payload["verdict"] = "complete"
        payload["next_command"] = status_command_for(track_id)
        return payload
    permissions = required_permissions(pack)
    payload["required_permissions"] = sorted(permissions)
    intent_lock, blockers, blocked_permission = intent_errors(
        track_id,
        permissions=permissions,
        intent_lock_root=intent_lock_root,
    )
    if blockers:
        payload["verdict"] = "needs_authorization"
        payload["blockers"].extend(blockers)
        payload["next_command"] = allow_command_for(track_id, blocked_permission or "crate_creation")
        return payload
    lock_errors = execution_lock_errors(
        track_id,
        contract_pack_root=contract_pack_root,
        lock_root=lock_root,
        requested_permissions=permissions,
        run_mode="full-track",
    )
    if lock_errors:
        payload["verdict"] = "drift"
        payload["blockers"].extend(lock_errors)
        payload["next_command"] = go_command_for(track_id)
        return payload
    if pack.actions:
        payload["verdict"] = "ready"
        payload["blockers"] = []
        payload["next_command"] = go_command_for(track_id)
    return payload


def print_payload(payload: dict[str, Any], *, as_json: bool) -> None:
    payload = dict(payload)
    payload.setdefault("quality_doctrine_id", QUALITY_DOCTRINE_ID)
    payload.setdefault("quality_doctrine", quality_doctrine_lines())
    if as_json:
        print(json.dumps(payload, sort_keys=True, indent=2))
        return
    console.print(f"Quality doctrine: {payload.get('quality_doctrine_id', QUALITY_DOCTRINE_ID)}")
    console.print(f"Verdict: {payload.get('verdict')}")
    console.print(f"Track: {payload.get('track_id')}")
    if payload.get("next_action"):
        console.print(f"Next action: {payload['next_action']}")
    if payload.get("required_permissions"):
        console.print(f"Required permissions: {', '.join(payload['required_permissions'])}")
    if payload.get("truth_claims"):
        console.print("Truth claims:")
        for line in payload["truth_claims"]:
            console.print(line)
    if payload.get("truth_certificates"):
        console.print("Truth certificates:")
        for line in payload["truth_certificates"]:
            console.print(line)
    if payload.get("blockers"):
        console.print("Blockers:")
        for blocker in payload["blockers"]:
            console.print(f"- {blocker}")
    console.print(f"Next command: {payload.get('next_command')}")


@app.command("status")
def status(
    track: str | None = typer.Option(None, "--track"),
    as_json: bool = typer.Option(False, "--json"),
    local_state_path: Path = typer.Option(LOCAL_STATE_PATH),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
    lock_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
    intent_lock_root: Path = typer.Option(TRACK_INTENT_LOCK_ROOT),
    run_ledger_root: Path = typer.Option(RUN_LEDGER_ROOT),
) -> None:
    try:
        track_id = resolve_track(track, local_state_path=local_state_path)
        payload = status_payload(
            track_id,
            contract_pack_root=contract_pack_root,
            lock_root=lock_root,
            intent_lock_root=intent_lock_root,
            manifest_source_root=manifest_source_root,
            run_ledger_root=run_ledger_root,
        )
        print_payload(payload, as_json=as_json)
    except WorkflowError as error:
        payload = {
            "verdict": "blocked",
            "track_id": track or "",
            **doctrine_payload_fields(),
            "next_action": None,
            "next_command": "task track:use -- --track <TRACK_ID>",
            "blockers": str(error).splitlines(),
            "files_changed": [],
            "ledger_paths": [],
            "truth_findings": [],
        }
        print_payload(payload, as_json=as_json)
        raise typer.Exit(1) from error


@app.command("use")
def use_track(
    track: str = typer.Option(..., "--track"),
    set_by: str = typer.Option("human", "--set-by"),
    note: str = typer.Option("", "--note"),
    local_state_path: Path = typer.Option(LOCAL_STATE_PATH),
) -> None:
    try:
        state = ActiveTrackState(track_id=track, set_by=set_by, set_at=now_utc_iso(), note=note)
        path = write_active_track(state, local_state_path=local_state_path)
        console.print("[green]Active track set.[/green]")
        console.print(f"State: {repo_path(path)}")
        console.print(f"Track: {track}")
    except WorkflowError as error:
        console.print("[red]track:use failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("allow")
def allow_permission(
    track: str | None = typer.Option(None, "--track"),
    permission: str = typer.Option(..., "--permission"),
    reason: str = typer.Option(..., "--reason"),
    by: str = typer.Option("human", "--by"),
    local_state_path: Path = typer.Option(LOCAL_STATE_PATH),
    intent_lock_root: Path = typer.Option(TRACK_INTENT_LOCK_ROOT),
) -> None:
    try:
        track_id = resolve_track(track, local_state_path=local_state_path)
        permission = clean_text(permission, "permission")
        if permission == "foundation_extraction":
            raise WorkflowError("foundation_extraction requires a separate accepted extraction workflow; task track:allow cannot grant it")
        if permission not in STRATEGIC_PERMISSIONS:
            raise WorkflowError(f"{permission}: task track:allow is only for strategic permissions")
        lock = load_intent_lock(track_id, root=intent_lock_root) or TrackIntentLock(track_id=track_id, updated_at=now_utc_iso())
        lock.denied_permissions.pop(permission, None)
        lock.granted_permissions[permission] = PermissionAuthorization(
            permission=permission,
            by=by,
            at=now_utc_iso(),
            reason=reason,
        )
        ensure_default_denials(lock, by=by)
        lock.updated_at = now_utc_iso()
        path = write_intent_lock(lock, root=intent_lock_root)
        console.print("[green]Track Intent Lock updated.[/green]")
        console.print(f"Intent Lock: {repo_path(path)}")
        console.print(f"Granted: {permission}")
    except WorkflowError as error:
        console.print("[red]track:allow failed[/red]")
        for line in str(error).splitlines():
            console.print(f"- {line}")
        raise typer.Exit(1) from error


@app.command("go")
def go(
    track: str | None = typer.Option(None, "--track"),
    as_json: bool = typer.Option(False, "--json"),
    max_actions: int = typer.Option(999, "--max-actions", min=1),
    locked_by: str = typer.Option("track-control", "--locked-by"),
    production_source: Path = typer.Option(PRODUCTION_SOURCE),
    roadmap_source: Path = typer.Option(ROADMAP_SOURCE),
    manifest_source_root: Path = typer.Option(TRACK_EXECUTION_MANIFEST_ROOT),
    contract_pack_root: Path = typer.Option(CONTRACT_PACK_ROOT),
    lock_root: Path = typer.Option(EXECUTION_LOCK_ROOT),
    intent_lock_root: Path = typer.Option(TRACK_INTENT_LOCK_ROOT),
    run_ledger_root: Path = typer.Option(RUN_LEDGER_ROOT),
    evidence_root: Path | None = typer.Option(None, "--evidence-root"),
    local_state_path: Path = typer.Option(LOCAL_STATE_PATH),
) -> None:
    output_buffer = io.StringIO()
    try:
        track_id = resolve_track(track, local_state_path=local_state_path)
        prepare_payload = prepare_execution(
            track_id,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
            contract_pack_root=contract_pack_root,
            lock_root=lock_root,
            intent_lock_root=intent_lock_root,
            locked_by=locked_by,
        )
        if prepare_payload["verdict"] != "ready":
            print_payload(prepare_payload, as_json=as_json)
            raise typer.Exit(1)
        if prepare_payload.get("remaining_actions", 0) == 0:
            completion_drift = repo_visible_completion_drift()
            payload = {
                **prepare_payload,
                "verdict": "complete_uncheckpointed" if completion_drift else "complete",
                **doctrine_payload_fields(),
                "next_action": None,
                "next_command": (
                    "checkpoint the current validated repository state, then rerun task track"
                    if completion_drift
                    else status_command_for(track_id)
                ),
                "blockers": (
                    [
                        "track has no remaining ActionContracts, but repository-visible files are not checkpointed",
                        *completion_drift[:25],
                    ]
                    if completion_drift
                    else []
                ),
                "files_changed": [],
                "ledger_paths": [],
                "truth_findings": [],
            }
            print_payload(payload, as_json=as_json)
            if completion_drift:
                raise typer.Exit(1)
            return
        run_kwargs = dict(
            track=track_id,
            mode="full-track",
            allow=prepare_payload["required_permissions"],
            deny=prepare_payload.get("denied_permissions", sorted(DEFAULT_DENIED_PERMISSIONS)),
            max_actions=max_actions,
            production_source=production_source,
            roadmap_source=roadmap_source,
            manifest_source_root=manifest_source_root,
            contract_pack_root=contract_pack_root,
            lock_root=lock_root,
            run_ledger_root=run_ledger_root,
            evidence_root=evidence_root or EVIDENCE_ROOT,
            repo_root=REPO_ROOT,
        )
        try:
            if as_json:
                with contextlib.redirect_stdout(output_buffer), contextlib.redirect_stderr(output_buffer):
                    execution_run_command(**run_kwargs)
            else:
                execution_run_command(**run_kwargs)
        except typer.Exit as error:
            failure_status = status_payload(
                track_id,
                contract_pack_root=contract_pack_root,
                lock_root=lock_root,
                intent_lock_root=intent_lock_root,
                manifest_source_root=manifest_source_root,
                run_ledger_root=run_ledger_root,
            )
            if failure_status["verdict"] == "blocked":
                if as_json:
                    failure_status["captured_output"] = output_buffer.getvalue().splitlines()
                print_payload(failure_status, as_json=as_json)
                raise typer.Exit(error.exit_code or 1) from error
            payload = {
                "verdict": "blocked",
                "track_id": track_id,
                **doctrine_payload_fields(),
                "next_action": None,
                "next_command": go_command_for(track_id),
                "blockers": ["execution kernel stopped before track completion"],
                "files_changed": [],
                "ledger_paths": [],
                "truth_findings": [],
                "captured_output": output_buffer.getvalue().splitlines() if as_json else [],
            }
            print_payload(payload, as_json=as_json)
            raise typer.Exit(error.exit_code or 1) from error
        pack = load_contract_pack(track_id, root=contract_pack_root)
        errors = post_completion_errors(track_id, production_source=production_source, manifest_source_root=manifest_source_root)
        if errors:
            payload = {
                "verdict": "truth_blocked",
                "track_id": track_id,
                **doctrine_payload_fields(),
                "next_action": pack.actions[0].action_id if pack and pack.actions else None,
                "next_command": go_command_for(track_id),
                "blockers": errors,
                "files_changed": [],
                "ledger_paths": [],
                "truth_findings": errors,
                "captured_output": output_buffer.getvalue().splitlines() if as_json else [],
            }
            print_payload(payload, as_json=as_json)
            raise typer.Exit(1)
        completion_drift = repo_visible_completion_drift() if pack is not None and not pack.actions else []
        payload = {
            "verdict": (
                "complete_uncheckpointed"
                if completion_drift
                else "complete"
                if pack is not None and not pack.actions
                else "blocked"
            ),
            "track_id": track_id,
            **doctrine_payload_fields(),
            "next_action": pack.actions[0].action_id if pack and pack.actions else None,
            "next_command": (
                "checkpoint the current validated repository state, then rerun task track"
                if completion_drift
                else status_command_for(track_id)
                if pack is not None and not pack.actions
                else go_command_for(track_id)
            ),
            "blockers": (
                [
                    "track has no remaining ActionContracts, but repository-visible files are not checkpointed",
                    *completion_drift[:25],
                ]
                if completion_drift
                else []
                if pack is not None and not pack.actions
                else ["track run stopped before all actions completed"]
            ),
            "files_changed": [],
            "ledger_paths": [],
            "truth_findings": [],
            "captured_output": output_buffer.getvalue().splitlines() if as_json else [],
        }
        print_payload(payload, as_json=as_json)
        if payload["verdict"] != "complete":
            raise typer.Exit(1)
    except WorkflowError as error:
        payload = {
            "verdict": "blocked",
            "track_id": track or "",
            **doctrine_payload_fields(),
            "next_action": None,
            "next_command": status_command_for(track) if track else "task track",
            "blockers": str(error).splitlines(),
            "files_changed": [],
            "ledger_paths": [],
            "truth_findings": [],
            "captured_output": output_buffer.getvalue().splitlines() if as_json else [],
        }
        print_payload(payload, as_json=as_json)
        raise typer.Exit(1) from error


@app.command("_commands", hidden=True)
def commands() -> None:
    console.print("status use allow go")


if __name__ == "__main__":
    app()
