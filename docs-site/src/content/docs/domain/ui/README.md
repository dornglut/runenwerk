---
title: UI Domain
description: Documentation index for Runenwerk-local UI substrate, runtime, program, proof, and integration semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-25
---

# UI Domain

`domain/ui/*` owns the UI implementation and contracts that currently live in
Runenwerk: substrate/foundation crates, authored definition/formation contracts,
retained runtime layers, UiProgram-era program/artifact/proof crates, and
renderer-facing UI products.

This is **Runenwerk-local implementation authority**, not the future reusable UI
framework for the Dornglut repository family. Standalone
[`dornglut/runen-ui`](https://github.com/dornglut/runen-ui) owns future reusable
framework semantics. Existing Runenwerk UI remains valid until an explicit
consumer cutover replaces a named boundary.

Runenwerk currently has coexisting local execution/program paths:

```text
Authored UI / editor definitions
  -> validation / normalization
  -> formed interaction contracts
  -> retained UI and/or UiProgram-era local consumers
  -> local story/proof and host integration
  -> renderer/product-surface output
```

Renderer output is derived product data. It is not UI authority.

## Canonical Local Architecture

Start with:

- [Runenwerk UI Local Runtime and Integration Architecture](../../architecture/ui-framework-architecture.md)

That page is the top-down Runenwerk-local ownership/adoption spine. For future
reusable framework architecture and maturity, use standalone RunenUI's
[architecture](https://github.com/dornglut/runen-ui/blob/main/ARCHITECTURE.md),
[status](https://github.com/dornglut/runen-ui/blob/main/docs/status.md), and
[roadmap](https://github.com/dornglut/runen-ui/blob/main/docs/roadmap.md).

This page remains the UI-domain landing page and current docs router.

## Source Of Truth Order

1. Current Runenwerk code and executable tests own local behavior.
2. Accepted ADRs and lifecycle-owned local designs own durable Runenwerk decisions.
3. [Runenwerk UI Local Runtime and Integration Architecture](../../architecture/ui-framework-architecture.md)
   owns the local top-down integration/adoption boundary.
4. [Current Architecture](./architecture.md) records detailed local code truth.
5. [Roadmap](./roadmap.md) records durable local sequence/dependencies only.
6. Standalone RunenUI owns future reusable-framework semantics and maturity.
7. Historical plans, closeouts, reports, and superseded designs are evidence,
   not live framework authority.

## Current UI Truth

- [UI Definition Usage](./ui-definition-usage.md)
- [UI Composition Usage](./ui-composition-usage.md)
- [Adaptive Composition Usage](./ui-adaptive-composition-usage.md)
- [Current Architecture](./architecture.md)
- [Roadmap](./roadmap.md)
- [Story V2 Acceptance and Review Checklist](./story-acceptance-and-review-checklist.md)

## Story Workflow

- [Runenwerk UI Story V2 Consumer and Proof Boundary](../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md)
- [Story V2 Acceptance and Review Checklist](./story-acceptance-and-review-checklist.md)

The current Runenwerk story system is V2/workflow-graph based. It is local
proof/consumer infrastructure; it is not a second reusable-framework testing
roadmap.

## Formation / Source Model

- [UI Definition Formation Framework Design](../../design/implemented/ui-definition-formation-foundation-design.md)
- [Editor Self-Authoring and UI Workspace Design](../../design/implemented/editor-self-authoring-and-final-ui-design.md)

## Structural Composition

- [App-Neutral UI Composition Design](../../design/accepted/app-neutral-ui-composition-design.md)
- [ADR 0013: App-Neutral UI Composition Clean Cutover](../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md)
- [UI Composition Usage](./ui-composition-usage.md)
- [Adaptive Composition Usage](./ui-adaptive-composition-usage.md)

`domain/ui/ui_composition` is the app-neutral structural authority for saved
composition definitions, ratified mutable structure, typed transactions,
structural history, explicit promotion, content liveness vocabulary,
deterministic linked persistence bundles, atomic generation activation, and
headless fixture contracts. It does not own adaptive projection, app-extension
meaning, storage-root policy, native windows, product content, or editor
semantics.

`domain/ui/ui_adaptive_composition` derives transient projection, reflow,
hit-test, preview, drag/resize-session, proposal, accessibility, and explicit
promotion-delta products from immutable composition snapshots. It cannot mutate
or persist canonical composition state.

Editor and Draw structure project through `ui_composition`. Current editor
structural commands commit through the composition transaction path; legacy
`WorkspaceState` structures remain compatibility/test or migration inputs where
current source still retains them, not a parallel live structural authority.
`ui_surface` remains a temporary predecessor compatibility boundary rather than a
second structural target owner. ADR 0013 still requires its deletion after each
maintained responsibility is mapped to RunenUI or the actual Runenwerk app/domain
owner; it is not a durable framework boundary.

## RunenUI Adoption Classification

The accepted adoption architecture distinguishes retained Runenwerk semantics
from predecessor reusable-runtime authority.

Retained Runenwerk authority includes `ui_composition`,
`ui_adaptive_composition`, authored `ui_definition` / `ui_schema`, the
UiProgram/lowering/compiler/artifact toolchain, Story V2 proof policy, and
product/editor mutation semantics.

Predecessor framework authority includes the local retained
`ui_tree`/`ui_widgets`/`ui_runtime` path and overlapping reusable
math/input/layout/text/theme/geometry/state/accessibility/testing semantics.
These remain current implementation authority only until a named consumer moves
cleanly to standalone RunenUI.

Mixed packages such as `ui_controls`, `ui_binding`, `ui_hosts`,
`ui_surface`, `ui_evaluator`, `ui_runtime_view`, renderer-neutral output
packages, and `engine::plugins::ui` are split by semantic responsibility
during adoption. They are not retained wholesale merely because current code
uses them.

The canonical classification and consumer sequence live in
[Runenwerk UI Local Runtime and Integration Architecture](../../architecture/ui-framework-architecture.md).

## Interaction / Runtime

- [Editor UI Runtime V2 and Interaction Formation Design](../../design/implemented/editor-ui-runtime-v2-and-interaction-formation-design.md)
- [ADR 0009: UI Interaction Formation V2](../../adr/accepted/0009-ui-interaction-formation-v2.md)

## UiProgram / Artifact Path

- [UI Program Architecture](../../design/implemented/ui-program-architecture.md)
- [UI Program Architecture Owner Map](../../design/implemented/ui-program-architecture-owner-map.md)

The UiProgram-era crates coexist with retained `ui_tree`, `ui_widgets`, and
`ui_runtime`; their existence does not imply total retained-UI replacement.

## Editor Coordination And Legacy Predecessors

Current normalized editor coordination authority:

- [ADR 0025: Normalize Editor Coordination and Semantic Ownership](../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md)
- [Runenwerk Editor Coordination Semantic Model](../../design/accepted/runenwerk-editor-coordination-semantic-model.md)
- [Editor Tool Suite Registry And Workbench Host Design](../../design/implemented/editor-tool-suite-registry-and-workbench-host-design.md)

The former workspace/document-mode and tool-surface predecessor designs are
retired from the live corpus. Their normalized current authority is [ADR
0025](../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md)
and its [companion semantic model](../../design/accepted/runenwerk-editor-coordination-semantic-model.md).

## Deferred Execution Targets

- [UI Model Multiple Execution Strategies Design](../../design/deferred/ui-model-multiple-execution-strategies-design.md)

Deferred local execution ideas do not become reusable-framework authority. Any
future standalone RunenUI adoption is separately issue-owned and must start from
the exact then-current RunenUI revision.

## Interaction V2 Migration Spine

ADR 0009 makes Interaction V2 the shared Runenwerk guardrail for popup stack,
scroll ownership, focus, menu sizing, chrome slots, docking/drop-zone, status
overflow, and viewport input arbitration.

Every retained UI migration slice should flow through:

```text
definition vocabulary
  -> validation rule
  -> FormedInteractionModel record
  -> retained UI formation adapter
  -> ui_runtime enforcement
  -> editor/app guard
```

Historical shell-polish and popup/adornment records are supporting evidence. They
do not own long-term UI policy and cannot promote alternate execution targets.

The current retained UI slice catalog is:

- `IV2-menu-stack`
- `IV2-scroll-ownership`
- `IV2-menu-sizing`
- `IV2-chrome-slots`
- `IV2-dock-drop-zones`
- `IV2-status-and-viewport-arbitration`

Each slice is defined in
[Editor UI Runtime V2 and Interaction Formation Design](../../design/implemented/editor-ui-runtime-v2-and-interaction-formation-design.md)
and must be consumed as a contract by downstream retained UI work.

## Scope Boundary

`domain/ui` currently contains both retained Runenwerk authority and predecessor
reusable-runtime implementation. Retained authority includes app-neutral
structural composition (`ui_composition`), transient adaptive mechanism
(`ui_adaptive_composition`), general authored UI definition/schema formation,
and the UiProgram/artifact/Story proof families. The retained tree/runtime/widget
and generic surface/runtime packages remain current implementation only for
not-yet-migrated consumers and are not a second long-term reusable-framework
authority.

It does not own editor-shell product wording or app extensions, app runtime
wiring, provider behavior, app IO, native-window lifecycle, renderer execution
policy, concrete command execution, or future reusable-framework semantics.

Authored UI definitions must not persist runtime `WidgetId`, ECS entity ids,
app IO, provider state, provider behavior, or command execution. Editor command
semantics stay in editor/app owners and enter UI products only through explicit
route slots and ratified command paths.

## Standalone RunenUI boundary

Future reusable controls, production platform breadth, animation/motion,
virtualization, reusable renderer/platform integration, accessibility/text
maturity, framework devtools, and release qualification belong to standalone
RunenUI rather than a Runenwerk-local framework roadmap.

A future Runenwerk consumer cutover must be accepted separately and must leave
one mounted/runtime/interaction authority for that consumer. The first adoption
proof is intentionally headless and bounded: one maintained authored/UI-program
consumer should project through ordinary public RunenUI runtime/testing
contracts before engine, Draw, or Editor migration. Until a cut is accepted,
current Runenwerk-local code/tests remain behavior authority for that consumer.
