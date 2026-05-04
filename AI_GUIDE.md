# AI Guide

## Purpose

This document tells AI-assisted contributors how to work safely in Runenwerk. It is not an AI subsystem.

## Core Rule

AI must use the same contracts as humans, tests, editor tools, and scripts. There is no privileged AI mutation path.

## Before Changing Code

For documentation placement, lifecycle states, frontmatter, and naming rules, follow `docs-site/src/content/docs/workspace/documentation-structure.md`.

Read the relevant root docs:

```text
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
GLOSSARY.md
TESTING.md
```

For architectural changes, update or add a design doc under `docs-site/src/content/docs/design/`. For long-term decisions, update or add an ADR under `docs-site/src/content/docs/adr/`.

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
