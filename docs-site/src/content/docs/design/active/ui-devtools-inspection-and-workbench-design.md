---
title: UI Devtools Inspection And Workbench Design
description: Long-term developer tooling, inspector, source-map navigation, live preview, graph inspection, runtime state inspection, profiling, diagnostics, and workbench requirements for Runenwerk UI.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-testing-conformance-and-proof-matrix-design.md
  - ./ui-performance-virtualization-assets-and-profiling-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Devtools Inspection And Workbench Design

## Status

Active long-term UI design direction. This document defines developer tooling,
inspection, source-map navigation, live preview, profiling, diagnostics, and
workbench requirements. It does not authorize implementation by itself.

## Decision

A mature Runenwerk UI framework must be inspectable. Devtools are not optional.

A UI implementation is not acceptable if it cannot explain:

```text
where source came from
which packages contributed controls/components
which source lowered to which program nodes
why layout resolved to final rects
why style/theme tokens resolved to final values
why a binding updated
why a route/action was accepted or rejected
why focus moved
why an output packet was emitted
why runtime state was preserved or reset
why a host accepted or rejected a feature
```

## Inspection Surfaces

Required tools/panels:

```text
SourceInspector
ProgramGraphInspector
PackageCatalogInspector
ControlDescriptorInspector
ComponentInspector
LayoutInspector
StyleThemeInspector
BindingInspector
RuntimeStateInspector
InputRoutingInspector
FocusNavigationInspector
AccessibilityInspector
RenderPacketInspector
ArtifactInspector
CacheProfiler
LivePreviewInspector
ProofReplayViewer
HostCompatibilityInspector
MigrationInspector
DiagnosticDashboard
```

## Source-Map Navigation

Source maps must support navigation across:

```text
Rust projection source
RON/template source
visual designer source
generated source
component template expansion
package descriptor source
UiProgram nodes
runtime artifact tables
UiOutput packets
diagnostics
proof reports
```

Every generated or transformed stage should preserve enough provenance to explain
and repair defects.

## Runtime Inspection

Runtime inspection must expose:

```text
active surfaces
current UiProgram revision
current artifact id
current evaluation revision
retained runtime state
focus scopes
hover/pressed/capture state
scroll state
text edit/IME state
animation clocks
popup stack
host data snapshot
input modality
```

Runtime inspection must not mutate state unless routed through explicit debug
capabilities and reports.

## Graph Inspection

Graph inspectors should show:

```text
control graph
layout graph
style graph
interaction graph
binding graph
accessibility graph
visual/render packet graph where applicable
invalidated dependency graph
```

Graph inspection must distinguish source graph, program graph, dependency graph,
and runtime artifact tables. These are not one structure.

## Live Preview Devtools

Live preview tools should show:

```text
source edit timeline
validation diagnostics
normalization changes
program diff
artifact invalidation
state retention decisions
output delta
host application result
last-known-good snapshot
rollback target
```

Invalid preview source should be inspectable without replacing the last known good
runtime artifact.

## Performance And Cache Devtools

Required views:

```text
evaluation time
layout time
style resolution time
text shaping time
binding evaluation time
render packet generation time
asset loading time
cache hit/miss counts
virtualization realized item count
allocation and memory pressure
hot-path violation report
```

Profiler events must carry source/program/artifact ids where possible.

## Diagnostics Dashboard

Diagnostics must be queryable by:

```text
severity
source id
program node id
package id
host profile
capability id
diagnostic category
proof scenario
migration status
```

Diagnostics should include suggested repair where available.

## Remote And Headless Inspection

Remote/headless hosts must be able to emit machine-readable inspection reports:

```text
UiInspectionBundle
UiProofBundle
UiReplayBundle
UiPerformanceTrace
UiCompatibilityReport
```

A remote devtools connection must be capability-checked and must not expose
untrusted package execution or host services by default.

## Workbench Integration

The future Runenwerk workbench should be able to inspect and edit UI programs
beside other domain programs.

Workbench UI should support:

```text
domain browser
source editor
visual UI editor
component catalog browser
package catalog browser
program graph viewer
artifact viewer
preview surface
fixture runner
migration panel
proof report viewer
performance dashboard
```

The workbench is a host and authoring product. It must not bypass the UI source
and program pipeline.

## Reports

Required reports:

```text
UiInspectionReport
UiSourceMapNavigationReport
UiGraphInspectionReport
UiRuntimeStateInspectionReport
UiDevtoolsCapabilityReport
UiLivePreviewInspectionReport
UiProfilerInspectionReport
UiDiagnosticDashboardReport
UiRemoteInspectionReport
```

## Rejected Shapes

Reject:

```text
ad hoc debug logs as the only inspection surface
visual preview without source-map navigation
devtools mutating runtime state without debug capability report
one graph viewer that collapses source/program/runtime/artifact graphs
remote inspection without trust/capability checks
proof replay visible only as terminal text
```

## Acceptance Criteria

A first devtools proof should demonstrate:

```text
select a UI output packet and navigate back to source
inspect route/action rejection diagnostics
inspect focus movement trace
inspect retained state preservation decision
view live-preview program diff
view cache hit/miss report
export headless inspection bundle
```
