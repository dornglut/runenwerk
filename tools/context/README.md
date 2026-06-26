# Context Export

`export_repo_context.py` exports selected repository files into one line-numbered text file for AI or manual review.

The default export is intentionally small:

```bash
python3 tools/context/export_repo_context.py
```

This uses the `ai-core` profile. It is meant for normal GPT startup and includes current authority files, not the full repository.

## Profiles

List available profiles:

```bash
python3 tools/context/export_repo_context.py --list-profiles
```

Use a profile:

```bash
python3 tools/context/export_repo_context.py --profile ui-current
python3 tools/context/export_repo_context.py --profile ui-component-platform --output ui-component-platform-context.txt
python3 tools/context/export_repo_context.py --profile full-audit --warn-only
```

Profiles live in:

```text
tools/context/profiles/
```

## Profile intent

```text
ai-core
  Small current authority context for most AI work.

workspace-planning
  Workspace planning, routines, and current roadmap context.

ui-current
  Current UI authority and active UI design context.

ui-component-platform
  UI Component Platform implementation context.

full-audit
  Large full-repository audit context. Use only when historical or broad review is required.
```

## Budgets

Budget options prevent accidentally creating a huge context file:

```bash
python3 tools/context/export_repo_context.py --profile ui-current --max-files 120 --max-bytes 1500000
```

By default, a budget breach fails the export. To write anyway:

```bash
python3 tools/context/export_repo_context.py --profile full-audit --max-bytes 3000000 --warn-only
```

## Manifest

Every generated context file starts with a manifest that records:

```text
profile name
description
root path
included file count
total source bytes
include globs
exclude globs
warnings
```

This makes it clear whether a new AI thread is seeing a small authority context, a current UI context, or a full audit dump.

## Rule

Use the smallest profile that can answer the task.

Do not use `full-audit` as the default. It is intentionally large and may include historical docs that are not current authority.
