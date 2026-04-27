#!/usr/bin/env bash
set -euo pipefail

mkdir -p docs-site/src/content/docs/workspace/prompt-templates
mkdir -p docs-site/src/content/docs/workspace/routines

cat > docs-site/src/content/docs/workspace/prompt-templates/README.md <<'MD'
---
title: Prompt Templates
description: Reusable Codex and AI-agent prompt templates for Runenwerk repository work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
---

# Prompt Templates

This folder contains reusable prompts for Codex and AI-assisted repository work.

Prompt templates are documentation artifacts. They do not define runtime behavior, domain invariants, foundation APIs, or AI integration code.

Use these templates when a task benefits from a repeatable instruction shape but still needs repository inspection before editing.

## Available Templates

- [Architecture Audit](./architecture-audit.md)
- [Code Review](./code-review.md)
- [Commit Organization](./commit-organization.md)
- [Crate Design](./crate-design.md)
- [Documentation Refactor](./docs-refactor.md)
- [Implementation Batch](./implementation-batch.md)

## Rules

- Treat templates as starting points, not automatic authority.
- Inspect relevant files before changing code.
- Name exact files and functions/modules for requested changes.
- Run the smallest relevant validation commands.
- Stop when validation fails and report the concrete failure.
- Do not use templates to bypass domain ownership, ratification, diagnostics, or dependency rules.

## Related Docs

- [`../agents.md`](../agents.md)
- [`../routines/README.md`](../routines/README.md)
- [`../documentation-structure.md`](../documentation-structure.md)
MD

cat > docs-site/src/content/docs/workspace/prompt-templates/architecture-audit.md <<'MD'
---
title: Architecture Audit Prompt
description: Prompt template for auditing Runenwerk architecture, boundaries, and refactor priorities.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../../guidelines/architecture.md
  - ../../guidelines/runenwerk-architecture.md
---

# Architecture Audit Prompt

Use this template when asking Codex or another AI agent to audit architecture.

## Template

```text
Audit the current Runenwerk repository architecture for the following scope:

Scope:
- <crate/domain/subsystem/files>

Before giving recommendations:
1. Inspect the relevant files first.
2. Do not treat docs as truth when code contradicts docs.
3. Identify current ownership boundaries.
4. Identify dependency direction and layering.
5. Identify public API and usage ergonomics issues.
6. Identify documentation drift caused by the current code.
7. Do not guess. If evidence is missing, say exactly what is missing.

Output:
1. Findings ordered by severity.
2. Exact file paths and function/module locations.
3. Boundary or doctrine violated, if any.
4. Recommended fixes in priority order.
5. Validation commands to run.
6. Suggested commit split.

Do not implement changes unless explicitly asked.
```

## Expected Agent Behavior

The agent should produce findings first, not a speculative redesign.

Recommend cleanup before redesign unless there is clear semantic inconsistency, duplicated architectural logic, repeated public API friction, or unclear ownership that cannot be solved locally.
MD

cat > docs-site/src/content/docs/workspace/prompt-templates/code-review.md <<'MD'
---
title: Code Review Prompt
description: Prompt template for focused Runenwerk code review.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../routines/public-api-review-routine.md
---

# Code Review Prompt

Use this template for focused code review.

## Template

```text
Review the current changes for:

Scope:
- <files/crates/subsystems>

Review requirements:
1. Inspect the actual changed files.
2. Prioritize correctness, regressions, ownership boundaries, API friction, and missing tests.
3. Do not rewrite unless explicitly asked.
4. Do not guess about intent; infer only from code, docs, and tests.
5. Mention if no issues are found.

Output:
1. Findings first, ordered by severity.
2. Exact file path and function/module location for each finding.
3. Why it matters.
4. Minimal recommended fix.
5. Tests or validation missing.
6. Safe commit split if relevant.
```

## Severity Guide

- Critical: broken invariant, data loss, unsoundness, wrong dependency direction.
- High: likely runtime bug, broken public API, missing validation on authoritative state.
- Medium: unclear ownership, duplicated logic, brittle design, insufficient tests.
- Low: naming, discoverability, docs drift, minor ergonomics.
MD

cat > docs-site/src/content/docs/workspace/prompt-templates/commit-organization.md <<'MD'
---
title: Commit Organization Prompt
description: Prompt template for splitting Runenwerk working-tree changes into coherent commits.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../routines/commit-splitting-routine.md
---

# Commit Organization Prompt

Use this template when the working tree contains mixed changes.

## Template

