---
title: UI Testing Conformance And Proof Matrix Design
description: Long-term testing, conformance, proof matrix, replay, fixture, snapshot, interaction, accessibility, performance, migration, and host compatibility requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-accessibility-internationalization-and-text-conformance-design.md
  - ./ui-performance-virtualization-assets-and-profiling-design.md
  - ./typed-app-program-counter-proof-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Testing Conformance And Proof Matrix Design

## Status

Active long-term UI design direction. This document defines testing,
conformance, proof matrix, replay, fixture, snapshot, interaction, accessibility,
performance, migration, and host compatibility requirements. It does not
authorize implementation by itself.

## Decision

Runenwerk UI correctness must be proven through deterministic fixtures, replay,
structural assertions, diagnostics, source maps, host compatibility reports, and
runtime evidence.

Rendered screenshots are useful, but they are not sufficient as the only proof.

## Test Layers

Required test layers:

```text
source validation tests
normalization tests
interaction formation tests
UiProgram formation tests
compiler/artifact tests
evaluator tests
reactive invalidation tests
binding tests
state retention tests
layout tests
style/theme tests
text/i18n tests
accessibility tests
input/focus/navigation tests
game/world-space host tests
live-preview tests
performance/virtualization tests
migration tests
renderer boundary tests
```

## Proof Artifacts

Every proof should produce machine-readable artifacts:

```text
ProofManifest
FixtureManifest
InputTrace
ReplayTrace
SourceValidationReport
UiProgramFormationReport
UiRuntimeArtifactReport
UiEvaluationReport
UiUpdateReport
UiOutputSummary
UiDiagnosticReport
HostCompatibilityMatrix
SourceMapReport
```

## Replay Model

Replay inputs:

```text
initial source snapshot
initial app/model snapshot
initial runtime state snapshot
host profile
theme/profile snapshot
package catalog snapshot
input event sequence
expected assertions
```

Replay output:

```text
step id
input event
route/action resolution
model/effect outcome
invalidated scope
retained state decision
output delta
host application report
diagnostics
assertion result
```

## Assertion Types

Required assertion types:

```text
source assertion
program graph assertion
package requirement assertion
capability assertion
route assertion
action availability assertion
reducer outcome assertion
effect proposal assertion
layout rect assertion
style token assertion
text output assertion
accessibility semantic assertion
focus/navigation assertion
input routing assertion
output delta assertion
artifact cache assertion
diagnostic assertion
performance budget assertion
host compatibility assertion
```

## Golden And Visual Tests

Visual tests are allowed but must be paired with structural facts.

Visual proof should include:

```text
render target profile
DPI scale
font profile
locale
theme
host profile
screenshot hash or perceptual diff
structural output summary
source-map links
known tolerance policy
```

A screenshot without structural report is not sufficient evidence.

## Accessibility Conformance Tests

Accessibility tests must verify:

```text
accessible name/description
role/state/value
focus order
keyboard activation
controller activation where required
modal focus trapping
error/validation semantics
semantic tree stability
high-contrast/reduced-motion behavior
text scaling behavior
```

## Performance Tests

Performance tests must verify:

```text
budget compliance
cache hits/misses
virtualization realization count
container recycling correctness
allocation budget
frame hot-path exclusions
profiler event presence
large collection behavior
live-preview hot-swap budget
```

## Host Compatibility Tests

Every host profile must report:

```text
accepted packages
rejected packages
accepted controls
unsupported controls
input support
surface support
accessibility support
text/font support
animation support
world-space support
fallback requirements
```

## Migration Tests

Migration tests must verify:

```text
source version migration
control package migration
schema migration
theme token migration
localization key migration
runtime state migration or reset
artifact invalidation
backward compatibility stubs where required
```

## Counter Minimum Proof Matrix

Counter first proof must include:

```text
initial_counting_screen
increment_once
increment_to_nine
increment_to_ten_switches_to_win
increment_after_win_rejected
reset_from_win_returns_to_counter
decrement_at_zero_rejected
reject_unknown_route
reject_missing_capability
reject_invalid_payload
reactive_update_report_present
no_product_event_packet_plumbing
```

## Maturity Gate Matrix

A UI implementation is not mature until it proves:

```text
base controls
component/slot composition
reactive updates
retained state
text editing and IME
accessibility
localization and bidi
layout and overflow
style/theme variants
animation and reduced motion
overlay/popup/layering
large virtualized collections
gamepad navigation
world-space projection
live editing and preview
host compatibility
performance budgets
migration
renderer boundary
```

## Rejected Test Shapes

Reject:

```text
screenshots as the only proof
manual visual inspection as readiness evidence
hidden fixture setup
non-deterministic replay
route/action rejection without diagnostics
performance tests without budgets
accessibility claims without semantic assertions
host compatibility only in prose
```
