---
title: Runenwerk UI Story V2 Consumer and Proof Boundary
description: Current Runenwerk-local story manifest, workflow, report, gallery/CLI, proof, and mount-decision boundary without claiming reusable-framework authority.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../implemented/ui-program-architecture.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../domain/ui/story-acceptance-and-review-checklist.md
  - ../../architecture/ui-framework-architecture.md
---

# Runenwerk UI Story V2 Consumer and Proof Boundary

## Status

This document records the current **Runenwerk-local** Story V2 proof and consumer
boundary implemented by `domain/ui/ui_story` and its current gallery/CLI
consumers.

It is not a future reusable-framework specification. Standalone
[`dornglut/runen-ui`](https://github.com/dornglut/runen-ui) owns future reusable
UI-framework semantics, testing architecture, host profiles, renderer contracts,
controls, and production maturity. This document does not claim that Runenwerk
has adopted RunenUI.

## Decision

Runenwerk keeps its current local story system as proof and product-consumer
infrastructure until a separately authorized consumer cutover replaces a named
boundary.

The current story model is V2 and workflow-graph based. The former flat
`UiStoryRunReport` / fixed-stage architecture is not current API authority.

Current shape:

```text
UiStoryManifestV2
  -> UiStoryRegistryV2
  -> selected UiStoryWorkflowProfileV2
  -> UiStoryRunV2 / owner-produced workflow evidence
  -> UiStoryWorkflowReportV2
  -> UiStoryMountDecisionV2
  -> gallery / CLI / host consumer
```

## Manifest V2

`UiStoryManifestV2` identifies one local story and selects the proof workflow
that applies to it. Current manifest facts include:

```text
schema version
story id
story revision
title
category id
source
program id
host profile id
theme profile id
viewport matrix
workflow profile id
expected outcome
mount policy
```

The manifest selects a workflow profile. It does not define one universal flat
stage list for every story.

## Workflow profiles

Current built-in workflow profiles include:

```text
ui_story.workflow.source_load_only
ui_story.workflow.compiler_only
ui_story.workflow.static_preview
ui_story.workflow.executable_interaction_proof
```

The profile owns the graph shape and required dependencies between proof nodes.
Different story kinds may therefore prove different bounded contracts without
inventing a second runner or pretending every consumer has identical stages.

### Static preview profile

The current static-preview graph includes the local chain:

```text
manifest
  -> source_load
  -> source_parse
  -> program_formation
  -> compiler
  -> runtime_view
  -> render_primitives
  -> render_data
  -> static_mount
  -> preview_frame
```

### Executable interaction profile

The executable interaction proof extends the compiled/runtime path with
interaction evidence:

```text
manifest
  -> source_load
  -> source_parse
  -> program_formation
  -> compiler
  -> runtime_view
  -> interaction_story
       -> interaction_replay
       -> live_interaction_proof
  -> replay_live_parity
  -> interaction_static_mount
  -> preview_frame
```

These graphs are current Runenwerk implementation facts. They are not a promise
that standalone RunenUI uses the same public API or internal decomposition.

## Evidence ownership

`ui_story` orchestrates proof. It does not take semantic ownership away from the
crate that produces the evidence.

Examples:

- `ui_definition` owns source validation/normalization facts;
- `ui_program_lowering` owns local program-formation facts;
- `ui_compiler` / `ui_artifacts` own compilation and artifact facts;
- `ui_runtime_view` / current runtime owners own runtime-read-model facts;
- `ui_render_primitives`, `ui_render_data`, and static/headless proof owners own
  their derived output;
- app/editor/game owners retain product state, commands, mutation, and effects.

Application-owned or host-owned evidence attaches to workflow nodes. Story
orchestration does not convert those facts into `ui_story` semantic ownership.

## Workflow report V2

`UiStoryWorkflowReportV2` is the current local aggregate proof report. It records
at least the workflow graph, node reports/outcomes, diagnostics, expected-failure
matching, aggregate outcome, and first blocker.

The report is a proof/inspection product. It is not authored UI truth, product
state, renderer truth, or a generic reusable-framework conformance standard.

## Expected failures

Expected-failure matching is explicit. A failure story is useful proof only when
the observed failing node/diagnostic matches the declared expectation. An
expected failure is not mount permission.

## Mount decision V2

`UiStoryMountDecisionV2` is fail-closed.

Mount is blocked when any required workflow condition is invalid or incomplete,
including a failed workflow, an expected-failure story, a `Never` or
`GalleryOnly` mount policy for production mounting, missing/failed required
preview proof, or a non-passed aggregate outcome.

The current positive rule is deliberately narrow:

```text
mount_policy == EligibleWhenPassed
AND required preview proof passed
AND workflow outcome == Passed
-> locally mount-eligible
```

Local mount eligibility does not bypass app/engine policy, route/capability
checks, or product mutation ownership.

## Gallery and CLI consumers

Runenwerk's gallery and CLI consume the same V2 story authority rather than
maintaining a second button-specific proof model.

Current local responsibilities include:

- checked-in story assets and registry discovery;
- V2 workflow execution;
- CLI summary/inspection projection;
- gallery preview and inspection;
- expected-failure diagnostics;
- local mount-decision reporting;
- deterministic proof data used by tests and review.

The editor/app remains a consumer. Gallery UI does not become reusable control,
renderer, host, or product-state authority merely because it visualizes story
evidence.

## Local adoption boundary

The local `ui_story` crate may continue supporting existing Runenwerk code while
Runenwerk still owns those consumers. New work must not use this document to
expand Runenwerk into a second reusable UI framework.

Future reusable story/testing semantics belong to RunenUI. A future Runenwerk
cutover must be re-derived against the exact accepted RunenUI revision at that
time and must be owned by a new issue naming the concrete consumer path being
replaced.

## Product-specific consumers

Graph editors, timelines, progression systems, game HUDs, world-space UI, visual
design tools, and other products keep their product/domain semantics outside
`ui_story`.

Story proof may be consumed by a product where the current local code requires
it. That does not make this document authority for future generic NodeCanvas,
TrackSurface, transition/effect, platform, accessibility, renderer, or
virtualization framework semantics.

## Non-goals

This document does not authorize:

- standalone RunenUI adoption or dependency changes;
- recreating RunenUI's roadmap inside Runenwerk;
- the retired flat `UiStoryRunReport` API;
- a universal fixed stage list for every story;
- new generic framework controls or platform targets;
- direct host/app/editor/game mutation from generic UI;
- renderer-owned UI semantics;
- a second gallery/CLI runner;
- mounting when V2 workflow/mount proof fails.

## Validation boundary

Current code/tests are the behavior authority for the exact V2 APIs. The
[Story Acceptance and Review Checklist](../../domain/ui/story-acceptance-and-review-checklist.md)
summarizes the local review obligations without redefining the reusable
framework.

If a future change needs reusable framework behavior rather than Runenwerk-local
consumer behavior, start from standalone RunenUI authority instead of extending
this design.
