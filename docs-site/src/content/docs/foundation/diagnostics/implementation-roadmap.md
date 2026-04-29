---
title: Diagnostics Implementation Roadmap
description: Phased implementation roadmap for the foundation diagnostics crate.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-04-30
related:
  - ./README.md
  - ../../design/accepted/foundation-diagnostics-design.md
---

# `foundation/diagnostics` Implementation Roadmap

## Purpose

This roadmap defines a phased implementation plan for `foundation/diagnostics`.

The goal is to implement a durable foundation crate for structured diagnostic reporting without turning it into a logger, observability platform, editor UI system, global error hierarchy, ratifier, or command-governance layer.

The design is considered implementation-ready because it has already locked the important long-term boundaries:

- diagnostics are report artifacts, not control-flow errors;
- diagnostics describe issues, but do not decide domain validity;
- reports summarize severity, but do not decide acceptance or rejection;
- subjects are structured, not display-only strings;
- metadata is structured and owned by emitting domains;
- emission order is preserved;
- static diagnostic codes are preferred;
- owned codes exist for deserialization/adapters, not normal domain emission;
- foundation diagnostics must not depend on domain, engine, app, adapter, editor, net, logging, tracing, or UI crates.

---

## Implementation Doctrine

### Design Now, Integrate Gradually

The first implementation must include the long-term core model, but must not force a workspace-wide migration.

Correct:

```text
foundation/diagnostics defines stable reporting vocabulary.
One or two small consumers prove the boundary.
Other domains migrate only when they have a real validation/reporting boundary.
```

Incorrect:

```text
Replace every existing error type.
Move editor_core governance into foundation.
Add global diagnostic registry immediately.
Wire diagnostics into tracing/editor UI immediately.
```

### Diagnostics Are Not Errors

Errors are control-flow artifacts.

Diagnostics are report artifacts.

A function may:

- return an error and emit diagnostics;
- return diagnostics without erroring;
- return a report from validation/ratification;
- convert an existing typed error into a diagnostic for reporting.

Diagnostics must not replace typed domain errors where typed control flow is required.

### Diagnostics Are Not Ratification

A `DiagnosticReport` may answer:

```text
What issues were observed?
How severe are they?
Which subject/location/metadata/related issue is involved?
```

It must not answer:

```text
Is this candidate accepted?
Should this command be applied?
Should this state enter history?
Should this change be shared/reconciled?
```

Those decisions belong to domain ratifiers and higher-level governance.

---

## Recommended PR Split

Use small, reviewable PRs. The crate is foundational; each public API shape should be easy to review.

```text
PR 1  — Design + crate skeleton
PR 2  — Core scalar types
PR 3  — Structured subject and location model
PR 4  — Metadata and related diagnostics
PR 5  — Diagnostic, report, and sink
PR 6  — Optional serde and feature-boundary hardening
PR 7  — First proving consumer: ui_surface
PR 8  — Second proving consumer: editor_shell
PR 9  — Optional third consumer: editor_persistence
PR 10 — Documentation and contributor guidance update
```

PRs 7–9 should not be merged until the crate API has proven stable through its own tests.

---

## Phase 0 — Commit the Design

### Goal

Commit the design document before writing implementation code.

### Files

```text
docs-site/src/content/docs/design/accepted/foundation-diagnostics-design.md
```

### Required Content

The design document must include:

```text
Purpose
Scope
Non-Scope
Architectural Position
Ownership Rules
Public API Policy
Long-Term Compatibility Requirements
Invariants
Failure Modes
Diagnostics
Ratification
Commands
Inspection
Persistence and Versioning
Extension Points
Testing Strategy
Negative Doctrine
Current Repo Findings
Additional Design Decisions
Primary Type Model
Relationship to Existing Error Types
Integration Plan
Migration Plan
First Implementation Boundary
```

### Validation

```bash
cargo fmt --all -- --check
cargo check --workspace
```

### Exit Criteria

- The design is committed.
- The design explicitly says diagnostics are not errors, ratifiers, commands, logging, tracing, or UI.
- The design identifies `ui_surface` as the first proving consumer.
- No Rust code has been introduced yet.