```text
Organize the current working tree into clean commits.

Before recommending commands:
1. Inspect git status, diff stat, name-status, and relevant manifest diffs.
2. Identify unrelated changes.
3. Group changes by domain and architectural ownership.
4. Do not stage or commit everything together unless the tree is truly one coherent change.
5. Protect unrelated work from being reverted or lost.

Output:
1. Proposed commit order.
2. Files included in each commit.
3. Files explicitly excluded from each commit.
4. Exact git add commands.
5. Validation commands before each commit.
6. Commit messages.
7. Final post-commit status check.

Never use destructive git commands.
```

## Minimum Evidence

The agent should inspect:

```text
git status --short
git diff --find-renames --stat
git diff --find-renames --name-status
git diff --summary
git diff --cached --find-renames --name-status
```
MD

cat > docs-site/src/content/docs/workspace/prompt-templates/crate-design.md <<'MD'
---
title: Crate Design Prompt
description: Prompt template for designing or revising Runenwerk crate architecture.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../../guidelines/architecture.md
  - ../../guidelines/runenwerk-architecture.md
---

# Crate Design Prompt

Use this template before adding a new crate or redesigning an existing crate boundary.

## Template

```text
Design or revise the Runenwerk crate:

Crate:
- <crate path/name>

Goal:
- <goal>

Constraints:
- Respect foundation -> domain -> engine/runtime -> apps/adapters/tools.
- Do not put runtime/app/editor/backend concerns into foundation.
- Do not put concrete runtime glue into pure domain crates.
- Prefer small, reusable contracts over god abstractions.
- Public APIs should be discoverable and easy to use correctly.

Required process:
1. Inspect existing crates/docs that may already own the concept.
2. Challenge whether a new crate is needed.
3. Compare alternatives:
   - no new crate
   - smaller module
   - new crate
   - split crate
   - move to engine/runtime
4. Pick the best long-term boundary.
5. Define purpose, scope, non-scope, dependencies, invariants, public API shape, validation, and docs impact.

Output:
1. Boundary decision.
2. Alternatives rejected and why.
3. Proposed module/file structure.
4. Public API shape.
5. Dependency rules.
6. Test plan.
7. Documentation updates.
8. Phased implementation plan.

Do not implement unless explicitly asked.
```
MD

cat > docs-site/src/content/docs/workspace/prompt-templates/docs-refactor.md <<'MD'
---
title: Documentation Refactor Prompt
description: Prompt template for documentation restructuring work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../documentation-structure.md
  - ../routines/docs-refactor-routine.md
---

# Documentation Refactor Prompt

Use this template when moving, renaming, pruning, or reorganizing documentation.

## Template

```text
Refactor the Runenwerk documentation for this scope:

Scope:
- <docs area/files>

Requirements:
1. Inspect current docs structure and links first.
2. Follow docs-site frontmatter, status, lifecycle, and filename conventions.
3. Do not treat outdated docs as truth when code contradicts docs.
4. Do not rename or move files only for aesthetics.
5. Preserve source-of-truth clarity.
6. Update indexes and internal links.
7. Run documentation validation.

Output:
1. Current structure findings.
2. Proposed moves/renames/prunes.
3. Exact files to change.
4. Link updates needed.
5. Validation commands.
6. Suggested commit split.
```
MD

cat > docs-site/src/content/docs/workspace/prompt-templates/implementation-batch.md <<'MD'
---
title: Implementation Batch Prompt
description: Prompt template for bounded implementation work in Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../routines/code-refactor-routine.md
  - ../routines/crate-implementation-routine.md
---

# Implementation Batch Prompt

Use this template for a bounded implementation task.

## Template

```text
Implement this Runenwerk change:

Task:
- <task>

Scope:
- <crate/files/subsystem>

Requirements:
1. Inspect existing code before editing.
2. Reuse existing abstractions where appropriate.
3. Preserve domain boundaries and dependency direction.
4. Implement the smallest coherent change.
5. Add or update tests for changed invariants.
6. Update docs when public behavior, architecture, routines, or usage changes.
7. Run the smallest relevant validation commands.

Output after implementation:
1. What changed.
2. Files and exact functions/modules changed.
3. Why the change belongs there.
4. Tests/validation run.
5. Remaining risks or follow-up tasks.
```

## Stop Conditions

Stop and report instead of continuing when:

- ownership is unclear;
- a required dependency would violate layer direction;
- validation fails for a reason unrelated to the change;
- the task expands beyond the requested scope.
MD

cat > docs-site/src/content/docs/workspace/routines/code-refactor-routine.md <<'MD'
---
title: Code Refactor Routine
description: Bounded routine for safe code refactors across Runenwerk crates and subsystems.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
  - ../../guidelines/architecture.md
