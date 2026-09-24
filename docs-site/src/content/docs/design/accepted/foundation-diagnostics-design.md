---
title: Foundation Diagnostics Design
description: Reusable diagnostic reporting vocabulary for Runenwerk foundation.
status: accepted
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-04-26
related_adrs: []
---

# `foundation/diagnostics` Design

## Purpose

`foundation/diagnostics` exists to provide Runenwerk with a small, reusable vocabulary for structured diagnostic reporting.

The crate standardizes how validation failures, warnings, recoverable issues, fatal issues, contextual notes, and domain-owned observations are represented, transported, aggregated, inspected, serialized, and converted into user/tool-facing reports.

It does **not** decide what is valid in any domain. It defines the reporting protocol that domain crates, engine/runtime crates, apps, adapters, tests, and tooling can share.

The long-term goal is to make failures boring, precise, machine-readable, and human-readable without coupling foundation code to logging backends, editor UI panels, tracing subscribers, or domain-specific validation rules.

The first implementation may be small, but the public type model should be treated as the long-term contract. The crate should avoid temporary shapes that will force downstream migrations once reports become serialized, inspected by tools, converted from ratification reports, or displayed in editor/debug interfaces.

---

## Scope

This crate owns the shared diagnostic reporting vocabulary.

In scope:

- diagnostic severity;
- stable diagnostic codes;
- diagnostic domains;
- structured diagnostic subjects;
- diagnostic subject kinds and subject identifiers;
- diagnostic messages;
- diagnostic notes;
- optional source spans/ranges;
- structured diagnostic metadata;
- related diagnostics;
- diagnostic reports;
- issue aggregation helpers;
- simple diagnostic sinks/emitters;
- conversion boundaries from validation and ratification reports;
- optional serialization support behind a feature;
- lightweight formatting helpers for debug/test output.

The crate should support `no_std`-compatible core value types where practical. Owned messages, collections, metadata maps, related diagnostics, sinks, reports, and serialization may require `alloc` or feature flags.

The crate must be designed so that future optional capabilities can be added without changing the core diagnostic identity model.

---

## Non-Scope

This crate explicitly does not own:

- tracing subscriber setup;
- OpenTelemetry integration;
- profiler integration;
- log file rotation;
- crash reporting;
- panic handling;
- runtime dashboards;
- editor diagnostic panels;
- UI presentation;
- ECS integration;
- asset database integration;
- mandatory global diagnostic registration;
- domain-specific validation rules;
- ratification decision logic;
- editor command governance;
- undo/redo;
- transaction history;
- session sharing;
- reconciliation semantics.

Those concerns belong in engine/runtime crates, apps, adapters, or the owning domain crates.

The crate may remain compatible with optional adapters for those systems, but it must not directly depend on them.

---

## Architectural Position

Layer:

```text
foundation
```

Recommended crate path:

```text
foundation/diagnostics
```

Recommended package name:

```text
diagnostics
```

Allowed dependencies:

```text
core
alloc optional
serde optional, feature-gated
```

Possible local dependencies:

```text
foundation/id only if a concrete shared ID formatting need emerges
```

Forbidden dependencies:

```text
domain/*
engine
apps/*
adapters/*
net/*
editor_core
ui_surface
scene
scheduler
tracing subscriber/logging backends
UI crates
```

Dependency direction must remain downward/stable. Domains consume diagnostics; diagnostics must not know the domains that consume it.

---

## Ownership Rules

`foundation/diagnostics` owns the generic representation of a diagnostic and a diagnostic report.

It creates and maintains:

- the shared `Severity` model;
- the shared `DiagnosticCode` contract;
- the shared `DiagnosticDomain` contract;
- the shared structured `DiagnosticSubject` transport shape;
- the shared `DiagnosticLocation`/source-location model;
- the shared `DiagnosticMetadata` model;
- the shared `Diagnostic`, `DiagnosticReport`, and `DiagnosticSink` containers;
- optional serialization compatibility for those types.

It does not create domain-specific codes. Domain crates create and maintain their own diagnostic code families.

It does not mutate domain state. Diagnostics are observations, not commands.

It does not validate domain state. Domain validators and ratifiers decide what is valid and emit diagnostics or ratification issues.

It is consumed by:

- domain validators;
- domain ratifiers;
- import pipelines;
- editor/runtime adapters;
- test assertions;
- debug tooling;
- future AI-assisted inspection/reporting tools.

