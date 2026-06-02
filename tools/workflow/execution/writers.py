from __future__ import annotations

import subprocess
import threading
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Protocol

import yaml

from roadmap_state import REPO_ROOT, WorkflowError, repo_path
from prompt_doctrine import quality_doctrine_block

from execution.contracts import ActionContract


@dataclass(frozen=True)
class AgentResult:
    returncode: int
    stdout: str
    stderr: str
    transcript_paths: tuple[Path, ...] = ()
    timed_out: bool = False


class AgentBackend(Protocol):
    def run(self, *, workspace: Path, prompt: str, transcript_dir: Path | None = None) -> AgentResult:
        ...


class AgentWriterError(WorkflowError):
    def __init__(self, message: str, *, transcript_paths: tuple[Path, ...] = ()) -> None:
        super().__init__(message)
        self.transcript_paths = transcript_paths


class CodexExecBackend:
    def __init__(self, *, codex_bin: str = "codex", timeout_seconds: int = 1800) -> None:
        self.codex_bin = codex_bin
        self.timeout_seconds = timeout_seconds

    def run(self, *, workspace: Path, prompt: str, transcript_dir: Path | None = None) -> AgentResult:
        transcript_paths: list[Path] = []
        started_at = now_utc_iso()
        if transcript_dir is not None:
            transcript_dir.mkdir(parents=True, exist_ok=True)
            prompt_path = transcript_dir / "prompt.md"
            prompt_path.write_text(prompt_transcript_markdown(prompt), encoding="utf-8", newline="\n")
            transcript_paths.append(prompt_path)
        stdout_chunks: list[str] = []
        stderr_chunks: list[str] = []
        stdout_path = transcript_dir / "stdout.log" if transcript_dir is not None else None
        stderr_path = transcript_dir / "stderr.log" if transcript_dir is not None else None
        if stdout_path is not None:
            stdout_path.write_text("", encoding="utf-8")
            transcript_paths.append(stdout_path)
        if stderr_path is not None:
            stderr_path.write_text("", encoding="utf-8")
            transcript_paths.append(stderr_path)
        timed_out = False
        try:
            process = subprocess.Popen(
                [
                    self.codex_bin,
                    "exec",
                    "--ephemeral",
                    "--skip-git-repo-check",
                    "--sandbox",
                    "workspace-write",
                    "-C",
                    str(workspace),
                    "-",
                ],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
            )
        except OSError as error:
            return AgentResult(
                returncode=127,
                stdout="",
                stderr=str(error),
                transcript_paths=tuple(transcript_paths),
            )
        assert process.stdout is not None
        assert process.stderr is not None
        threads = [
            threading.Thread(
                target=stream_process_output,
                args=(process.stdout, stdout_chunks, stdout_path),
                daemon=True,
            ),
            threading.Thread(
                target=stream_process_output,
                args=(process.stderr, stderr_chunks, stderr_path),
                daemon=True,
            ),
        ]
        for thread in threads:
            thread.start()
        if process.stdin is not None:
            process.stdin.write(prompt)
            process.stdin.close()
        try:
            returncode = process.wait(timeout=self.timeout_seconds)
        except subprocess.TimeoutExpired:
            timed_out = True
            process.kill()
            process.wait()
            returncode = 124
        for thread in threads:
            thread.join(timeout=5)
        completed_at = now_utc_iso()
        summary_path = None
        if transcript_dir is not None:
            summary_path = transcript_dir / "summary.yaml"
            summary_path.write_text(
                yaml.safe_dump(
                    {
                        "started_at": started_at,
                        "completed_at": completed_at,
                        "returncode": returncode,
                        "timed_out": timed_out,
                        "workspace": str(workspace),
                    },
                    sort_keys=False,
                    width=4096,
                ),
                encoding="utf-8",
                newline="\n",
            )
            transcript_paths.append(summary_path)
        stderr_text = "".join(stderr_chunks)
        if timed_out and not stderr_text.strip():
            stderr_text = "codex exec timed out"
            if stderr_path is not None:
                stderr_path.write_text(stderr_text + "\n", encoding="utf-8")
        return AgentResult(
            returncode=returncode,
            stdout="".join(stdout_chunks),
            stderr=stderr_text,
            transcript_paths=tuple(transcript_paths),
            timed_out=timed_out,
        )


