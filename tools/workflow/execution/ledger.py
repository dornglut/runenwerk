from __future__ import annotations

from pathlib import Path

import yaml

from roadmap_state import REPO_ROOT, repo_path

from execution.contracts import ActionContract, now_utc_iso
from execution.runner import HarnessRunResult


RUN_LEDGER_ROOT = REPO_ROOT / "docs-site/src/content/docs/reports/track-execution-runs"


def new_run_id(track_id: str) -> str:
    return f"{track_id.lower()}-{now_utc_iso().replace(':', '').replace('-', '').replace('T', '-').replace('Z', 'z')}"


def run_ledger_path(track_id: str, run_id: str, *, root: Path = RUN_LEDGER_ROOT) -> Path:
    return root / track_id.lower() / f"{run_id}.yaml"


def append_run_action(
    *,
    track_id: str,
    run_id: str,
    action: ActionContract,
    result: HarnessRunResult,
    pre_action_digests: dict[str, str],
    post_action_digests: dict[str, str],
    root: Path = RUN_LEDGER_ROOT,
    stop_reason: str,
) -> Path:
    path = run_ledger_path(track_id, run_id, root=root)
    if path.exists():
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            data = {}
    else:
        data = {
            "version": 1,
            "track_id": track_id,
            "run_id": run_id,
            "started_at": now_utc_iso(),
            "actions": [],
        }
    data["updated_at"] = now_utc_iso()
    data.setdefault("actions", []).append(
        {
            "status": "passed",
            "action_id": action.action_id,
            "milestone_id": action.milestone_id,
            "wr_id": action.wr_id,
            "execution_kind": action.execution_kind,
            "executor_kind": action.executor_kind,
            "writer_strategy": action.writer_strategy,
            "files_changed": [repo_path(path) for path in result.written_paths],
            "agent_files_changed": [repo_path(path) for path in result.agent_files_changed],
            "validation_files_changed": [repo_path(path) for path in result.validation_files_changed],
            "evidence_files_changed": [repo_path(path) for path in result.evidence_files_changed],
            "validation_results": [
                {
                    "command_id": validation.command_id,
                    "argv": list(validation.argv),
                    "returncode": validation.returncode,
                    "files_changed": list(validation.files_changed),
                }
                for validation in result.validation_results
            ],
            "evidence_paths": [repo_path(path) for path in result.evidence_paths],
            "transcript_paths": [repo_path(path) for path in result.transcript_paths],
            "pre_action_digests": pre_action_digests,
            "post_action_digests": post_action_digests,
            "next_legal_action": result.next_action,
            "stop_reason": stop_reason,
        }
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(data, sort_keys=False, width=4096), encoding="utf-8", newline="\n")
    return path


def update_latest_run_action_state(
    *,
    track_id: str,
    run_id: str,
    action_id: str,
    post_action_digests: dict[str, str],
    next_legal_action: str,
    stop_reason: str,
    root: Path = RUN_LEDGER_ROOT,
) -> Path:
    path = run_ledger_path(track_id, run_id, root=root)
    data = yaml.safe_load(path.read_text(encoding="utf-8")) if path.exists() else None
    if not isinstance(data, dict):
        raise ValueError(f"{repo_path(path)} must contain a run ledger before it can be updated")
    actions = data.get("actions")
    if not isinstance(actions, list):
        raise ValueError(f"{repo_path(path)} must contain an actions list")
    for entry in reversed(actions):
        if isinstance(entry, dict) and entry.get("status") == "passed" and entry.get("action_id") == action_id:
            entry["post_action_digests"] = post_action_digests
            entry["next_legal_action"] = next_legal_action
            entry["stop_reason"] = stop_reason
            data["updated_at"] = now_utc_iso()
            path.write_text(yaml.safe_dump(data, sort_keys=False, width=4096), encoding="utf-8", newline="\n")
            return path
    raise ValueError(f"{repo_path(path)} has no passed action entry for {action_id}")


def append_run_failure(
    *,
    track_id: str,
    run_id: str,
    action: ActionContract | None,
    error: str,
    pre_action_digests: dict[str, str],
    post_action_digests: dict[str, str],
    stop_reason: str,
    transcript_paths: tuple[Path, ...] = (),
    root: Path = RUN_LEDGER_ROOT,
) -> Path:
    path = run_ledger_path(track_id, run_id, root=root)
    if path.exists():
        data = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            data = {}
    else:
        data = {
            "version": 1,
            "track_id": track_id,
            "run_id": run_id,
            "started_at": now_utc_iso(),
            "actions": [],
        }
    data["updated_at"] = now_utc_iso()
    data.setdefault("actions", []).append(
        {
            "status": "failed",
            "action_id": action.action_id if action is not None else None,
            "milestone_id": action.milestone_id if action is not None else None,
            "wr_id": action.wr_id if action is not None else None,
            "execution_kind": action.execution_kind if action is not None else None,
            "executor_kind": action.executor_kind if action is not None else None,
            "writer_strategy": action.writer_strategy if action is not None else None,
            "files_changed": [],
            "validation_results": [],
            "evidence_paths": [],
            "transcript_paths": [repo_path(path) for path in transcript_paths],
            "pre_action_digests": pre_action_digests,
            "post_action_digests": post_action_digests,
            "next_legal_action": "blocked; repair the failure and rerun the harness.",
            "stop_reason": stop_reason,
            "error": error,
        }
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(data, sort_keys=False, width=4096), encoding="utf-8", newline="\n")
    return path
