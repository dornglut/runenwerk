---
title: Foundation Vocabulary Crates
description: Design direction for Runenwerk foundation vocabulary crates and their implementation order.
status: implemented
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-16
related_adrs: []
---

# Foundation Vocabulary Crates

## Purpose

This document records the intended foundation vocabulary crates for Runenwerk.

A vocabulary crate defines shared language and stable contract types. It does not own domain behavior, runtime execution, editor policy, backend integration, AI behavior, or concrete domain invariants.

The goal is to keep foundational concepts reusable without turning foundation into a global framework.

## Architectural Rule

Foundation crates may define common vocabulary.

Owning domains still define and enforce their own rules.

```text
Foundation defines words.
Domains define meaning.
Runtime composes execution.
Apps and tools orchestrate workflows.
```

## Final Foundation Vocabulary Set

Immediate and active:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
```

Later candidates:

```text
foundation/capabilities
foundation/provenance
```

Not planned now:

```text
foundation/time
foundation/units
foundation/reflection
foundation/inspection
```

## Crate Decisions

### `foundation/id`

Owns typed identity primitives and allocation vocabulary.

It must not own domain registries, ECS-specific APIs, database adapters, editor identity policy, or global object identity.

### `foundation/id_macros`

Owns proc-macro support for typed ID wrappers.

It remains separate from `foundation/id` because proc-macro crates have different compilation and dependency constraints.

### `foundation/diagnostics`

Owns structured diagnostic reporting vocabulary.

Diagnostics explain issues. They do not decide acceptance, execute commands, mutate state, or own domain validation rules.

### `foundation/ratification`

Owns reusable candidate acceptance-report vocabulary.

Ratification answers whether a candidate was accepted, accepted with warnings, rejected, or fatally invalid.

It must not own editor history, undo/redo, reconciliation, command execution, or concrete domain invariants.

### `foundation/schema`

Owns schema and descriptor vocabulary for describing data shapes.

Schema describes values, fields, parameters, component surfaces, command parameters, asset structures, and tool-facing descriptors.

It must not own concrete schemas. Concrete schemas are published by the owning domain.

Current status:

```text
Phase 0 design accepted
Phase 1 core vocabulary implemented
Phase 2 optional diagnostics bridge implemented
Phase 3 first low-risk consumer implemented
Phase 4 scene LocalTransform descriptor publication implemented
Phase 5 foundation/commands design preparation completed
```

It still must not own runtime behavior, command execution, editor policy, ECS reflection, registries, domain validation, AI behavior, or generic `SchemaValue`-against-`SchemaShape` validation.

### `foundation/commands`

Owns command descriptor and inert proposal vocabulary.

It defines how command contracts are described and how portable command proposals are represented.

It must not own concrete command enums, command executors, editor command buses, global command registries, undo/redo engines, AI runners, or domain mutation logic.

Current status:

```text
Phase 0 design accepted
Phase 1 core descriptor/proposal vocabulary implemented
Phase 2 optional diagnostics bridge implemented
Phase 3 one domain-owned command descriptor implemented
Phase 4 one explicit proposal-to-domain-intent adapter implemented
Phase 5 ratification/diagnostics integration evaluation completed
```

## Relationship Between Core Vocabulary Crates

```text
commands     = what can be requested
schema       = what shape the request/data has
ratification = whether a candidate/request is accepted
diagnostics  = what explains warnings, errors, and failures
```

Example:

```text
CommandDescriptor:
  scene.rename_entity requires entity_id and new_name

Schema:
  entity_id is an entity identifier
  new_name is a non-empty string

CommandProposal:
  rename entity 12 to Player

Ratification:
  rejected

Diagnostic:
  entity 12 does not exist
```

## AI and Tooling Boundary

AI integrations must not live in foundation or pure domain crates.

AI and editor tooling may consume command descriptors, schemas, ratification reports, and diagnostics.

The expected flow is:

```text
AI/tooling proposes
  -> app/editor router resolves
  -> owning domain command boundary executes
  -> owning domain ratifies
  -> diagnostics explain result
