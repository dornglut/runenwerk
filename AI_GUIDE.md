# AI Guide

## Purpose

This document tells AI-assisted contributors how to work safely in Runenwerk. It is not an AI subsystem.

## Core Rule

AI must use the same contracts as humans, tests, editor tools, and scripts. There is no privileged AI mutation path.

## Before Changing Code

For documentation placement, lifecycle states, frontmatter, and naming rules, follow `docs-site/src/content/docs/workspace/documentation-structure.md`.

For planning, implementation, routine selection, and closeout shape, follow `docs-site/src/content/docs/workspace/planning-and-implementation-workflow.md`.

For new Codex threads, use the one-line repo workflow commands first:

```text
Run task batch:kickoff -- --next and follow the generated workflow.
Run task roadmap:intake -- --idea "<design/change idea>" and prepare it for roadmap review.
Run task ai:goal -- --track "<PT-ID>" and use the generated /goal coordinator prompt for full production-track work.
```

Use `task batch:kickoff -- --next` for existing roadmap implementation work.
Use `task roadmap:intake` for new designs or change ideas that need roadmap
review before implementation.
Use `task ai:goal -- --track <PT-ID>` only for production-track coordination;
it must still execute one legal milestone or WR slice at a time and must not
bypass production or roadmap gates.

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
task ai:architecture-governance -- --task "<task>" --scope "<scope>"
```

Where shell support is available, the stable wrapper is:

```text
task ai:architecture-governance -- --task "<task>" --scope "<scope>"
```

For approved concurrent roadmap work, the lower-level commands remain:

```text
task batch:kickoff -- --next
task roadmap:validate
task batch:propose -- --goal "<batch goal>" --scope "L0"
task ai:parallel-roadmap-batch -- --task "<batch goal>" --scope "<WR rows or roadmap docs>"
task ai:goal -- --track "<PT-ID>"
```

Roadmap rows are structured in
`docs-site/src/content/docs/workspace/roadmap-items.yaml`. After changing
roadmap evidence or topology, run:

```text
task roadmap:render
task roadmap:check
task puml:validate
```

The batch coordinator must propose work and wait for approval before spawning
or coordinating workers.

## UI Story V2

`domain/ui/ui_story` is a proof/orchestration crate. Its canonical flow is:

```text
Manifest V2 -> Registry V2 -> Workflow Graph -> App-owned Evidence -> Workflow Report -> Mount Decision -> CLI/Gallery
```

`ui_story` must not perform filesystem IO, compiler execution, renderer
execution, static mount execution, or editor/gallery behavior. The
editor/gallery runtime owns concrete app behavior and records evidence into the
V2 workflow. Preview publication is derived from all three facts: the workflow
report outcome, the mount decision, and actual mounted-frame existence.

Old flat-stage APIs are superseded and are not canonical. Do not reintroduce
`UiStoryStageKind`, `UiStoryStageReport`, `UiStoryRunReport`,
`UiStoryMountEligibility`, `required_stages`, or
`run_story_with_stage_reports` as active APIs.

When touching UI Story V2, use the scoped validation gate:

```text
cargo test -p ui_story
cargo test -p runenwerk_editor --bin runenwerk_ui_gallery
cargo test -p runenwerk_editor --bin runenwerk_ui_designer
cargo fmt --all --check
```

Broad `cargo test -p runenwerk_editor` failures in
`RunenwerkEditorShellState::workspace_state`, `replace_workspace_state`, or
`apply_workspace_mutation` are not UI Story proof failures unless the failure
also touches the V2 story proof path.

Do not remove V2 suffixes or add speculative interaction, accessibility,
performance, or visual-diff workflows in polish work. Add new workflow profiles
only when there is a real app/runtime evidence producer.

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

## Patch Expectations

Significant patches should state changed files, reason, affected domains, expected invariant impact, tests to run, and documentation impact.

After completing a phased implementation, run the phase completion drift-check routine before starting the next phase.

## Validation

Use `TESTING.md`. If validation cannot be run, say so explicitly.