---

## Phase 1 — Add Crate Skeleton

### Goal

Create `foundation/diagnostics` as a workspace crate with no domain dependency leaks.

### Files

```text
foundation/diagnostics/
  Cargo.toml
  src/
    lib.rs
```

Update root workspace manifest:

```text
Cargo.toml
```

Add:

```text
foundation/diagnostics
```

to workspace members.

### Recommended Cargo Features

Use a simple feature model.

```toml
[features]
default = ["std"]
std = ["alloc"]
alloc = []
serde = ["dep:serde", "alloc"]
```

Recommended dependency policy:

```toml
[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
```

### Rules

- Do not depend on `domain/*`.
- Do not depend on `engine`.
- Do not depend on `apps/*`.
- Do not depend on `adapters/*`.
- Do not depend on `net/*`.
- Do not depend on `editor_core`.
- Do not depend on `tracing`, `log`, `opentelemetry`, or UI crates.
- Keep the crate empty except for module declarations and documentation if necessary.

### Validation

```bash
cargo check -p diagnostics
cargo test -p diagnostics
cargo tree -p diagnostics
```

### Exit Criteria

- The crate builds.
- The crate has no forbidden dependencies.
- `cargo tree -p diagnostics` is clean.
- No consumer crate depends on it yet.

---

## Phase 2 — Implement Core Scalar Types

### Goal

Implement the smallest stable diagnostic identity vocabulary.

### Files

```text
foundation/diagnostics/src/severity.rs
foundation/diagnostics/src/code.rs
foundation/diagnostics/src/domain.rs
foundation/diagnostics/src/message.rs
foundation/diagnostics/src/lib.rs
```

### Public Types

```text
Severity
DiagnosticCode
DiagnosticCodeError
DiagnosticDomain
DiagnosticDomainError
DiagnosticMessage
DiagnosticNote
```

### Semantics

`Severity`:

```text
Info
Warning
Error
Fatal
```

Definitions:

```text
Info:
  Relevant observation, not a problem.

Warning:
  Accepted but suspicious, degraded, deprecated, or potentially problematic.

Error:
  The current candidate/operation is invalid or rejected, but the caller/system may continue.

Fatal:
  The current processing context cannot safely continue.
```

`Fatal` must be used sparingly. It is not a stronger spelling of `Error`.

### Diagnostic Code Rules

Diagnostic codes are stable machine-readable identifiers.

Format:

```text
<domain>.<area>.<condition>
```

Examples:

```text
ui_surface.mount.unknown_host
editor_shell.route.stale_projection_epoch
scene.persistence.missing_parent
foundation.diagnostics.code.invalid_format
```

Validation rules:

```text
non-empty
dot-separated
no whitespace
no human sentence text
lowercase domain-style segments preferred
```

Static codes are preferred for normal domain emission.

Owned/dynamic codes are allowed only for:

```text
deserialization
external adapters
tooling
tests where dynamic construction is unavoidable
```

### Diagnostic Domain Rules

Domains identify ownership.

Examples:

```text
foundation.diagnostics
foundation.id
ui_surface
editor_shell
editor_persistence
scene
render_graph
scheduler
engine.net
```

Do not create a global foundation enum of all domains.

### Tests

```text
severity_ordering_is_stable
fatal_is_distinct_from_error
valid_static_code_is_preserved
empty_code_is_rejected
code_with_whitespace_is_rejected
code_with_sentence_text_is_rejected
empty_domain_is_rejected
message_is_not_identity
note_preserves_text
```

### Validation

```bash
cargo test -p diagnostics
cargo check -p diagnostics
```

### Exit Criteria

- Core scalar types exist.
- Constructors preserve basic invariants.
- Static code constants are ergonomic.
- No domain dependency exists.

---

## Phase 3 — Implement Structured Subjects and Locations

### Goal

Make diagnostics tool-readable without requiring message parsing.

### Files

```text
foundation/diagnostics/src/subject.rs
foundation/diagnostics/src/location.rs
foundation/diagnostics/src/lib.rs
```

### Public Types

