#!/usr/bin/env python3
"""
Generate a Runenwerk AI workflow kickoff prompt and checklist.

File: tools/workflow/ai_task.py
Function: main
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from textwrap import dedent


ROOT_DOCS = [
    "AGENTS.md",
    "AI_GUIDE.md",
    "ARCHITECTURE.md",
    "DEPENDENCY_RULES.md",
    "DOMAIN_MAP.md",
    "GLOSSARY.md",
    "TESTING.md",
]


@dataclass(frozen=True)
class WorkflowShape:
    name: str
    description: str
    primary_docs: tuple[str, ...]
    prompt: str
    validation: tuple[str, ...]
    stop_conditions: tuple[str, ...]


def build_shapes() -> dict[str, WorkflowShape]:
    common_docs = (
        "docs-site/src/content/docs/workspace/planning-and-implementation-workflow.md",
    )

    return {
        "investigation": WorkflowShape(
            name="investigation",
            description="Understand current repo truth without editing files.",
            primary_docs=common_docs,
            prompt=dedent(
                """\
                Investigate this Runenwerk area.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md first and follow the referenced root docs.
                2. Inspect current code, tests, docs, and git state before answering.
                3. Do not edit files.
                4. Report owning domains, relevant files/modules, current behavior, gaps, and likely next implementation steps.
                """
            ),
            validation=("No validation required unless files are changed.",),
            stop_conditions=(
                "Ownership cannot be determined from current code or docs.",
                "Evidence is missing for a firm recommendation.",
            ),
        ),
        "planning": WorkflowShape(
            name="planning",
            description="Plan architecture, ownership, phase boundaries, or validation without editing.",
            primary_docs=common_docs
            + (
                "docs-site/src/content/docs/workspace/prompt-templates/crate-design.md",
                "docs-site/src/content/docs/workspace/documentation-structure.md",
            ),
            prompt=dedent(
                """\
                Plan this Runenwerk change.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md first and follow the referenced root docs.
                2. Inspect current repo truth before proposing a plan.
                3. Do not edit files.
                4. Name the owning domain, crate, subsystem, and likely modules.
                5. Include invariants, existing helpers, validation commands, stop conditions, and deferred work.
                """
            ),
            validation=("No validation required unless files are changed.",),
            stop_conditions=(
                "The change requires an unaccepted architectural decision.",
                "The owner or dependency direction is unclear.",
            ),
        ),
        "implementation": WorkflowShape(
            name="implementation",
            description="Make a bounded code/docs change and verify it.",
            primary_docs=common_docs
            + ("docs-site/src/content/docs/workspace/prompt-templates/implementation-batch.md",),
            prompt=dedent(
                """\
                Implement this bounded Runenwerk change.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md first and follow the referenced root docs.
                2. Inspect existing code and tests before editing.
                3. Preserve unrelated dirty work.
                4. Respect domain ownership and dependency direction.
                5. Implement the smallest coherent change.
                6. Add or update focused tests for changed invariants.
                7. Update docs when public behavior, architecture, routines, validation, or roadmap state changes.
                8. Run focused validation and report skipped checks explicitly.
                """
            ),
            validation=(
                "cargo fmt --all -- --check",
                "cargo test -p <changed-crate>",
                "python3 tools/docs/validate_docs.py  # when docs changed",
            ),
            stop_conditions=(
                "Ownership is unclear.",
                "A forbidden dependency would be required.",
                "The task expands into later phases or unrelated domains.",
            ),
        ),
        "milestone": WorkflowShape(
            name="milestone",
            description="Implement a named roadmap/design milestone end to end.",
            primary_docs=common_docs
            + (
                "docs-site/src/content/docs/workspace/prompt-templates/roadmap-milestone-kickoff.md",
                "{roadmap}",
            ),
            prompt=dedent(
                """\
                Implement this Runenwerk roadmap milestone.

                Milestone:
                - {task}

                Owning roadmap/design:
                - {roadmap}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md first and follow the referenced root docs.
                2. Read the owning roadmap/design section for this milestone.
                3. Inspect current repo truth before editing.
                4. Do not implement later milestones except minimal required seams.
                5. Implement code, tests, and required docs updates.
                6. Run focused validation first, then broader validation when appropriate.
                7. If the phase is complete, run the phase completion drift-check routine before moving on.
                """
            ),
            validation=(
                "cargo fmt --all -- --check",
                "cargo test -p <changed-crate>",
                "python3 tools/docs/validate_docs.py",
                "./quiet_full_gate.sh  # for milestone closeout when appropriate",
            ),
            stop_conditions=(
                "The milestone requires a dependency direction violation.",
                "The task expands into later milestones.",
                "A required design or contract is missing.",
            ),
        ),
        "refactor": WorkflowShape(
            name="refactor",
            description="Perform behavior-preserving structure, naming, boundary, or API cleanup.",
            primary_docs=common_docs
            + ("docs-site/src/content/docs/workspace/routines/code-refactor-routine.md",),
            prompt=dedent(
                """\
                Refactor this Runenwerk area.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md first and follow the referenced root docs.
                2. Inspect current implementation, tests, and call sites before editing.
                3. Preserve behavior unless an intentional behavior change is explicitly named.
                4. Keep the refactor in the owning domain/subsystem.
                5. Run focused validation and report remaining risks.
                """
            ),
            validation=(
                "cargo fmt --all -- --check",
                "cargo test -p <changed-crate>",
                "cargo check --workspace  # when the refactor crosses crate boundaries",
            ),
            stop_conditions=(
                "Behavior changes are needed but not requested.",
                "The refactor expands into unrelated domains.",
                "Validation failure is outside the refactor scope.",
            ),
        ),
        "docs-refactor": WorkflowShape(
            name="docs-refactor",
            description="Move, rename, prune, or restructure documentation.",
            primary_docs=common_docs
            + (
                "docs-site/src/content/docs/workspace/routines/docs-refactor-routine.md",
                "docs-site/src/content/docs/workspace/documentation-structure.md",
            ),
            prompt=dedent(
                """\
                Refactor this Runenwerk documentation.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md and documentation-structure.md first.
                2. Inspect inbound links and nearby README/index pages before editing.
                3. Preserve canonical docs and lifecycle status.
                4. Update internal links for any move or rename.
                5. Run docs validation.
                """
            ),
            validation=("python3 tools/docs/validate_docs.py",),
            stop_conditions=(
                "The canonical owner for the docs is unclear.",
                "A move would break historical context without a replacement link.",
            ),
        ),
        "phase-closeout": WorkflowShape(
            name="phase-closeout",
            description="Check a completed phase for drift before starting the next phase.",
            primary_docs=common_docs
            + (
                "docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md",
                "docs-site/src/content/docs/workspace/prompt-templates/phase-completion-drift-check.md",
            ),
            prompt=dedent(
                """\
                Perform a Runenwerk phase completion drift check.

                Completed phase:
                - {task}

                Owning roadmap/design:
                - {roadmap}

                Scope:
                - {scope}

                Requirements:
                1. Inspect the accepted design/roadmap, implemented code, tests, docs, and validation output.
                2. Do not start the next phase.
                3. Correct stale phase status, roadmap drift, and docs drift.
                4. Explicitly name the next phase and what remains deferred.
                5. Run docs validation and full gate when code/workspace behavior changed.
                """
            ),
            validation=(
                "python3 tools/docs/validate_docs.py",
                "./quiet_full_gate.sh  # when code or workspace behavior changed",
            ),
            stop_conditions=(
                "The phase cannot be matched to an owning roadmap/design.",
                "Validation output for the completed phase is unavailable and cannot be rerun.",
            ),
        ),
        "commit-organization": WorkflowShape(
            name="commit-organization",
            description="Group a mixed dirty working tree into coherent commits.",
            primary_docs=common_docs
            + (
                "docs-site/src/content/docs/workspace/routines/commit-splitting-routine.md",
                "docs-site/src/content/docs/workspace/prompt-templates/commit-organization.md",
            ),
            prompt=dedent(
                """\
                Organize the current Runenwerk working tree into clean commits.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Inspect git status, diff stat, name-status, and relevant diffs.
                2. Group changes by domain and architectural ownership.
                3. Do not stage or commit unless explicitly asked.
                4. Protect unrelated work from being reverted or lost.
                5. Provide exact git add commands and validation commands per commit.
                """
            ),
            validation=("Validation depends on each proposed commit group.",),
            stop_conditions=(
                "A file contains unrelated mixed edits that cannot be split safely.",
                "The user has not authorized staging or committing.",
            ),
        ),
        "review": WorkflowShape(
            name="review",
            description="Review current code or diffs without editing.",
            primary_docs=common_docs
            + (
                "docs-site/src/content/docs/workspace/prompt-templates/code-review.md",
                "docs-site/src/content/docs/workspace/routines/public-api-review-routine.md",
            ),
            prompt=dedent(
                """\
                Review this Runenwerk change.

                Task:
                - {task}

                Scope:
                - {scope}

                Requirements:
                1. Read AGENTS.md first and inspect the actual changed files.
                2. Do not edit files.
                3. Prioritize correctness, regressions, ownership boundaries, API friction, and missing tests.
                4. Report findings first, ordered by severity, with exact file paths and function/module locations.
                5. State clearly if no issues are found.
                """
            ),
            validation=("No validation required unless files are changed.",),
            stop_conditions=(
                "The requested review scope is not identifiable.",
                "Required changed files are unavailable.",
            ),
        ),
    }


def render_shape(shape: WorkflowShape, task: str, scope: str, roadmap: str) -> str:
    docs = [doc.format(roadmap=roadmap) for doc in ROOT_DOCS + list(shape.primary_docs)]
    prompt = shape.prompt.format(task=task, scope=scope, roadmap=roadmap).strip()

    lines: list[str] = []
    lines.append(f"# Runenwerk AI Workflow Kickoff: {shape.name}")
    lines.append("")
    lines.append(shape.description)
    lines.append("")
    lines.append("## Primary Docs")
    lines.extend(f"- {doc}" for doc in docs if doc)
    lines.append("")
    lines.append("## First Commands")
    lines.append("- git status --short")
    lines.append("- rg --files <owning-area-or-scope>")
    lines.append("- rg -n \"<key type/function/term>\" <owning-area-or-scope>")
    lines.append("")
    lines.append("## Prompt")
    lines.append("```text")
    lines.append(prompt)
    lines.append("```")
    lines.append("")
    lines.append("## Validation")
    lines.extend(f"- {command}" for command in shape.validation)
    lines.append("")
    lines.append("## Stop Conditions")
    lines.extend(f"- {condition}" for condition in shape.stop_conditions)
    lines.append("")
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    shapes = build_shapes()
    parser = argparse.ArgumentParser(
        description="Generate a Runenwerk AI workflow kickoff prompt and checklist."
    )
    parser.add_argument(
        "shape",
        nargs="?",
        choices=sorted(shapes),
        help="Workflow shape to generate. Use --list to show available shapes.",
    )
    parser.add_argument(
        "--task",
        default="<task>",
        help="Task or milestone text to place in the generated prompt.",
    )
    parser.add_argument(
        "--scope",
        default="<crate/files/subsystem>",
        help="Scope text to place in the generated prompt.",
    )
    parser.add_argument(
        "--roadmap",
        default="<owning roadmap/design path>",
        help="Owning roadmap/design path for milestone and phase-closeout tasks.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List available workflow shapes.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    shapes = build_shapes()

    if args.list or args.shape is None:
        print("Available Runenwerk AI workflow shapes:")
        for name in sorted(shapes):
            print(f"- {name}: {shapes[name].description}")
        if args.shape is None and not args.list:
            print("")
            print("Pass one shape to generate a prompt, for example:")
            print('  python3 tools/workflow/ai_task.py implementation --task "<task>" --scope "<scope>"')
        return 0

    print(render_shape(shapes[args.shape], args.task, args.scope, args.roadmap))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
