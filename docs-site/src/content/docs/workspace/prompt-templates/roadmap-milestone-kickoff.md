---
title: Roadmap Milestone Kickoff Prompt
description: Prompt template for starting a bounded Runenwerk roadmap milestone implementation with Codex, including optional subagents and git worktrees.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-05
related_docs:
  - ../agents.md
  - ./implementation-batch.md
  - ../routines/crate-implementation-routine.md
  - ../routines/phase-completion-drift-check-routine.md
---

# Roadmap Milestone Kickoff Prompt

Use this template when starting a roadmap milestone or sub-milestone in a fresh Codex thread.

This prompt explicitly authorizes Codex to use parallel codebase exploration and bounded subagents when the environment supports them. It does not authorize unbounded implementation, destructive git operations, or broad future-phase work.

## Template

```text
Implement this Runenwerk roadmap milestone:

Milestone:
- <for example: M1 - Editor Structural Core Closed>

Owning roadmap/design:
- <absolute or repo-relative path to roadmap/design>

Goal:
- Implement the milestone end to end, including code, tests, and required docs updates.
- Do not implement later milestones except for minimal seams required by this milestone.

Required setup:
1. Read AGENTS.md first, then AI_GUIDE.md, ARCHITECTURE.md, DEPENDENCY_RULES.md, DOMAIN_MAP.md, GLOSSARY.md, TESTING.md.
2. Read the owning roadmap/design section for this milestone.
3. Inspect current repo truth before editing. Use `rg`/`rg --files` first.
4. Check `git status --short` before editing. Preserve unrelated user changes.

Codex execution guidance:
1. You are explicitly authorized to use subagents for parallel codebase exploration, focused reviews, or disjoint implementation tasks.
2. Use subagents only for bounded work with clear ownership. Tell each worker they are not alone in the codebase and must not revert unrelated edits.
3. Prefer local work for the immediate critical path. Delegate sidecar exploration or disjoint file/module changes.
4. If a separate git worktree is useful and safe, create a `codex/<short-milestone-name>` branch/worktree after checking existing git state. Do not use destructive git commands.
5. Do not commit, push, or open a PR unless explicitly asked.

Implementation requirements:
1. Respect domain boundaries and dependency direction.
2. Use existing helpers, ids, ratification, diagnostics, command, schema, and projection patterns before adding new abstractions.
3. Keep modules organized by subdomain responsibility with `mod.rs` boundaries.
4. Add or update tests for changed invariants.
5. Update docs when public behavior, architecture, crate inventory, or roadmap status changes.
6. Run focused validation first, then broader validation when the milestone changes workspace behavior.

Validation expectations:
- Run the smallest relevant tests for changed crates.
- Run `python3 tools/docs/validate_docs.py` when docs change.
- Run `cargo fmt --all -- --check`.
- Run broader checks from TESTING.md when code paths or workspace behavior changed.
- Use `./quiet_full_gate.sh` for milestone closeout when appropriate.

Stop and report instead of continuing when:
- the milestone requires a dependency direction violation;
- an owning domain is unclear;
- validation fails for an unrelated dirty-worktree reason;
- the task expands into later milestones;
- a required design or contract is missing.

Final response:
1. Start with what changed.
2. List exact files and modules/functions changed.
3. Explain why the changes belong in those domains.
4. Report validation commands and results.
5. Name remaining risks, blockers, and the next milestone.
6. If a phase is complete, run the phase completion drift-check routine before moving on.
```

## Recommended First Use

For the current editor roadmap, use this template with:

```text
Milestone:
- M1 - Editor Structural Core Closed

Owning roadmap/design:
- docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md
```

M1 through M3 are the best implementation starting point because they are implementation-ready against current repo truth. M6 and later should be started only by sub-milestone after their owning contracts exist.
