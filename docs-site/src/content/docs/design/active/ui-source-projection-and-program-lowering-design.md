---
title: UI Source Projection And Program Lowering Design
description: UI-specific source vocabulary and lowering contract for app projections, authored templates, normalized templates, interaction formation, UiProgram, runtime artifacts, evaluation, and reports.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./domain-authoring-source-and-program-pattern.md
  - ./typed-app-program-counter-proof-design.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
  - ../../domain/ui/architecture.md
---

# UI Source Projection And Program Lowering Design

## Status

Active UI design direction. This document defines vocabulary and required
boundaries for UI source produced by app projections, Rust builders, RON/source
assets, visual designers, and generated tools. It does not authorize broad
implementation, crate creation, renderer changes, or product-specific bypasses.

## Decision

Use `UiSource` as the public UI source-stage term.

Do not use `UiSourceAst` as the public architecture term. AST, tree, graph,
template, imported, generated, and visual-designer bodies may exist as concrete
`UiSource` body kinds.

The UI lowering pipeline is:

```text
UiSource
-> AuthoredUiTemplate
-> NormalizedUiTemplate
-> FormedInteractionModel
-> UiProgram
-> UiRuntimeArtifact
-> UiEvaluator
-> UiOutput
   -> UiFrame
   -> UiEventPacket
   -> UiInspectionReport
   -> UiDiagnosticReport
```

`UiSource` is not a retained runtime tree, not an ECS entity set, not a renderer
primitive list, and not app mutation authority.

## UiSource Envelope

A `UiSource` record should carry or preserve:

```text
source id
source version
source body kind
required package references
required capabilities
route/action bindings
schema references
source-map provenance
localization keys
accessibility metadata
theme/style references
host requirements
validation diagnostics
```

## Source Body Kinds

Supported long-term body kinds:

```text
TreeBody
TemplateBody
GraphBody
GeneratedProjectionBody
ImportedBody
VisualDesignerBody
CompatibilityBody
```

A body kind describes source representation only. UI meaning still comes from UI
packages, control descriptors, schemas, interaction formation, and `UiProgram`.

## App Projection Boundary

An app-program projection may produce `UiSource`.

The app projection may know:

```text
app model read data
typed app actions
action availability
localization text keys
source ids for projected nodes
```

The app projection must not:

```text
build UiProgram directly unless an explicit proof-local design allows it
build render primitives
construct UiEventPacket manually
own hit targets
own prepared frames
mutate app model from UI controls
bypass route/capability checks
store retained WidgetId or ECS Entity ids in source
```

## UiActionProjection

`UiActionProjection<Action>` maps typed app actions into UI source action
bindings.

It should provide controls such as:

```text
actions.button(Action)
actions.menu_item(Action)
actions.action_prompt(Action)
```

The projection must lower to source facts containing:

```text
source control id
route id
action id
action version
route schema version
required capability
payload schema reference
availability state
accessible label
source-map provenance
```

`UiActionProjection` is a convenience over explicit route/action metadata. It is
not hidden callback mutation.

## Package Requirements

Every source node kind must resolve through package-backed descriptors.

For the counter proof, the UI source must declare at least:

```text
runenwerk.ui.base_controls
```

Base controls such as `column`, `row`, `text`, and `button` are UI package
contributions. They must not grow through a central hardcoded enum that becomes a
catalog bottleneck.

## Accessibility And Localization

UI source should make accessibility and localization first-class:

```text
roles
accessible names
accessible descriptions
focus traversal
keyboard activation
text keys
format arguments
fallback text
```

Direct English strings may be allowed in proof sketches, but real source should
preserve stable text keys and report missing localization metadata.

## Theme And Layout Tokens

Source should reference tokens rather than hardcoding final renderer values:

```text
spacing tokens
typography tokens
color tokens
radius tokens
layout constraints
overflow policy
```

Token resolution belongs in UI theme/layout/program/artifact stages, not product
app code.

## Lowering Reports

The lowering path must produce reports for:

```text
UiSourceValidationReport
UiNormalizationReport
FormedInteractionReport
UiProgramFormationReport
UiPackageResolutionReport
UiCompilerReport
UiEvaluationReport
UiSourceMapReport
UiAccessibilityReport
```

Reports are required for humans, CI, and AI agents. Generated views are not
sufficient evidence unless their reports are inspectable.

## Relationship To UiProgram

`UiProgram` remains the durable executable UI-domain contract. `UiSource` is an
authoring/projection input. A retained tree, runtime artifact, and renderer frame
are derived products.

Correct direction:

```text
UiSource -> UiProgram -> UiRuntimeArtifact -> UiOutput
```

Rejected direction:

```text
UiSource -> renderer primitives
UiSource -> app callback mutation
UiSource -> ECS semantic ownership
UiSource -> retained WidgetId source truth
```

## Stop Conditions

Stop and redesign if UI source implementation requires:

```text
public UiSourceAst as the only source form
UiTree as source truth
retained WidgetId in authored source
ECS Entity id in authored source
renderer primitive in authored source
callbacks as action semantics
hidden global mutable control registry
product app route/event packet construction
```