---

# Code Refactor Routine

## Purpose

Use this routine for code refactors that preserve behavior while improving structure, names, boundaries, or public API ergonomics.

## Preconditions

Before editing:

1. Identify the owning crate and subsystem.
2. Inspect nearby modules and tests.
3. Identify public API impact.
4. Identify docs impact.
5. Identify validation commands.
6. Avoid broad speculative reshuffles.

## Routine

1. Capture current state:
   - `git status --short`
   - relevant `git diff -- <path>`
2. Inspect current implementation and call sites.
3. Define the smallest coherent refactor.
4. Apply the refactor.
5. Run formatting.
6. Run focused tests.
7. Run broader checks only when the refactor crosses crate boundaries.
8. Update docs when public behavior, ownership, or usage changes.
9. Report changed files, functions/modules, validation, and follow-up risks.

## Required Validation

Use the smallest relevant set:

```text
cargo fmt --all -- --check
cargo test -p <crate>
cargo check --workspace
```

## Stop Conditions

Stop and report when:

- ownership is unclear;
- a dependency direction violation would be required;
- behavior changes are needed but not requested;
- validation failure is outside the refactor scope;
- the refactor expands into unrelated domains.

## Final Report

Include:

- changed files;
- exact functions/modules changed;
- behavior preserved or intentionally changed;
- validation commands run;
- remaining risks.
MD

cat > docs-site/src/content/docs/workspace/routines/crate-implementation-routine.md <<'MD'
---
title: Crate Implementation Routine
description: Bounded routine for implementing new Runenwerk crates or major crate phases.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../../guidelines/architecture.md
  - ../prompt-templates/crate-design.md
---

# Crate Implementation Routine

## Purpose

Use this routine when implementing a new crate or a major crate phase from an accepted design.

## Preconditions

Before implementation:

1. Confirm the crate boundary is already designed.
2. Confirm the crate belongs in the selected layer.
3. Confirm dependencies obey repository direction.
4. Confirm workspace membership and crate inventory impact.
5. Identify tests and docs required for the public surface.

## Routine

1. Read the crate design or current crate docs.
2. Inspect adjacent crates for naming, module, and API patterns.
3. Create or update `Cargo.toml`.
4. Add module skeletons by responsibility, not technical layer.
5. Implement public vocabulary first.
6. Implement validation, ratification, or reporting logic second when relevant.
7. Add tests for invariants and serialization when relevant.
8. Update root and docs-site crate maps when workspace membership changes.
9. Run focused tests.
10. Run workspace checks when dependencies changed.

## Required Validation

```text
cargo fmt --all -- --check
cargo test -p <crate>
cargo check --workspace
python3 tools/docs/validate_docs.py
```

## Stop Conditions

Stop and report when:

- the crate boundary is not justified;
- implementation requires a forbidden dependency;
- the design contradicts current code;
- the crate becomes a god abstraction;
- public API is technically correct but awkward to discover.

## Final Report

Include:

- crate path;
- public API surface;
- dependency decisions;
- tests added;
- docs updated;
- remaining phase work.
MD

cat > docs-site/src/content/docs/workspace/routines/public-api-review-routine.md <<'MD'
---
title: Public API Review Routine
description: Bounded routine for reviewing discoverability and usability of Runenwerk public APIs.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../agents.md
---

# Public API Review Routine

## Purpose

Use this routine when reviewing whether a public crate or module is easy to discover, use, and compose correctly.

## Preconditions

Before reviewing:

1. Identify the public entry points.
2. Inspect `lib.rs`, `README.md`, usage docs, examples, and tests.
3. Identify the common happy path.
4. Identify advanced or uncommon APIs.
5. Identify whether a prelude exists or should exist.

## Routine

1. Review exports from `lib.rs`.
2. Review module names and type names.
3. Review constructors and normal workflow ergonomics.
4. Review error/diagnostic discoverability.
5. Review examples and usage guides.
6. Compare docs with tests.
7. Identify friction that would make correct usage hard on first read.
8. Recommend small API or docs improvements before larger redesigns.

## Findings Format

For each finding:

```text
Severity:
File:
Function/module:
Issue:
Why it matters:
Recommended change:
Validation:
```

## Stop Conditions

Stop and report when:

- changing the public API would require a design decision;
- compatibility impact is unclear;
- docs and implementation disagree and ownership is unclear.
MD

cat > docs-site/src/content/docs/workspace/routines/commit-splitting-routine.md <<'MD'
---
title: Commit Splitting Routine
description: Bounded routine for organizing mixed Runenwerk working-tree changes into coherent commits.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../prompt-templates/commit-organization.md
---