A crate may only emit diagnostic codes from code families it owns, or from dependencies it is directly adapting/passing through.

Examples:

```text
ui_surface may emit ui_surface.*
editor_shell may emit editor_shell.* and may pass through ui_surface.*
editor_persistence may emit editor_persistence.* or scene.persistence.* if it owns that boundary
engine adapters may translate lower-level diagnostics but should not invent codes for unrelated domains
```

---

## Public API Policy

Stable public contracts should include:

- `Severity`;
- `DiagnosticCode`;
- `DiagnosticCodeError`;
- `DiagnosticDomain`;
- `DiagnosticDomainError`;
- `DiagnosticSubject`;
- `DiagnosticSubjectKind`;
- `DiagnosticSubjectId`;
- `DiagnosticLocation`;
- `DiagnosticLocationError`;
- `DiagnosticMessage`;
- `DiagnosticNote`;
- `DiagnosticMetadata`;
- `DiagnosticMetadataEntry`;
- `DiagnosticMetadataValue`;
- `DiagnosticRelated`;
- `Diagnostic`;
- `DiagnosticReport`;
- `DiagnosticSink`;
- conversion helpers that do not pull in domain dependencies.

Internal or unstable details should include:

- allocation strategy;
- owned vs borrowed storage details;
- formatting internals;
- optional serde implementation details;
- test helper internals;
- any future registry implementation.

Public API rules:

- Prefer small, explicit types over one large global error hierarchy.
- Prefer stable static diagnostic codes.
- Do not expose a global enum of every domain.
- Do not require global registration in the initial version.
- Keep the first version easy to use from domain crates without macros.
- Do not expose storage details that prevent future serialization.
- Do not make diagnostic subjects display-only strings.
- Keep validating constructors available from the first implementation.

---

## Long-Term Compatibility Requirements

The initial implementation may be intentionally small, but the public model must remain compatible with:

- optional `serde` serialization;
- optional future diagnostic registries;
- structured subjects;
- structured metadata;
- related diagnostics;
- diagnostic aggregation;
- conversion from ratification reports;
- editor/debug/CLI/AI inspection;
- cross-crate stable code ownership;
- persisted reports if they later cross process, tool, or session boundaries.

This means the core diagnostic model must avoid unstable identities such as `TypeId`, `Debug` output, backend-specific handles, raw pointers, or opaque domain objects.

Portable fields should be preferred:

```text
code
domain
severity
subject
message
notes
span
metadata
related
```

A future registry may document known code families, default severity, ownership, descriptions, examples, and remediation hints. The registry must remain optional. Emitting a diagnostic must not require global registration.

---

## Invariants

The following invariants must always hold:

- A diagnostic must have a severity.
- A diagnostic must have a stable machine-readable code.
- A diagnostic must identify an owning domain.
- A diagnostic must include a human-readable message.
- A diagnostic code must be stable once used across crate/tooling boundaries.
- A diagnostic domain must be non-empty.
- A diagnostic code must be non-empty.
- A diagnostic code must follow the documented code format unless explicitly constructed through an internal/test-only unchecked path.
- A diagnostic subject must be structured if present.
- A diagnostic subject must not require parsing a human-readable message.
- A diagnostic must not imply that foundation owns the validity rule that produced it.
- Diagnostic domain identifiers must not require foundation to know all domain crates.
- Fatal diagnostics must be distinguishable from ordinary rejection/errors.
- Reports must preserve all emitted diagnostics unless explicitly filtered by a caller-owned policy.
- Optional serialization must not become required for core usage.
- A `DiagnosticReport` may summarize severity, but must not define acceptance/rejection policy.

---

## Failure Modes

Invalid diagnostic construction should be handled by strongly typed constructors.

Examples:

- malformed diagnostic code;
- empty diagnostic code;
- empty diagnostic domain;
- missing message;
- invalid span/range;
- invalid subject kind;
- invalid subject identifier;
- unsupported owned allocation in `no_std` configurations.

Public constructors must preserve basic invariants from the first version. Fully owned/dynamic parsing may require `alloc`, but empty codes/domains and invalid spans must not be accepted silently.

Unchecked constructors may exist only if they are:

- clearly named;
- documented as requiring caller-side invariant proof;
- restricted to `pub(crate)`, test support, or const/static cases where full runtime validation is impossible.

