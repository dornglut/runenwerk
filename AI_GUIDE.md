# AI Guide

## Purpose

This document tells AI-assisted contributors how to work safely in Runenwerk. It is not an AI subsystem.

## Core Rule

AI must use the same contracts as humans, tests, editor tools, and scripts. There is no privileged AI mutation path.

## Before Changing Code

For documentation placement, lifecycle states, frontmatter, and naming rules, follow `docs-site/src/content/docs/workspace/documentation-structure.md`.

For planning, implementation, routine selection, and closeout shape, follow `docs-site/src/content/docs/workspace/planning-and-implementation-workflow.md`.

Read the relevant root docs:

```text
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
GLOSSARY.md
TESTING.md
```

For architectural changes, update or add a design doc under `docs-site/src/content/docs/design/`. For long-term decisions, update or add an ADR under `docs-site/src/content/docs/adr/`.

For changes that may affect dependency direction, domain ownership, durable
decision history, migration strategy, tradeoffs, enforcement, or ownership
mode, run the architecture governance workflow before implementation:

```text
python3 tools/workflow/ai_task.py architecture-governance --task "<task>" --scope "<scope>"
```

Where shell support is available, the stable wrapper is:

```text
./workflow architecture-governance --task "<task>" --scope "<scope>"
```

## Automation Boundary

Codex and AI workflow automation may generate prompts, checklists, first
commands, validation expectations, stop conditions, and closeout reminders. It
must not create a privileged AI mutation path or bypass repository inspection,
accepted ADR/design gates, domain ownership, dependency rules, or validation.

Keep AI prompts, workflow templates, and checklist automation in workspace docs
or `tools/`. Runtime AI integrations belong in `apps/`, `tools/`, or
`adapters/`. Do not add LLM clients, prompts, or autonomous agents to
`foundation` or `domain` crates.

## Where Concepts Belong

```text
Typed identity vocabulary        -> foundation/id or owning domain
Typed ID macro support           -> foundation/id_macros
Diagnostics vocabulary           -> foundation/diagnostics
Domain diagnostic codes          -> owning domain
Ratification vocabulary          -> foundation/ratification
Ratification reports             -> foundation/ratification plus owning domain issue codes
Domain validation rules          -> owning domain ratifier
Command vocabulary               -> foundation/commands
Concrete command enums           -> owning domain
Schema vocabulary                -> foundation/schema or owning domain
Graph substrate                    -> domain/graph
Concrete schemas                 -> owning domain
Editor workspace concepts        -> domain/editor/editor_shell
UI surface mounting concepts     -> domain/ui/ui_surface
Runtime scheduling               -> engine/src/runtime plus domain/scheduler
Backend-specific details         -> backend adapter
AI integrations                  -> apps/tools/adapters
```

## Do Not Do

- Do not add runtime dependencies to foundation crates.
- Do not put editor-only concepts in runtime APIs.
- Do not mutate domain state directly when a command boundary exists.
- Do not introduce raw strings or integers where typed IDs exist.
- Do not add fake capability tokens that are never enforced.
- Do not add LLM clients, prompts, or AI agents to foundation/domain crates.
- Do not make projections authoritative state unless explicitly designed.
- Do not create universal god abstractions such as `EngineObject`, `UniversalCommand`, or `GlobalRegistry`.

## Patch Expectations

Significant patches should state changed files, reason, affected domains, expected invariant impact, tests to run, and documentation impact.

After completing a phased implementation, run the phase completion drift-check routine before starting the next phase.

## Validation

Use `TESTING.md`. If validation cannot be run, say so explicitly.
