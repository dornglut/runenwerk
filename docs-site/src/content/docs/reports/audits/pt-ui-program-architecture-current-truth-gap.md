---
title: PT UI Program Architecture Current Truth Gap
description: Audit of current domain/ui code truth against the final UiProgram architecture implementation target.
status: completed
owner: ui
layer: domain
canonical: false
last_reviewed: 2026-06-01
related:
  - ../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../design/active/ui-program-architecture.md
  - ../../design/active/ui-program-contract-design.md
  - ../../workspace/track-execution-manifests/pt-ui-program.yaml
---

# PT UI Program Architecture Current Truth Gap

## Purpose

This audit records the current repository truth before creating
`PT-UI-PROGRAM-ARCHITECTURE`.

It supersedes no historical closeout. It does not authorize product code,
crate creation, MaterialProgram work, placeholder future folders, or
`foundation/meta` extraction.

## Current Code Truth

The current UI domain has retained-compatible proof-slice crates:

- `domain/ui/ui_definition`
- `domain/ui/ui_graph_editor`
- `domain/ui/ui_input`
- `domain/ui/ui_layout`
- `domain/ui/ui_math`
- `domain/ui/ui_render_data`
- `domain/ui/ui_runtime`
- `domain/ui/ui_surface`
- `domain/ui/ui_text`
- `domain/ui/ui_theme`
- `domain/ui/ui_tree`
- `domain/ui/ui_widgets`

Those crates contain useful retained UI runtime, widget, render-data, surface,
text, theme, input, and proof-slice evidence. They are not the final UiProgram
architecture implementation.

## Missing Final Architecture Contracts

The following long-term architecture contracts are not currently implemented
as concrete code truth:

- `domain/ui/ui_schema`
- `domain/ui/ui_program`
- `domain/ui/ui_controls`
- `domain/ui/ui_compiler`
- `domain/ui/ui_artifacts`
- `domain/ui/ui_evaluator`
- `domain/ui/ui_state`
- `domain/ui/ui_binding`
- `domain/ui/ui_hosts`
- `domain/ui/ui_accessibility`
- `domain/ui/ui_geometry`
- `domain/ui/ui_testing`

The current code does not expose final architecture contracts for:

- `UiProgram`
- `ControlGraph`
- `LayoutGraph`
- `StateGraph`
- `StyleGraph`
- `InteractionGraph`
- `BindingGraph`
- `VisualGraph`
- `AccessibilityGraph`
- `InspectionGraph`
- `ControlPackage` registry contracts
- `UiSchemaValue`
- `UiCompiler`
- `UiRuntimeArtifactManifest`
- `UiRuntimeArtifactTables`
- `UiEvaluator`
- final editor, game, world-space, and headless host contracts

## Required Reconciliation

`PT-UI-PROGRAM-ARCHITECTURE` must treat the final owner map as the target shape,
not as immediate folder creation.

The track must explicitly reconcile:

- `ui_tree` as retained tree compatibility until migrated into `UiProgram` and
  control package ownership.
- `ui_widgets` as retained widget compatibility until migrated into
  `ui_controls`.
- `ui_runtime` as retained runtime compatibility until replaced by compiler,
  artifact, and evaluator contracts.
- `ui_layout` as a current owner that must be mapped into `LayoutGraph`, layout
  kernels, artifact tables, evaluator behavior, or retained compatibility.
- `ui_math` as a current owner that may migrate toward `ui_geometry` only
  through an explicit compatibility and deprecation plan.
- `ui_surface` as a final owner for surface identity, mounting, projection,
  visibility, lifetime, and presentation contracts.

## Downstream Gates

MaterialProgram proof planning remains blocked by default until the UiProgram
architecture implementation truth claim is satisfied or a later accepted ADR
explicitly changes that gate.

Shared `foundation/meta` extraction remains blocked until UI and
MaterialProgram both prove the same domain-agnostic primitive and a separate
extraction design accepts it.