Boundary violations should be reported by the caller’s domain-specific error model or diagnostics. This crate should avoid panicking except for invariant violations in debug-only helpers or tests.

---

## Diagnostics

This crate owns only diagnostic code families for diagnostics-infrastructure failures.

Recommended family:

```text
foundation.diagnostics.*
```

Possible codes:

```text
foundation.diagnostics.code.empty
foundation.diagnostics.code.invalid_format
foundation.diagnostics.domain.empty
foundation.diagnostics.subject.invalid_kind
foundation.diagnostics.subject.invalid_id
foundation.diagnostics.span.invalid_range
foundation.diagnostics.report.empty_when_error_required
```

This crate does **not** own domain diagnostic families such as:

```text
ui_surface.*
editor_shell.*
scene.*
asset.*
render_graph.*
scheduler.*
```

Those belong to the owning domain crate.

---

## Ratification

This crate does not ratify domain state.

It provides diagnostic vocabulary that ratifiers may use when they report issues.

Relationship to `foundation/ratification`:

```text
foundation/diagnostics
  describes issue reporting vocabulary

foundation/ratification
  describes candidate acceptance report vocabulary
```

`foundation/ratification` may convert ratification issues into diagnostics if the dependency direction remains acceptable. `foundation/diagnostics` should not depend on `foundation/ratification` unless a concrete cycle-free need is proven.

`DiagnosticReport` must not decide acceptance policy. A report may provide helpers such as:

```text
has_errors
has_fatal
highest_severity
count_by_severity
```

But the ratifier or owning domain decides whether those diagnostics imply rejection.

---

## Commands

This crate owns no mutating command boundary.

Diagnostics are output/observation artifacts. They must not mutate domain state, execute commands, repair invalid candidates, or apply migrations.

Command systems may emit diagnostics, but commands belong to domain crates or higher-level governance crates.

---

## Inspection

This crate should expose read-only inspection-friendly DTOs through its own public types.

Inspection consumers should be able to read:

- diagnostic code;
- domain;
- severity;
- subject kind;
- subject identifier;
- subject label;
- message;
- notes;
- span/source information;
- metadata entries;
- related diagnostics;
- report summary counts.

These DTOs are suitable for:

- tests;
- debug overlays;
- editor panels;
- CLI reports;
- future AI-assisted tooling.

Inspection must remain read-only. Diagnostic inspection must not expose mutable internals of producing domains.

Subjects and metadata must be structured enough that tools do not need to parse human-facing messages.

---

## Persistence and Versioning

Diagnostic codes are stable contracts once exposed outside a single crate.

Rules:

- Diagnostic codes should use static string identifiers by default.
- Codes should follow `<domain>.<area>.<condition>`.
- Renaming a public code is a breaking change for tools/tests that consume it.
- Serialized diagnostics are optional and should be feature-gated.
- The core type model must remain serialization-compatible even when `serde` is disabled.
- Serialized diagnostic reports are observation artifacts, not authoritative domain state.
- Persisted reports should include enough information to interpret severity, code, domain, subject, message, notes, span, metadata, and related diagnostics.

Versioning policy:

- A report schema version should be introduced when serialized reports cross process, tool, session, or persistence boundaries.
- Domain-specific diagnostic code versioning remains the responsibility of the owning domain.
- Adding metadata keys is usually non-breaking.
- Removing or renaming public diagnostic codes is breaking.
- Changing severity defaults in a future registry may be breaking if external tools depend on them.

The public type model must avoid requiring future persisted formats to parse display strings for identity.

---

## Extension Points

Intended extension mechanisms:

- domain-defined diagnostic code constants;
- domain-defined subject kinds;
- domain-defined subject identifiers;
- domain-defined subject labels;
- domain-defined metadata keys;
- optional conversion from domain errors into diagnostics;
- optional conversion from ratification reports into diagnostic reports;
- optional sink/collector implementations;
- optional serde support;
- optional test assertion helpers;
- optional future diagnostic registry.

Extension mechanisms that should be avoided initially:

- mandatory global plugin registry;
- mandatory global diagnostic registry;
- proc-macro-heavy diagnostic declaration;
- runtime logging backend integration;
- UI renderer integration.

The type model must not prevent a future optional registry from documenting:

- known codes;
- owning crate/domain;
- default severity;
- descriptions;
- remediation hints;
- deprecation/rename metadata;
- examples.