# Commit Splitting Routine

## Purpose

Use this routine when the working tree contains mixed docs, code, crate, manifest, or tooling changes.

## Preconditions

Before staging:

1. Do not commit blindly.
2. Preserve all unrelated working-tree changes.
3. Inspect status and diffs.
4. Identify shared manifest files.
5. Identify generated files that must not be staged.

## Routine

1. Capture:
   - `git status --short`
   - `git diff --find-renames --stat`
   - `git diff --find-renames --name-status`
   - `git diff --summary`
2. Group files by architectural ownership.
3. Identify files that require patch staging.
4. Stage one coherent commit at a time.
5. Verify staged scope:
   - `git diff --cached --find-renames --stat`
   - `git diff --cached --find-renames --name-status`
6. Run validation for that commit.
7. Commit.
8. Repeat until the tree is clean or intentionally left with known work.

## Safety Rules

- `git restore --staged .` is allowed to clear the index.
- Do not use destructive git commands.
- Do not stage generated context exports.
- Do not combine unrelated domains for convenience.
- Do not hide failing validation.

## Final Report

Include:

- commit list;
- files included per commit;
- validation per commit;
- remaining working-tree state.
MD

cat > docs-site/src/content/docs/workspace/routines/README.md <<'MD'
---
title: Workspace Routines
description: Repeatable maintenance routines for Runenwerk documentation, refactors, and contributor workflows.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
---

# Workspace Routines

This folder contains bounded maintenance routines for humans and AI coding agents.

Use routines when a task needs repeated inspection, patching, validation, and repair.

## Available Routines

- [Code Refactor Routine](./code-refactor-routine.md)
- [Commit Splitting Routine](./commit-splitting-routine.md)
- [Crate Implementation Routine](./crate-implementation-routine.md)
- [Documentation Refactor Routine](./docs-refactor-routine.md)
- [Public API Review Routine](./public-api-review-routine.md)

## Routine Rules

- Routines are bounded.
- Routines must have explicit stop conditions.
- Routines must identify validation commands.
- Routines must not use unbounded loops.
- Routines must preserve unrelated work.
- Routines must report what was changed, skipped, blocked, or left for follow-up.

## Related Docs

- [`../prompt-templates/README.md`](../prompt-templates/README.md)
- [`../agents.md`](../agents.md)
- [`../documentation-structure.md`](../documentation-structure.md)
MD

python3 - <<'PY'
from pathlib import Path

path = Path("docs-site/src/content/docs/workspace/overview.md")
text = path.read_text()

addition = (
    "- Workspace routines: [`routines/README.md`](routines/README.md)\n"
    "- Prompt templates: [`prompt-templates/README.md`](prompt-templates/README.md)\n"
)

if addition not in text:
    anchor = "- Workspace agent rules: [`agents.md`](agents.md)\n"
    if anchor not in text:
        raise SystemExit("Start Here anchor not found in workspace overview")
    text = text.replace(anchor, anchor + addition, 1)

path.write_text(text)
print("Updated workspace overview links.")
PY

python3 - <<'PY'
from pathlib import Path

root = Path("AGENTS.md")
docs = Path("docs-site/src/content/docs/workspace/agents.md")

root_text = root.read_text()

old = """For documentation moves, renames, pruning, or restructuring, follow `docs-site/src/content/docs/workspace/routines/docs-refactor-routine.md`.

For documentation-only validation, run:
"""

new = """For bounded repository workflows, follow `docs-site/src/content/docs/workspace/routines/README.md`.

For reusable Codex and AI-agent prompts, use `docs-site/src/content/docs/workspace/prompt-templates/README.md`.

For documentation moves, renames, pruning, or restructuring, follow `docs-site/src/content/docs/workspace/routines/docs-refactor-routine.md`.

For documentation-only validation, run:
"""

if new not in root_text:
    if old not in root_text:
        raise SystemExit("AGENTS.md routine anchor not found")
    root_text = root_text.replace(old, new, 1)
    root.write_text(root_text)

docs_text = docs.read_text()
if not docs_text.startswith("---\n"):
    raise SystemExit("workspace agents doc is missing frontmatter")

frontmatter, _body = docs_text.split("\n---\n", 1)
docs.write_text(frontmatter + "\n---\n" + root.read_text())

print("Updated AGENTS.md and synced workspace agents copy.")
PY

python3 tools/docs/validate_docs.py

echo
echo "Changed files:"
git status --short

echo
echo "Diff stat:"
git diff --stat

echo
echo "Name status:"
git diff --name-status
