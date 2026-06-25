# AI Guide

## Purpose

This document tells AI-assisted contributors how to work safely in Runenwerk. It is not an AI subsystem and it does not define runtime AI behavior.

## Core Rule

AI must use the same repository contracts as humans, tests, editor tools, and scripts. There is no privileged AI mutation path.

Runenwerk AI workflow is **scriptless by default**. The normal path must work through GitHub connector, ChatGPT context tooling, Codex patching, manual file browsing, or a local checkout.

## Start Here

For any non-trivial task, read:

```text
docs-site/src/content/docs/workspace/start-here.md
```

Then use the matching routine from:

```text
docs-site/src/content/docs/workspace/routines/README.md
```

For copy-paste task prompts, use:

```text
docs-site/src/content/docs/workspace/task-cards/README.md
```

Do not require `task`, `python`, shell scripts, generated prompts, or rendered planning output to understand or complete a workflow. Those tools are optional local helpers only.

## Before Changing Code

Read the relevant root docs:

```text
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
GLOSSARY.md
TESTING.md
```

Then read the authority files named by the selected routine.

For architectural changes, update or add a design doc under `docs-site/src/content/docs/design/`. For durable decisions, update or add an ADR under `docs-site/src/content/docs/adr/`.

For changes that may affect dependency direction, domain ownership, durable decision history, migration strategy, tradeoffs, enforcement, or ownership mode, use:

```text
docs-site/src/content/docs/workspace/routines/architecture-governance-review-routine.md
```

## Connector and Context-Tool Mode

When command execution is unavailable:

- Inspect files by exact path.
- Name the authority files used as evidence.
- Patch files directly.
- Do not claim command validation was run.
- Use the routine's manual validation checklist.
- Report missing validation honestly.
- Stop when required authority cannot be inspected.

Use the GitHub connector task card when preparing work for a file-by-file agent:

```text
docs-site/src/content/docs/workspace/task-cards/github-connector-task.md
```

Use the Codex task card when preparing work for a patch agent:

```text
docs-site/src/content/docs/workspace/task-cards/codex-task.md
```

## Automation Boundary

AI prompts, workflow templates, checklists, task cards, and optional scripts may help prepare work. They must not bypass repository inspection, accepted ADR/design gates, domain ownership, dependency rules, validation, or closeout evidence.

Keep AI prompts, task cards, and workflow guidance in workspace docs or `tools/`. Runtime AI integrations belong in `apps/`, `tools/`, or `adapters/`. Do not add LLM clients, prompts, or autonomous agents to `foundation` or `domain` crates.

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
Graph substrate                  -> domain/graph
Concrete schemas                 -> owning domain
Editor workspace concepts        -> domain/editor/editor_shell
UI surface mounting concepts     -> domain/ui/ui_surface
UI story proof orchestration     -> domain/ui/ui_story
UI story concrete evidence       -> owning app/editor runtime
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
- Do not make scripts or generated views the required source of workflow truth.

## Patch Expectations

Significant patches should state:

- changed files;
- exact functions, methods, modules, routines, or document sections changed;
- reason for the change;
- affected domains;
- expected invariant impact;
- tests or manual validation performed;
- documentation impact;
- unresolved validation gaps.

After completing phased work, use the phase closeout routine before starting the next phase:

```text
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
```

## Validation

Use `TESTING.md`. If validation cannot be run, say so explicitly and provide the manual evidence checklist used.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:root-note -->
## UI Component Platform note

The current UI Component Platform activation is `PT-UI-COMPONENT-PLATFORM`, starting after `PM-UI-STORY-004`. It defines reusable, story-proven `ControlPackage` and surface kernels before product-specific Gallery, Workbench, Designer, game HUD, or world-space UI behavior. See `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md` and the `ui-component-platform-*-design.md` active design docs.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:root-note -->