```

The editor may orchestrate command routing, but it must not own every command for ECS, network, replay, UI, scene, or other domains.

## Implementation Order

### Phase 1: Stabilize Current Foundation Crates

Status: complete.

The initial foundation crates were kept focused:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
```

Do not widen them to absorb commands, schemas, reflection, or runtime policy.

### Phase 2: Implement `foundation/ratification`

Status: complete for the current foundation-vocabulary milestone.

The crate now owns shared ratification status, severity, issue, report, ratifier, and optional diagnostics-bridge vocabulary.

Completed first consumers:

```text
domain/ui/ui_surface
domain/editor/editor_shell
```

The remaining `editor_persistence` consumer should be handled when normalized scene validation reporting is actively worked, not as a blocker for schema design.

Do not start by rewriting every validation error in the workspace.

### Phase 3: Diagnostics Completion Gate

Status: complete.

Before schema or command vocabulary work starts, `foundation/diagnostics` and `foundation/ratification` must be warning-free under their relevant feature sets.

Required:

```text
cargo test -p diagnostics
cargo test -p diagnostics --features serde
cargo test -p ratification
cargo test -p ratification --features diagnostics
cargo test -p ratification --features serde,diagnostics
cargo fmt --all -- --check
python3 tools/docs/validate_docs.py
```

Recommended before committing the milestone:

```text
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Not required:

```text
rewrite every domain error as diagnostics
replace EditorMutationError
replace GoverningChangeError
wire diagnostics into editor UI
wire diagnostics into tracing/logging
create a global diagnostic registry
```

### Phase 4: Implement `foundation/schema`

Status: complete for the schema-vocabulary stabilization milestone, with Phase 0 through Phase 4 complete.

Completed:

```text
Phase 0: accepted design
Phase 1: crate skeleton and core vocabulary
Phase 2: optional diagnostics bridge for schema-definition issues
Phase 3: editor_inspector path/value interoperability helpers
Phase 4: scene LocalTransform descriptor publication
```

Schema describes reusable shapes and value contracts. It must not become a serialization framework, ECS reflection system, editor inspector engine, runtime object model, command executor, registry, AI path, or domain validator.

### Phase 5: Design `foundation/commands`

Status: complete.

The accepted command design lives at `docs-site/src/content/docs/design/implemented/foundation-commands-design.md`. Phase 1 and Phase 2 are implemented at `foundation/commands`; Phase 3 and Phase 4 are implemented in `domain/editor/editor_scene`. Phase 5 found no repeated proposal rejection pattern that justifies new shared helper vocabulary.

```text
CommandDescriptor uses schema for parameters.
CommandProposal carries SchemaValue parameters.
```

Do not add command execution, global registries, ratification dependencies, provenance, permission, transaction, or patch semantics before the owning design phase allows them.

### Phase 6: Re-evaluate Later Candidates

Consider `foundation/capabilities` only when there is real enforcement.

Consider `foundation/provenance` only when origin, actor, trust, causality, AI proposal, replay, editor history, and multiplayer concepts start duplicating across domains.

## Non-Goals

This foundation vocabulary set must not introduce:

```text
UniversalCommand
GlobalCommandExecutor
GlobalRegistry
EngineObject
AnyDomainState
AiCommandRunner
EditorCommandBus
reflection-based mutation backdoors
runtime scheduling policy
backend adapters
LLM clients
```

## Acceptance Criteria for New Foundation Vocabulary Crates

A new foundation vocabulary crate is allowed only when all of these are true:

1. At least two independent domains need the same vocabulary.
2. The vocabulary can be defined without depending on domain, runtime, app, adapter, backend, or AI crates.
3. The crate can remain small and non-executing.
4. The owning domains keep their own invariants.
5. The crate improves discoverability and consistency instead of hiding behavior behind generic abstractions.

## Summary

The intended foundation vocabulary set is:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
```

The strongest later candidates are:

```text
foundation/capabilities
foundation/provenance
```

The rejected or deferred candidates are:

```text
foundation/time
foundation/units
foundation/reflection
foundation/inspection
```

Reflection remains important, but it should come after schema, commands, diagnostics, and ratification are stable. It must not become an alternative mutation path around commands and ratification.