```text
DiagnosticSubject
DiagnosticSubjectKind
DiagnosticSubjectId
DiagnosticLocation
DiagnosticLocationError
```

The design may still support the phrase `DiagnosticSpan`, but implementation should prefer `DiagnosticLocation` because engine diagnostics are not limited to text/source-code spans.

### Subject Model

Conceptual shape:

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

Rules:

- Subjects are structured descriptors.
- Subjects are not display-only strings.
- Foundation must not define a global enum of all possible subject kinds.
- Subject kind and subject id validation should reject empty values.
- Labels are human-facing, not identity.

### Location Model

`DiagnosticLocation` should support more than source code spans.

Minimum useful location kinds:

```text
text range
file path + optional range
logical path
field path
```

Examples:

```text
file: "assets/scene.rwscene", range: line/column if available
logical path: "workspace.tool_surfaces[2]"
field path: "entities[4].parent"
```

Rules:

- Invalid ranges are rejected.
- Location display output is not parseable API.
- Structured fields are authoritative.

### Tests

```text
subject_preserves_kind_id_and_label
subject_without_id_is_allowed
empty_subject_kind_is_rejected
empty_subject_id_is_rejected
label_is_not_identity
location_rejects_invalid_range
location_can_represent_file_path
location_can_represent_field_path
location_can_represent_logical_path
```

### Validation

```bash
cargo test -p diagnostics
cargo check -p diagnostics
```

### Exit Criteria

- Subjects are structured.
- Locations are generic enough for code, asset, scene, UI, and graph diagnostics.
- No caller has to parse a message to identify a subject.

---

## Phase 4 — Implement Metadata and Related Diagnostics

### Goal

Support structured context without turning diagnostics into arbitrary object graphs.

### Files

```text
foundation/diagnostics/src/metadata.rs
foundation/diagnostics/src/related.rs
foundation/diagnostics/src/lib.rs
```

### Public Types

```text
DiagnosticMetadata
DiagnosticMetadataEntry
DiagnosticMetadataKey
DiagnosticMetadataValue
DiagnosticRelated
```

### Metadata Value Types

For v1, use:

```text
String
Integer
Bool
Id
Path
```

Do not include floats in v1 unless there is an immediate proving use case.

Reason:

```text
floats complicate Eq
floats complicate deterministic golden tests
floats often require tolerance semantics
```

A float value can be added later if a concrete need appears.

### Metadata Rules

- Metadata preserves insertion order.
- Metadata keys are structured strings.
- Metadata keys are owned by the emitting domain.
- Metadata must not become a JSON dumping ground.
- Metadata values must remain portable and serialization-compatible.
- Metadata display output is not parseable API.

Examples:

```text
expected = "registered surface host"
actual = "host id 7"
field = "parent"
path = "entities[4].parent"
version = 3
```

### Related Diagnostics

For v1, related diagnostics should be lightweight references/summaries, not recursive full diagnostics.

Recommended conceptual shape:

```text
code
domain
subject: optional
message: optional
```

Rules:

- Related diagnostics preserve order.
- Related diagnostics do not recursively contain full diagnostics in v1.
- Related diagnostics must not create graph/cycle complexity.
- Related diagnostics help explain context but are not the authoritative report list.

### Tests

```text
metadata_preserves_order
metadata_rejects_empty_key
metadata_preserves_key_value_identity
metadata_value_string_is_preserved
metadata_value_integer_is_preserved
metadata_value_bool_is_preserved
metadata_value_id_is_preserved
metadata_value_path_is_preserved
related_preserves_code_domain_subject
related_preserves_order
related_does_not_require_recursive_diagnostic
```

### Validation

```bash
cargo test -p diagnostics
cargo check -p diagnostics
```

### Exit Criteria

- Structured metadata exists.
- Metadata does not require arbitrary JSON.
- Related diagnostics are cycle-safe and deterministic.

---

## Phase 5 — Implement Diagnostic

### Goal

Create the core diagnostic artifact.

### Files

```text
foundation/diagnostics/src/diagnostic.rs
foundation/diagnostics/src/lib.rs
```

