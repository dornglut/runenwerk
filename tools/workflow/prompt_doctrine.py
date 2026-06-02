#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


QUALITY_DOCTRINE_ID = "runenwerk-quality-doctrine-v1"

_DOCTRINE_LINES = (
    f"Quality doctrine: {QUALITY_DOCTRINE_ID}",
    "- Prefer the long-term architecture-correct solution over local patches.",
    "- No shortcuts, no half measures, no success-shaped placeholders.",
    "- Inspect current code truth before editing.",
    "- Do not claim completion while known gaps, drift, missing evidence, or unimplemented target contracts remain.",
    "- If scope cannot satisfy the contract honestly, stop and report the missing authority or design gap.",
    "- `perfectionist_verified` requires zero findings, zero known gaps, zero known risks, and zero truth drift.",
)


REQUIRED_PROMPT_BUILDER_USAGE = {
    "tools/workflow/execution/writers.py": "quality_doctrine_block",
    "tools/workflow/production_goal.py": "quality_doctrine_block",
    "tools/workflow/production_plan.py": "quality_doctrine_block",
    "tools/workflow/ai_task.py": "quality_doctrine_block",
    "tools/workflow/parallel_batch.py": "quality_doctrine_block",
    "tools/workflow/track_control_cli.py": "QUALITY_DOCTRINE_ID",
}

REQUIRED_PROMPT_TEMPLATE_MARKERS = (
    "docs-site/src/content/docs/workspace/prompt-templates/README.md",
    "docs-site/src/content/docs/workspace/prompt-templates/production-implementation-contract.md",
    "docs-site/src/content/docs/workspace/prompt-templates/goal-execution.md",
    "docs-site/src/content/docs/workspace/prompt-templates/implementation-batch.md",
    "docs-site/src/content/docs/workspace/prompt-templates/parallel-roadmap-batch.md",
)


def quality_doctrine_lines() -> list[str]:
    return list(_DOCTRINE_LINES)


def quality_doctrine_block() -> str:
    return "\n".join(_DOCTRINE_LINES)


def audit_prompt_doctrine(*, repo_root: Path | None = None) -> list[str]:
    root = repo_root or Path(__file__).resolve().parents[2]
    errors: list[str] = []
    for relative, required_text in REQUIRED_PROMPT_BUILDER_USAGE.items():
        path = root / relative
        if not path.exists():
            errors.append(f"{relative}: required prompt builder file is missing")
            continue
        text = path.read_text(encoding="utf-8")
        if required_text not in text:
            errors.append(f"{relative}: missing canonical quality doctrine usage {required_text!r}")
    for relative in REQUIRED_PROMPT_TEMPLATE_MARKERS:
        path = root / relative
        if not path.exists():
            errors.append(f"{relative}: required prompt-template doc is missing")
            continue
        text = path.read_text(encoding="utf-8")
        if QUALITY_DOCTRINE_ID not in text:
            errors.append(f"{relative}: missing {QUALITY_DOCTRINE_ID}")
    return errors


def main(argv: list[str] | None = None) -> int:
    args = argv if argv is not None else sys.argv[1:]
    command = args[0] if args else "validate"
    if command != "validate":
        print("usage: prompt_doctrine.py validate")
        return 2
    errors = audit_prompt_doctrine()
    if errors:
        print("prompt doctrine validation failed")
        for error in errors:
            print(f"- {error}")
        return 1
    print(f"prompt doctrine validation passed: {QUALITY_DOCTRINE_ID}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