def subprocess_output_text(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def now_utc_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def prompt_transcript_markdown(prompt: str) -> str:
    return "\n".join(
        [
            "---",
            'title: "Agent Prompt Transcript"',
            "status: completed",
            "---",
            "",
            prompt,
        ]
    )


def stream_process_output(stream, chunks: list[str], transcript_path: Path | None) -> None:
    with stream:
        for chunk in iter(stream.readline, ""):
            chunks.append(chunk)
            if transcript_path is not None:
                with transcript_path.open("a", encoding="utf-8") as handle:
                    handle.write(chunk)


def safe_transcript_name(value: str) -> str:
    safe = "".join(character if character.isalnum() or character in {"-", "_"} else "-" for character in value.lower())
    while "--" in safe:
        safe = safe.replace("--", "-")
    return safe.strip("-") or "agent-action"


def action_context(workspace: Path, action: ActionContract) -> str:
    snippets: list[str] = []
    plan_root = workspace / "docs-site/src/content/docs/reports/implementation-plans"
    if plan_root.exists():
        for path in sorted(plan_root.glob(f"{action.wr_id.lower()}-*/*")):
            if path.name not in {"plan.md", "plan.contract.yaml"}:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except OSError:
                continue
            relative = path.relative_to(workspace).as_posix()
            snippets.append(f"--- {relative} ---\n{text[:8000]}")
    return "\n\n".join(snippets)


def action_prompt(action: ActionContract, *, workspace: Path) -> str:
    allowed = "\n".join(f"- {path}" for path in [*action.allowed_outputs, *action.new_outputs])
    forbidden = "\n".join(f"- {path}" for path in action.forbidden_outputs)
    validations = "\n".join(f"- {' '.join(command.argv)}" for command in action.validation_commands)
    evidence = "\n".join(
        f"- {requirement.kind}: {requirement.name} ({', '.join(requirement.paths) or 'no declared path'})"
        for requirement in action.evidence_required
    )
    stop_conditions = "\n".join(f"- {condition}" for condition in action.stop_conditions)
    return "\n".join(
        [
            "You are running inside an isolated Track Execution Harness workspace.",
            "Read AGENTS.md and nearby source before editing.",
            "Modify only the allowed outputs. Do not run destructive git commands.",
            "If the requested implementation cannot be completed within the allowed outputs, stop with a clear error instead of touching other files.",
            "",
            quality_doctrine_block(),
            "",
            f"Action: {action.action_id}",
            f"Parent action: {action.parent_action_id}" if action.parent_action_id else "Parent action: none",
            (
                f"Agent sub-action: {action.agent_subaction.sub_action_id} - {action.agent_subaction.title}"
                if action.agent_subaction is not None
                else "Agent sub-action: none"
            ),
            f"Execution kind: {action.execution_kind}",
            f"Executor kind: {action.executor_kind}",
            f"Authority level: {action.authority_level}",
            "",
            "Sub-action prompt:",
            action.agent_subaction.prompt if action.agent_subaction is not None else "- none",
            "",
            "Allowed outputs:",
            allowed or "- none",
            "",
            "Forbidden outputs:",
            forbidden or "- none",
            "",
            "Forbidden patterns:",
            "\n".join(f"- {pattern}" for pattern in action.forbidden_patterns) or "- none",
            "",
            "Required evidence records:",
            evidence or "- none",
            "",
            "Validation commands after import:",
            validations or "- none",
            "",
            "Stop conditions:",
            stop_conditions or "- none",
            "",
            "Active WR implementation-plan authority:",
            action_context(workspace, action) or "- no implementation-plan context found",
        ]
    )


def ensure_path_allowed(action: ActionContract, path: str) -> None:
    allowed = set(action.allowed_outputs) | set(action.new_outputs)
    if path not in allowed:
        raise WorkflowError(f"{action.action_id}: writer output is not declared: {path}")


def run_template_writer(action: ActionContract, *, workspace_root: Path) -> list[Path]:
    if not action.template_outputs:
        raise WorkflowError(f"{action.action_id}: template_writer requires template_outputs")
    written: list[Path] = []
    for relative, content in action.template_outputs.items():
        ensure_path_allowed(action, relative)
        target = workspace_root / relative
        if not target.parent.exists():
            if relative not in action.new_outputs:
                raise WorkflowError(f"{action.action_id}: output parent directory does not exist: {repo_path(target.parent)}")
            target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8", newline="\n")
        written.append(target)
    return written


def run_patch_writer(action: ActionContract, *, workspace_root: Path) -> list[Path]:
    if not action.patches:
        raise WorkflowError(f"{action.action_id}: patch_writer requires patches")
    written: list[Path] = []
    for patch in action.patches:
        ensure_path_allowed(action, patch.path)
        target = workspace_root / patch.path
        if not target.exists():
            raise WorkflowError(f"{action.action_id}: patch target does not exist: {patch.path}")
        text = target.read_text(encoding="utf-8")
        if patch.find not in text:
            raise WorkflowError(f"{action.action_id}: patch find text not found in {patch.path}")
        target.write_text(text.replace(patch.find, patch.replace, 1), encoding="utf-8", newline="\n")
        written.append(target)
    return written


def run_agent_writer(
    action: ActionContract,
    *,
    backend: AgentBackend,
    lock_validated: bool,
    workspace_root: Path,
    transcript_root: Path | None = None,
    transcript_paths_out: list[Path] | None = None,
) -> list[Path]:
    if not lock_validated:
        raise WorkflowError(f"{action.action_id}: agent_writer requires a current execution lock")
    transcript_dir = None
    if transcript_root is not None:
        subaction_suffix = (
            f"sub-{safe_transcript_name(action.agent_subaction.sub_action_id)}"
            if action.agent_subaction is not None
            else "full-action"
        )
        transcript_dir = transcript_root / safe_transcript_name(action.action_id) / subaction_suffix
    result = backend.run(
        workspace=workspace_root,
        prompt=action_prompt(action, workspace=workspace_root),
        transcript_dir=transcript_dir,
    )
    if transcript_paths_out is not None:
        transcript_paths_out.extend(result.transcript_paths)
    if result.returncode != 0:
        details = "\n".join(part for part in (result.stdout.strip(), result.stderr.strip()) if part)
        suffix = f"\n{details[:4000]}" if details else ""
        transcript_note = (
            "\nTranscripts:\n" + "\n".join(f"- {repo_path(path)}" for path in result.transcript_paths)
            if result.transcript_paths
            else ""
        )
        raise AgentWriterError(
            f"{action.action_id}: agent backend failed with exit {result.returncode}{suffix}{transcript_note}",
            transcript_paths=result.transcript_paths,
        )
    return []


def run_writer_in_workspace(
    action: ActionContract,
    workspace_root: Path,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool = False,
    transcript_root: Path | None = None,
    transcript_paths_out: list[Path] | None = None,
) -> list[Path]:
    if action.writer_strategy == "no_writer":
        raise WorkflowError(f"{action.action_id}: no_writer cannot execute")
    if action.writer_strategy == "template_writer":
        return run_template_writer(action, workspace_root=workspace_root)
    if action.writer_strategy == "patch_writer":
        return run_patch_writer(action, workspace_root=workspace_root)
    if action.writer_strategy == "agent_writer":
        if backend is None:
            backend = CodexExecBackend()
        return run_agent_writer(
            action,
            backend=backend,
            lock_validated=lock_validated,
            workspace_root=workspace_root,
            transcript_root=transcript_root,
            transcript_paths_out=transcript_paths_out,
        )
    if action.writer_strategy == "verification_writer":
        return []
    if action.writer_strategy == "proof_aggregation_writer":
        return []
    raise WorkflowError(f"{action.action_id}: unsupported writer strategy {action.writer_strategy}")


def run_writer(
    action: ActionContract,
    *,
    backend: AgentBackend | None = None,
    lock_validated: bool = False,
    repo_root: Path = REPO_ROOT,
    run_validations: bool = True,
    transcript_root: Path | None = None,
) -> list[Path]:
    from execution.workspace import create_full_snapshot, dispose_workspace, import_scoped_changes, reset_validation_only_outputs

    workspace = create_full_snapshot(action, repo_root=repo_root)
    try:
        run_writer_in_workspace(
            action,
            workspace.workspace,
            backend=backend,
            lock_validated=lock_validated,
            transcript_root=transcript_root,
        )
        reset_validation_only_outputs(action, workspace)
        if run_validations:
            from execution.runner import run_validation_commands

            run_validation_commands(action.validation_commands, repo_root=workspace.workspace, action=action)
        return import_scoped_changes(action, workspace, repo_root=repo_root)
    finally:
        dispose_workspace(workspace)