### Public Type

```text
Diagnostic
```

### Fields

```text
severity
code
domain
subject
location
message
notes
metadata
related
```

Optional future field if needed:

```text
source
```

`DiagnosticSource` may be introduced later if metadata proves insufficient for provenance/pipeline-stage reporting. Do not add it in v1 unless a real consumer needs it.

### Construction Policy

A diagnostic must have:

```text
severity
code
domain
message
```

Optional:

```text
subject
location
notes
metadata
related
```

Builder-style construction is allowed, but required fields must be enforced.

Recommended ergonomic pattern:

```text
Diagnostic::new(severity, code, domain, message)
  .with_subject(subject)
  .with_location(location)
  .with_note(note)
  .with_metadata(key, value)
  .with_related(related)
```

### Rules

- `Diagnostic` is an observation artifact.
- `Diagnostic` does not implement domain validation.
- `Diagnostic` does not implement command execution.
- Display output is for humans/debugging only.
- Structured fields are authoritative.

### Tests

```text
diagnostic_requires_core_fields
diagnostic_preserves_severity_code_domain_message
diagnostic_preserves_subject
diagnostic_preserves_location
diagnostic_preserves_notes
diagnostic_preserves_metadata
diagnostic_preserves_related
diagnostic_debug_output_is_available
diagnostic_display_output_is_not_identity
```

### Validation

```bash
cargo test -p diagnostics
cargo check -p diagnostics
```

### Exit Criteria

- A complete diagnostic can be constructed.
- Required fields are enforced.
- Optional structured context is preserved.

---

## Phase 6 — Implement Report Aggregation and Sink

### Goal

Provide deterministic collection and aggregation.

### Files

```text
foundation/diagnostics/src/report.rs
foundation/diagnostics/src/sink.rs
foundation/diagnostics/src/lib.rs
```

### Public Types

```text
DiagnosticReport
DiagnosticSink
```

### Report Helpers

```text
new
is_empty
len
iter
push
extend
merge
has_warnings
has_errors
has_fatal
highest_severity
count_by_severity
```

### Sink Interface

Minimal:

```text
emit(diagnostic)
```

`DiagnosticReport` should implement `DiagnosticSink`.

### Rules

- Emission order is preserved.
- Merging preserves left-to-right order.
- Reports may summarize severity.
- Reports must not define acceptance/rejection policy.
- Filtering, if added later, is caller-owned policy.
- No implicit sorting unless a caller explicitly requests it.

### Tests

```text
sink_collects_in_emission_order
report_push_preserves_order
report_extend_preserves_order
report_merge_preserves_left_to_right_order
report_counts_by_severity
report_highest_severity_is_stable
report_has_warning_error_fatal_helpers_work
report_helpers_do_not_define_acceptance
```

### Validation

```bash
cargo test -p diagnostics
cargo check -p diagnostics
```

### Exit Criteria

- Reports are deterministic.
- Reports can collect diagnostics through a sink.
- Aggregation helpers work.
- Acceptance remains outside diagnostics.

---

## Phase 7 — Optional Serde Feature

### Goal

Make diagnostics serialization-compatible without making serde mandatory.

### Files

```text
foundation/diagnostics/Cargo.toml
foundation/diagnostics/src/*.rs
```

### Rules

- `serde` is optional.
- `serde` requires `alloc`.
- Serialized diagnostics are observation artifacts.
- Serialized output preserves structured identity.
- Do not include non-serializable internals in public type shapes.

### Tests

```text
serde_round_trip_preserves_code_domain_severity_message
serde_round_trip_preserves_subject
serde_round_trip_preserves_location
serde_round_trip_preserves_metadata
serde_round_trip_preserves_related
serde_round_trip_preserves_report_order
```

### Validation

```bash
cargo test -p diagnostics
cargo test -p diagnostics --features serde
cargo check -p diagnostics --no-default-features
cargo check -p diagnostics --no-default-features --features alloc
```

### Exit Criteria

- Serde is optional.
- Core usage does not require serde.
- Serialization preserves structured identity.
- No forbidden dependencies appear through feature flags.

---