---

## Testing Strategy

Required tests:

- diagnostic codes preserve exact string identity;
- invalid codes are rejected by validating constructors;
- empty domains are rejected by validating constructors;
- invalid spans are rejected by validating constructors;
- structured subjects preserve kind/id/label;
- metadata preserves key/value identity;
- related diagnostics preserve order and identity;
- severity ordering or classification behaves as documented;
- reports aggregate counts correctly;
- report status helpers are deterministic if included;
- sinks collect diagnostics in emission order;
- optional serde round-trips preserve code/domain/severity/message/subject/metadata/related;
- no domain dependency leaks into the crate;
- no feature unexpectedly pulls engine/app/editor dependencies.

Recommended test categories:

```text
unit tests
invariant tests
serialization feature tests
no_std/alloc feature checks if supported
dependency-boundary checks
```

---

## Negative Doctrine

This crate is not a logger.

This crate is not a tracing subscriber.

This crate is not an observability platform.

This crate is not a profiler.

This crate is not an editor diagnostic UI.

This crate is not a domain validator.

This crate is not a ratifier.

This crate is not a global error hierarchy.

This crate is not a command system.

This crate is not an undo/redo or reconciliation system.

This crate is not a mandatory global registry.

This crate is not a domain-specific error taxonomy.

---

## Current Repo Findings

The current repository already has several local error and reporting patterns:

- `foundation/id` already owns typed identity errors such as invalid raw IDs, allocator exhaustion, unknown slots, and stale generations. This demonstrates that foundation-level crates may expose precise local error types, but not a shared diagnostic protocol yet.
- `editor_core` owns `EditorMutationError` and `GoverningChangeError` as editor-authoring errors. These are editor-specific and should not be moved wholesale into foundation.
- `editor_persistence` has explicit persistence validation errors such as unsupported scene version, duplicate entity IDs, missing parents, and cyclic parent references.
- `ui_surface` has ratification dispatch errors such as missing surface capability and adapter errors.
- `engine` and `scheduler` use tracing/logging for runtime observability. That is not the same problem as structured diagnostics and must remain outside this foundation crate.

The evidence points to a repeated need for structured issue reporting, but not to a need for a foundation-owned global error hierarchy.

---

## Additional Design Decisions

- Diagnostics are report artifacts, not control-flow errors.
- Emission order is preserved.
- Subjects, metadata, and suggestions are structured; display strings are not authoritative.
- `DiagnosticLocation` is generic and not limited to source code.
- Static diagnostic codes are preferred; owned codes exist for deserialization/adapters.
- Metadata keys are owned by emitting domains.
- Related diagnostics are lightweight references/summaries in v1, not recursive full diagnostics.
- Core scalar types may be `no_std`; reports require `alloc`.
- Diagnostic display output is not parseable API.

---

## Primary Type Model

Recommended initial public vocabulary:

```text
Severity
DiagnosticCode
DiagnosticCodeError
DiagnosticDomain
DiagnosticDomainError
DiagnosticSubject
DiagnosticSubjectKind
DiagnosticSubjectId
DiagnosticLocation
DiagnosticLocationError
DiagnosticMessage
DiagnosticNote
DiagnosticMetadata
DiagnosticMetadataEntry
DiagnosticMetadataValue
DiagnosticRelated
Diagnostic
DiagnosticReport
DiagnosticSink
```

### `Severity`

Minimum useful set:

```text
Info
Warning
Error
Fatal
```

Semantics:

- `Info`: relevant observation, not a problem.
- `Warning`: accepted but suspicious or degraded.
- `Error`: rejected or invalid, but process can continue.
- `Fatal`: cannot safely continue the current operation.

`Trace` should not be included initially unless a clear non-logging diagnostic use appears.

### `DiagnosticCode`

Stable machine-readable code.

Expected format:

```text
<domain>.<area>.<condition>
```

Examples:

```text
ui_surface.mount.unknown_host
ui_surface.mount.duplicate_instance
editor_shell.route.stale_projection_epoch
scene.persistence.missing_parent
scene.persistence.cyclic_parent_reference
render_graph.resource.unbound
```

Recommended shape:

```text
DiagnosticCode
```

The implementation may internally optimize static codes, but the public contract should support both static and owned codes when `alloc` is enabled.

Basic validation rules:

