---
title: Runenwerk UI Story Acceptance and Review Checklist
description: Acceptance, proof, and review checklist for the story-driven UI golden workflow and future UI platform components.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-16
related_designs:
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../design/active/runenwerk-ui-platform-capability-roadmap.md
  - ../../design/active/ui-runtime-rendering-pipeline-roadmap.md
related_docs:
  - ./architecture.md
  - ./roadmap.md
---

# Runenwerk UI Story Acceptance and Review Checklist

## Purpose

This checklist defines the quality bar for Runenwerk UI stories, components,
surfaces, advanced platform controls, gallery inspection, and mount eligibility.

A UI artifact is not complete because it renders. It is complete only when the
story workflow proves its authored source, program formation, runtime artifact,
runtime view, bindings, routes, layout, style, text, accessibility,
interaction, render primitives, render data, static mount, diagnostics, and
mount policy.

## Global Acceptance Rule

Every UI unit must pass through `UiStoryRunReport`.

Accepted units:

- primitive controls;
- compound components;
- full surfaces;
- state stories;
- failure stories;
- interaction stories;
- layout stories;
- accessibility stories;
- host-profile stories;
- advanced platform controls such as GraphCanvas and Timeline.

No direct mounting without story proof.

## Story Manifest Checklist

A story manifest is acceptable when it defines:

- [ ] stable `story_id`;
- [ ] human `title`;
- [ ] `category`;
- [ ] `source_kind` as `node` or `template`;
- [ ] `source_path`;
- [ ] `source_id`;
- [ ] `program_id`;
- [ ] `control_package`;
- [ ] host kind/profile;
- [ ] theme profile;
- [ ] viewport matrix;
- [ ] pass/fail expectation;
- [ ] mount policy;
- [ ] diagnostic expectations for failure stories;
- [ ] route map or explicit route policy;
- [ ] host data if bindings are used;
- [ ] state profiles if the component is interactive;
- [ ] input traces if the component is interactive;
- [ ] accessibility policy.

## Source Checklist

Authored source is acceptable when:

- [ ] it is parsed through the declared `source_kind`;
- [ ] template stories use `AuthoredUiTemplate`;
- [ ] component/control stories use `UiNodeDefinition` or an approved
  equivalent;
- [ ] source identity is stable;
- [ ] source maps attach to all rows that claim source identity;
- [ ] no runtime widget IDs, ECS IDs, renderer IDs, or app command
  implementations leak into authored source;
- [ ] source declares intent, not renderer execution.

## Definition And Schema Checklist

Definition/schema stage is acceptable when:

- [ ] template IDs are valid;
- [ ] node IDs are valid;
- [ ] duplicate IDs fail;
- [ ] invalid child counts fail;
- [ ] invalid menus/focus/availability records fail;
- [ ] unknown control kinds fail formation;
- [ ] unknown properties fail by default;
- [ ] missing required properties fail;
- [ ] invalid enum values fail;
- [ ] non-finite numeric values fail;
- [ ] route refs validate as route refs;
- [ ] diagnostics include source path, story ID, stage, stable code, and
  actionable message.

## Control Package Checklist

A control package contribution is acceptable when it provides:

- [ ] property schema;
- [ ] state schema;
- [ ] event payload schema;
- [ ] layout kernel;
- [ ] interaction kernel;
- [ ] visual kernel;
- [ ] accessibility kernel;
- [ ] inspection kernel;
- [ ] migration hook;
- [ ] package ID;
- [ ] package version;
- [ ] stable control kind ID;
- [ ] package diagnostics.

Missing schema or kernel data must fail catalog derivation.

## Program Formation Checklist

`UiProgramFormationReport` is acceptable when:

- [ ] it receives an explicit `ControlPackageRegistrySnapshot`;
- [ ] it does not infer package truth from strings;
- [ ] unknown control kinds fail closed;
- [ ] skipped control kinds are reported;
- [ ] invalid properties stop usable rows from being treated as renderable;
- [ ] graph-family rows are source-mapped;
- [ ] combined diagnostics are carried into the story report.

## Compiler And Artifact Checklist

Compiler output is acceptable when:

- [ ] `UiCompilerReport` passes;
- [ ] package resolution diagnostics pass;
- [ ] capability diagnostics pass;
- [ ] graph integrity diagnostics pass;
- [ ] `UiRuntimeArtifact` is produced;
- [ ] artifact tables are inspectable;
- [ ] manifest rows include package/control/schema/route/kernel/capability/source-map
  identity;
- [ ] artifact snapshots are deterministic where required.

## Runtime View Checklist

Runtime view is acceptable when:

- [ ] it derives from `UiRuntimeArtifact` plus declared host data;
- [ ] it does not reread authored source for runtime facts;
- [ ] it exposes IDs, labels, state, routes, capabilities, style axes,
  accessibility data, source maps, and diagnostics;
- [ ] missing/stale host data is diagnostic-bearing;
- [ ] control-specific views are optional inspector adapters, not the primary
  gallery pipeline.

## Binding And Host Route Checklist

Bindings/routes are acceptable when:

- [ ] every binding references declared host data or an approved missing-data
  failure case;
- [ ] dirty propagation is deterministic;
- [ ] authorization failures produce diagnostics;
- [ ] every real host route maps through `ui_hosts` or host/app route tables;
- [ ] route proposals carry payload schema and capability information;
- [ ] generic UI never mutates host/domain/app state directly.

## Layout, Style, Text, And Accessibility Checklist

Layout is acceptable when:

- [ ] bounds are computed;
- [ ] constraints are visible;
- [ ] clipping/scroll parents are visible;
- [ ] overflow is visible;
- [ ] failed constraints are diagnostic-bearing.

Style is acceptable when:

- [ ] semantic tokens resolve;
- [ ] missing tokens fail or follow explicit fallback policy;
- [ ] raw values are only inside token/theme definitions unless explicitly
  marked as test fixture values;
- [ ] renderer does not hardcode product UI colors.

Text is acceptable when:

- [ ] text layout requests/results are inspectable;
- [ ] text measurement affects layout where relevant;
- [ ] glyph runs or text primitives are deterministic;
- [ ] overflow/wrap/ellipsis are visible;
- [ ] missing glyph/localization expansion issues are diagnostic-bearing.

Accessibility is acceptable when:

- [ ] interactive controls produce accessibility nodes;
- [ ] roles are valid;
- [ ] labels are resolved;
- [ ] focusability is inspectable;
- [ ] invalid accessibility records fail;
- [ ] visible text can be proven as accessible name when explicit label is
  omitted.

## Interaction Checklist

Interactive UI is acceptable when stories prove:

- [ ] pointer hover;
- [ ] pointer press/release;
- [ ] keyboard activation;
- [ ] focus traversal;
- [ ] gamepad activation if game-facing;
- [ ] disabled state blocks activation;
- [ ] modal/popover behavior where relevant;
- [ ] drag/drop lifecycle where relevant;
- [ ] emitted route proposals are schema-valid;
- [ ] interaction does not directly mutate host/domain/app state.

## Render And Static Mount Checklist

Rendering proof is acceptable when:

- [ ] render primitives are derived from runtime views/artifacts;
- [ ] invalid runtime views produce diagnostics, not guessed primitives;
- [ ] render primitive output is deterministic;
- [ ] render data preserves provenance;
- [ ] static mount passes;
- [ ] frame has required surfaces/layers/primitives;
- [ ] stable draw order is proven;
- [ ] the visible frame is not treated as success if upstream reports failed.

## Component Completion Checklist

A component is complete only when it has:

- [ ] control package contract;
- [ ] schema validation;
- [ ] runtime view projection;
- [ ] binding/route behavior if applicable;
- [ ] layout/style/text/accessibility behavior;
- [ ] interaction behavior if applicable;
- [ ] render primitive lowering if visual;
- [ ] default story;
- [ ] state stories;
- [ ] failure stories;
- [ ] interaction traces;
- [ ] visual snapshots where required;
- [ ] docs page;
- [ ] validation tests.

## Surface Completion Checklist

A surface is complete only when it has:

- [ ] authored template;
- [ ] surface story manifest;
- [ ] host profile;
- [ ] route map;
- [ ] host data contract;
- [ ] viewport matrix;
- [ ] theme profile;
- [ ] accessibility proof;
- [ ] interaction proof;
- [ ] render/static mount proof;
- [ ] mount eligibility pass;
- [ ] docs page or owning design reference.

## Advanced Platform Component Checklist

GraphCanvas, Timeline, CodeEditor/RichText, drag/drop, world-space UI, UI
effects, and visual builder work must satisfy all preceding checks plus:

- [ ] reusable platform ownership;
- [ ] no product/domain-specific semantics in generic UI;
- [ ] story matrix covering normal, empty, large, invalid, interaction,
  accessibility, and performance states;
- [ ] clear host/domain mutation boundary;
- [ ] renderer contract only if existing primitives cannot express the visual
  output;
- [ ] no parallel one-off editor implementation.

## Review Checklist

A review passes only when:

- [ ] exact files and methods changed are listed;
- [ ] every new crate has accepted authority;
- [ ] generated docs are not edited directly;
- [ ] active design index is updated if a new active design is added;
- [ ] story manifests are assets, not Rust constants;
- [ ] story runner is domain-owned;
- [ ] gallery consumes `UiStoryRunReport`;
- [ ] CLI and gallery share the same runner;
- [ ] validators reject legacy/bypass paths;
- [ ] tests cover passing and failing stories;
- [ ] validation commands are recorded with results.

## Stop Conditions

Stop and redesign if:

- [ ] implementation renders directly from `.ron`;
- [ ] renderer owns UI semantics;
- [ ] hardcoded gallery fixtures remain as a production path;
- [ ] button-specific gallery pipeline remains primary;
- [ ] story manifests are bypassed for mountable surfaces;
- [ ] visual builder writes a second format;
- [ ] GraphCanvas/Timeline duplicate drag/selection/focus systems;
- [ ] generic UI mutates app/editor/game/domain state directly;
- [ ] failed upstream reports still produce claimed-success visible output.
