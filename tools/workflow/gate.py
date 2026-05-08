#!/usr/bin/env python3
"""
Run stable Runenwerk workflow commands.

File: tools/workflow/gate.py
Function: main
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from ai_task import build_shapes, render_shape


REPO_ROOT = Path(__file__).resolve().parents[2]
SHAPE_ALIASES = {
    "closeout": "phase-closeout",
}
COMMAND_ALIASES = {
    "phase-closeout": "closeout",
}


def run_command(command: list[str]) -> int:
    completed = subprocess.run(command, cwd=REPO_ROOT)
    return completed.returncode


def public_workflow_names() -> list[str]:
    names = set(build_shapes())
    names.difference_update(COMMAND_ALIASES)
    names.update(SHAPE_ALIASES)
    return sorted(names)


def normalize_command_aliases(argv: list[str]) -> list[str]:
    if argv and argv[0] in COMMAND_ALIASES:
        normalized = list(argv)
        normalized[0] = COMMAND_ALIASES[argv[0]]
        return normalized
    return argv


def canonical_shape_name(workflow: str) -> str:
    return SHAPE_ALIASES.get(workflow, workflow)


def render_prompt(args: argparse.Namespace) -> int:
    shape_name = canonical_shape_name(args.workflow)
    shapes = build_shapes()
    print(render_shape(shapes[shape_name], args.task, args.scope, args.roadmap))
    return 0


def add_prompt_args(parser: argparse.ArgumentParser, workflow: str) -> None:
    parser.set_defaults(handler=render_prompt, workflow=workflow)
    parser.add_argument(
        "--task",
        default="<task>",
        help="Task, milestone, or completed phase text.",
    )
    parser.add_argument(
        "--scope",
        default="<crate/files/subsystem>",
        help="Owning crate, files, subsystem, or repository area.",
    )
    parser.add_argument(
        "--roadmap",
        default="<owning roadmap/design path>",
        help="Owning roadmap/design path for milestone and closeout prompts.",
    )


def handle_docs(_: argparse.Namespace) -> int:
    return run_command(["python3", "tools/docs/validate_docs.py"])


def handle_full_gate(_: argparse.Namespace) -> int:
    return run_command(["./quiet_full_gate.sh"])


def handle_list(_: argparse.Namespace) -> int:
    shapes = build_shapes()
    print("Executable workflow commands:")
    print("- docs: run docs validation")
    print("- full-gate: run the full quiet workspace gate")
    print("")
    print("Prompt/checklist workflow commands:")
    for name in public_workflow_names():
        canonical = canonical_shape_name(name)
        print(f"- {name}: {shapes[canonical].description}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="workflow",
        description="Run stable Runenwerk workflow commands.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True, metavar="command")

    docs = subparsers.add_parser("docs", help="Run documentation validation.")
    docs.set_defaults(handler=handle_docs)

    full_gate = subparsers.add_parser("full-gate", help="Run the full quiet workspace gate.")
    full_gate.set_defaults(handler=handle_full_gate)

    list_parser = subparsers.add_parser("list", help="List available workflow commands.")
    list_parser.set_defaults(handler=handle_list)

    shapes = build_shapes()

    for name in public_workflow_names():
        canonical = canonical_shape_name(name)
        prompt_parser = subparsers.add_parser(
            name,
            help=shapes[canonical].description,
        )
        add_prompt_args(prompt_parser, name)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args(normalize_command_aliases(sys.argv[1:]))
    return args.handler(args)


if __name__ == "__main__":
    raise SystemExit(main())
