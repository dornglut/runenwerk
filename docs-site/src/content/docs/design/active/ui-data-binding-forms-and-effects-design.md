---
title: UI Data Binding Forms And Effects Design
description: Long-term data binding, forms, validation, command/action, async effect proposal, loading, error, cancellation, and collection binding model for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./typed-app-program-counter-proof-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Data Binding Forms And Effects Design

## Status

Active long-term UI design direction. This document defines data binding, form,
validation, command/action, async effect proposal, loading, error, cancellation,
and collection binding requirements. It does not authorize implementation by
itself.

## Decision

UI binding must be explicit, typed, capability-checked, and reportable.

Bindings may read snapshots and emit proposals. They must not hide app, editor,
game, asset, or host mutation inside controls.

Correct shape:

```text
snapshot data -> binding evaluation -> UI output facts
UI event -> route/action resolution -> typed app/domain action or effect proposal
host/app accepts or rejects proposal -> new snapshot revision
```

Rejected shape:

```text
control callback mutates app/game/editor state directly
```

## Binding Kinds

Required binding kinds:

```text
ReadBinding
ComputedBinding
CollectionBinding
SelectionBinding
TextInputBinding
WriteProposalBinding
HostDataBinding
AsyncStatusBinding
ValidationBinding
LocalizationBinding
ThemeBinding
```

Every binding must declare:

```text
binding id
binding version
source node id
input schema
output schema
dependency path(s)
capability requirements
failure policy
diagnostics
source-map provenance
```

## Snapshot Rules

Bindings read immutable snapshots:

```text
AppModelSnapshot
HostDataSnapshot
UiRuntimeStateSnapshot
ThemeSnapshot
LocalizationSnapshot
PackageCatalogSnapshot
```

A binding evaluation must not observe changing state halfway through one
evaluation revision. New state starts a new revision.

## Forms

Forms require explicit field models:

```text
FormId
FieldId
FieldSchema
FieldValue
FieldDraftValue
FieldValidationState
FieldDirtyState
FieldTouchedState
FieldSubmitState
```

Form controls must distinguish:

```text
committed model value
draft UI value
validation result
submission request
submission status
```

Text input must handle:

```text
cursor
selection
IME composition
undo/redo within field
copy/paste/cut
placeholder
password/security mode
input filters
normalization policy
```

## Validation

Validation can run at multiple levels:

```text
source/schema validation
field validation
form validation
cross-field validation
host/domain validation
async validation
submission validation
```

Validation output:

```text
ValidationReport
field diagnostics
form diagnostics
severity
source-map or field-map location
suggested repair where available
blocking/non-blocking flag
```

Validation must not be represented only as rendered red text. It is semantic
state with diagnostics and accessibility consequences.

## Actions And Commands

UI events route to typed actions or command proposals.

Action facts:

```text
action id
action version
payload schema
required capability
availability state
route id
source control id
host compatibility status
```

Command/effect proposal facts:

```text
proposal id
proposal kind
payload schema
required capability
origin source id
origin route id
expected host
cancellation policy
timeout policy
retry policy
idempotency key where applicable
```

## Async Effects

Async work must be represented as host/app-owned effect proposals and status
bindings, not hidden UI tasks.

Async lifecycle:

```text
NotStarted
Requested
Pending
Succeeded
Failed
Cancelled
TimedOut
Retried
Stale
```

Async reports must include:

```text
request id
origin action id
capability decision
host acceptance/rejection
status changes
cancellation
retry attempts
error diagnostics
stale-result handling
```

## Loading And Error UI

Loading and error UI should be modelled as source/program facts:

```text
loading placeholder
skeleton state
progress state
empty state
error state
retry action
fallback content
last-known-good content
```

Controls must expose accessibility state for loading and error conditions.

## Collection Binding

Collection binding must support:

```text
stable item ids
item schemas
insert/remove/move/update diffs
paging
streaming
selection
range selection
sorting
filtering
grouping
virtualization window
recycling policy
empty-state facts
partial-loading facts
```

Index-only identity is unsafe for mutable collections unless explicitly proven.

## Two-Way Binding Policy

Two-way binding is allowed only as explicit paired read/write-proposal bindings.

Correct shape:

```text
read: model.field -> control value
write proposal: control edit -> typed action/effect proposal
reducer/host accepts -> new model revision
```

Rejected shape:

```text
control writes directly into model.field
```

## Failure And Recovery

Every binding/effect must define failure behavior:

```text
show diagnostic
use fallback value
keep last known good value
disable action
retry
cancel
block submit
allow submit with warning
```

Failure must be visible in reports and, where user-facing, accessible in UI.

## Reports

Required reports:

```text
UiBindingEvaluationReport
UiBindingDependencyReport
UiFormValidationReport
UiActionAvailabilityReport
UiRouteActionResolutionReport
UiEffectProposalReport
UiAsyncStatusReport
UiCollectionDiffReport
UiFieldStateRetentionReport
```

## Rejected Shapes

Reject:

```text
untyped binding paths
silent binding failures
hidden async tasks owned by controls
two-way mutation without action/reducer/host boundary
form validation only as visual styling
collection identity by visible index only
callbacks as command semantics
```