```text
non-empty
lowercase domain-style segments preferred
dot-separated
no whitespace
no human sentence text
```

### `DiagnosticDomain`

Identifies the owning conceptual domain.

Recommended shape:

```text
DiagnosticDomain
```

Examples:

```text
foundation.id
foundation.diagnostics
ui_surface
editor_shell
editor_persistence
scene
render_graph
scheduler
engine.net
```

Do not make this a global enum in foundation.

### `DiagnosticSubject`

Identifies the affected subject.

The subject must be domain-agnostic but structured.

Recommended conceptual shape:

```text
kind: DiagnosticSubjectKind
id: Option<DiagnosticSubjectId>
label: Option<DiagnosticMessage>
```

Examples:

```text
kind = "surface_instance", id = "42", label = "Inspector"
kind = "surface_host", id = "main_dock"
kind = "entity", id = "12", label = "Player"
kind = "scene_file", id = "res://levels/test.scene"
kind = "render_resource", id = "depth"
```

Do not use a global enum of every possible subject. Do not reduce subjects to display-only strings.

### `DiagnosticMetadata`

Carries structured facts that should not be embedded only in the human message.

Examples:

```text
expected = "registered surface host"
actual = "host id 7"
field = "parent"
path = "entities[4].parent"
version = 3
```

Recommended value kinds:

```text
string
integer
float
bool
id-string
path
```

Metadata must remain simple and portable. It should not become an arbitrary object graph or JSON dumping ground.

### `DiagnosticRelated`

Carries related diagnostics or references to related diagnostic information.

Use cases:

```text
editor_shell.projection.failed
  related: ui_surface.mount.unknown_host

asset.import.failed
  related: asset.decode.unsupported_format
```

The first implementation may store related diagnostics directly or store lightweight references, but the public model should allow diagnostic relationships without requiring message parsing.

### `DiagnosticReport`

A report should aggregate diagnostics and provide summary helpers.

Useful helpers:

```text
is_empty
has_warnings
has_errors
has_fatal
highest_severity
count_by_severity
extend
merge
```

The report should not decide whether a candidate is accepted. That is ratification’s job.

### `DiagnosticSink`

Minimal sink interface:

```text
emit(diagnostic)
```

`DiagnosticReport` should be usable as a collector/sink.

This avoids every domain inventing ad-hoc collection plumbing.

---

## Relationship to Existing Error Types

Existing domain/editor errors should not be deleted immediately.

Recommended migration model:

```text
existing local error -> optional diagnostic conversion
```

Do not force every error type to become a diagnostic.

Do not replace `EditorMutationError` or `GoverningChangeError` as part of the first diagnostics PR.

Do not replace tracing/logging with diagnostics. Logging and diagnostics can coexist:

```text
diagnostics = structured domain issue reporting
tracing/logging = runtime event/observability stream
```

---

## Integration Plan

Recommended first consumers:

1. `ui_surface` validation/ratification reports.
2. `editor_shell` projection or route validation.
3. `editor_persistence` scene file validation errors.
4. render graph validation if/when a stable validation boundary exists.

Do not begin by changing every error type in the workspace.

---

## Migration Plan

Phase 1:

```text
Create foundation/diagnostics with core value types, validating constructors, structured subjects, metadata, related diagnostics, report aggregation, sink support, and tests.
```

Phase 2:

```text
Adapt one small domain boundary, preferably ui_surface validation, to emit diagnostics or convert its issues into diagnostics.
```

Phase 3:

```text
Adapt editor_shell projection/route validation once the diagnostic shape proves useful.
```

Phase 4:

```text
Add optional conversions from editor_persistence validation errors if useful.
```

Do not migrate `editor_core` governance errors in the first pass.

---

## First Implementation Boundary

The first implementation should include the long-term core model, but not all integrations.

Include now:

```text
Severity
DiagnosticCode
DiagnosticDomain
DiagnosticSubject
DiagnosticLocation
DiagnosticMessage
DiagnosticNote
DiagnosticMetadata
DiagnosticRelated
Diagnostic
DiagnosticReport
DiagnosticSink
validating constructors
basic tests
optional serde feature if low-friction
```

Do not include now:

```text
global registry
proc macros
logging/tracing adapter
editor panel adapter
ratification crate integration
workspace-wide error migration
domain-specific code catalog
```

This is not a shortcut. It establishes the durable public shape while avoiding premature adapter work.