## Phase 8 — Feature and Dependency Boundary Hardening

### Goal

Prove that the crate remains foundational.

### Checks

```bash
cargo tree -p diagnostics
cargo tree -p diagnostics --features serde
cargo check -p diagnostics
cargo check -p diagnostics --no-default-features
cargo check -p diagnostics --no-default-features --features alloc
cargo test -p diagnostics
cargo test -p diagnostics --features serde
```

### Boundary Requirements

`foundation/diagnostics` must not depend on:

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
tracing
log
opentelemetry
UI crates
```

### Exit Criteria

- Feature matrix is validated.
- No dependency leaks exist.
- All tests pass.
- Crate remains usable before any domain integration.

---

## Phase 9 — First Proving Consumer: `ui_surface`

### Goal

Use diagnostics in one small, dependency-light domain boundary.

`ui_surface` is the best first consumer because it already has surface capability, mount, intent, and ratification-like boundaries while remaining small.

### Likely Files

```text
domain/ui/ui_surface/Cargo.toml
domain/ui/ui_surface/src/ratification.rs
domain/ui/ui_surface/src/lib.rs
```

Exact files should be confirmed from the current repo before patching.

### Scope

Add diagnostics narrowly.

Do not rewrite all existing errors.

Possible code family:

```text
ui_surface.capability.missing
ui_surface.intent.unsupported
ui_surface.mount.unknown_definition
ui_surface.mount.unknown_host
ui_surface.mount.duplicate_instance
```

### Rules

- `ui_surface` owns `ui_surface.*` codes.
- Existing typed errors may remain.
- Add conversion/reporting where it improves validation visibility.
- Do not introduce editor concepts into `ui_surface`.
- Do not make `ui_surface` depend on `editor_core`.

### Tests

```text
missing_surface_capability_emits_diagnostic
unsupported_surface_intent_emits_diagnostic
ui_surface_diagnostic_codes_are_stable
ui_surface_diagnostic_subjects_are_structured
ui_surface_diagnostic_order_is_deterministic
```

### Validation

```bash
cargo test -p ui_surface
cargo test -p diagnostics
cargo check --workspace
```

### Exit Criteria

- `ui_surface` emits or converts to diagnostics in one narrow path.
- Diagnostic subjects are structured.
- Existing domain behavior is preserved.
- No editor/runtime dependency leaks into `ui_surface`.

---

## Phase 10 — Second Proving Consumer: `editor_shell`

### Goal

Use diagnostics for projection/route/structural-context failures.

### Likely Files

```text
domain/editor/editor_shell/Cargo.toml
domain/editor/editor_shell/src/workspace/projection.rs
domain/editor/editor_shell/src/commands/map_interactions.rs
```

Exact files should be confirmed from the current repo before patching.

### Possible Code Family

```text
editor_shell.projection.invalid_surface_definition
editor_shell.route.stale_projection_epoch
editor_shell.route.missing_structural_context
editor_shell.command.unsupported_surface_intent
```

### Rules

- `editor_shell` owns `editor_shell.*` codes.
- `editor_shell` may pass through `ui_surface.*` diagnostics if it directly consumes `ui_surface`.
- Do not move editor governance into diagnostics.
- Do not replace command execution.
- Do not replace ratified change history.

### Tests

```text
stale_projection_epoch_emits_diagnostic
missing_structural_context_emits_diagnostic
projection_diagnostics_are_deterministic
editor_shell_passes_through_ui_surface_diagnostics_without_rewriting_codes
```

### Validation

```bash
cargo test -p editor_shell
cargo check -p runenwerk_editor
cargo check --workspace
```

### Exit Criteria

- Projection/route diagnostics exist where valuable.
- Diagnostics improve observability without changing command semantics.
- Existing shell behavior remains intact.

---

## Phase 11 — Optional Third Consumer: `editor_persistence`

### Goal

Convert persistence validation failures into structured diagnostics while preserving typed persistence errors.

### Likely Files

```text
domain/editor/editor_persistence/Cargo.toml
domain/editor/editor_persistence/src/...
```

Exact files should be confirmed from the current repo before patching.

### Possible Code Family

```text
editor_persistence.scene.unsupported_version
editor_persistence.scene.duplicate_entity_id
editor_persistence.scene.missing_parent
editor_persistence.scene.cyclic_parent_reference
```

### Rules

- Do not replace persistence error types if they are useful for control flow.
- Add diagnostics as reporting artifacts.
- Use structured locations for file/path/entity references.
- Preserve deterministic ordering for validation reports.

### Tests

```text
unsupported_scene_version_emits_diagnostic
duplicate_entity_id_emits_diagnostic
missing_parent_emits_diagnostic
cyclic_parent_reference_emits_diagnostic
persistence_diagnostic_order_is_deterministic
```

### Validation

```bash
cargo test -p editor_persistence
cargo test -p diagnostics
cargo check --workspace
```

### Exit Criteria

- Persistence validation can produce structured diagnostics.
- Existing persistence behavior is preserved.
- Reports are deterministic and inspectable.

---

## Phase 12 — Documentation and Contributor Guidance

### Goal

Document how future domains should use diagnostics.

### Files

```text
CRATES.md
DOMAIN_MAP.md
AI_GUIDE.md
TESTING.md
docs-site/src/content/docs/design/accepted/foundation-diagnostics-design.md
```

Optional future doc:

```text
docs/guides/diagnostics.md
```

### Add Guidance For

```text
how to define diagnostic codes
how to choose a domain
how to create a subject
how to use metadata
how to use related diagnostics
how to emit into a report
how to convert a domain error
what not to put in diagnostics
how diagnostics differ from errors
how diagnostics differ from ratification
how diagnostics differ from tracing/logging
```

### Validation

```bash
cargo test --workspace
cargo check --workspace
```

### Exit Criteria

- Future contributors know how to add diagnostics.
- AI/human contributors know what not to do.
- Root docs and design docs remain aligned.

---

## Full Validation Checklist

Before considering `foundation/diagnostics` v1 complete:

```bash
cargo fmt --all -- --check
cargo test -p diagnostics
cargo test -p diagnostics --features serde
cargo check -p diagnostics
cargo check -p diagnostics --no-default-features
cargo check -p diagnostics --no-default-features --features alloc
cargo tree -p diagnostics
cargo tree -p diagnostics --features serde
cargo check --workspace
cargo test --workspace
```

Consumer-specific checks after integration:

```bash
cargo test -p ui_surface
cargo test -p editor_shell
cargo test -p editor_persistence
cargo check -p runenwerk_editor
```

---

## Definition of Done for `foundation/diagnostics` v1

```text
foundation/diagnostics exists as a workspace crate
core public types exist
constructors preserve invariants
subjects are structured
locations are generic and not source-code-only
metadata is structured and ordered
related diagnostics are lightweight and ordered
Diagnostic exists as an observation artifact
DiagnosticReport preserves emission order
DiagnosticSink exists
reports aggregate severity without deciding acceptance
serde is optional
feature matrix passes
no forbidden dependencies exist
at least one domain crate consumes diagnostics
docs explain code ownership and non-goals
cargo test -p diagnostics passes
cargo check --workspace passes
```

---

## Explicit Non-Goals for v1

Do not implement:

```text
global diagnostic registry
diagnostic proc macros
logging/tracing adapter
OpenTelemetry adapter
editor diagnostic panel
ratification crate integration
workspace-wide error migration
domain-specific global code catalog
automatic remediation commands
diagnostic persistence database
network reporting protocol
```

These may be designed later as separate adapters or consumers if real use cases emerge.

---

## Recommended Immediate Next Step

Start with:

```text
PR 1 — Design + Crate Skeleton
```

Contents:

```text
docs-site/src/content/docs/design/accepted/foundation-diagnostics-design.md
foundation/diagnostics/Cargo.toml
foundation/diagnostics/src/lib.rs
root Cargo.toml workspace member update
```

Validation:

```bash
cargo check -p diagnostics
cargo test -p diagnostics
cargo tree -p diagnostics
cargo check --workspace
```

Then proceed to scalar types in PR 2.
