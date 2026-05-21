---
title: WR-009 PM-RENDER-PG-006 Native Multi-Window Editor Presentation Plan
description: Promotion and implementation-readiness contract for the PM-RENDER-PG-006 native multi-window and multi-surface presentation slice.
status: active
owner: editor
layer: app / engine-runtime / render-runtime
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/editor-native-multi-window-presentation-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/product-surface-platform-hardening-design.md
  - ../../../design/implemented/editor-workspace-document-mode-panel-architecture.md
  - ../../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../../apps/runenwerk-editor/roadmap.md
  - ../../../engine/roadmaps/render-final-architecture-migration.md
---

# WR-009 PM-RENDER-PG-006 Native Multi-Window Editor Presentation Plan

## Goal

Promote and implement `PM-RENDER-PG-006` as one bounded native multi-window
and multi-surface presentation slice.

The slice turns the accepted editor native multi-window design into a real
presentation path:

```text
EditorWindowId
  -> NativeWindowId
  -> RenderSurfaceId
  -> surface-scoped prepared frame
  -> submit/present into that surface only
```

The app owns the binding between editor windows, native windows, and render
surfaces. The editor domain owns logical workspace/window state. The engine
runtime owns native window records and window-scoped platform events. The
render runtime owns render surfaces, swapchains, surface-scoped prepare,
submit, present, resize, loss recovery, and diagnostics.

The renderer remains an execution and presentation layer. It must not own
product truth, product selection, freshness, authority, fallback legality,
rebuild policy, residency policy, document semantics, material truth, or editor
workspace policy.

## Source Of Truth

- Production milestone: `PM-RENDER-PG-006`.
- Bounded implementation row: `WR-009`.
- Accepted PM-006 design:
  `docs-site/src/content/docs/design/accepted/editor-native-multi-window-presentation-design.md`.
- Boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- Product-surface prerequisite closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-005-product-surface-platform-hardening/closeout.md`.
- `WR-003` remains support-only context. It is not the PM-006 implementation
  row.

## Readiness

`task production:plan -- --milestone PM-RENDER-PG-006 --roadmap WR-009`
reported after design acceptance and exact blocker metadata repair:

- milestone state: `ready_next`;
- WR state: `ready_next`;
- dependency `WR-003:support_only`;
- next action: `write_promotion_contract`;
- promotion preflight: `promotable`.

The production design gate is accepted, but the original WR-009 write scopes
were too narrow for the accepted design. The contract therefore updates the WR
metadata before promotion so implementation can legally touch the required
editor-domain, editor-app, engine-runtime, and render-runtime owners.

Promotion command, after this contract is linked and validation passes:

```text
task roadmap:promote -- --id WR-009 --state current_candidate --evidence "Accepted PM-RENDER-PG-006 native multi-window presentation design and active promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-009-native-multi-window-editor-presentation/plan.md"
```

Do not promote if validation fails, if source files changed enough that
`task ai:goal -- --track PT-RENDER-PG` must be rerun, or if another current
candidate blocks promotion and the workflow requires
`task roadmap:switch-current`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "PM-RENDER-PG-006 native multi-window and multi-surface presentation" --scope "domain/editor/editor_shell, apps/runenwerk_editor, engine/src/runtime, engine/src/app, engine/src/plugins/render"
```

Before product code changes, inspect the named primary docs and enforce the
accepted ownership boundary in this contract. Add an ADR only if implementation
needs to change global runtime/window ownership policy, platform event
ownership policy, or render-surface ownership beyond the accepted design.

## Implementation Scope

Owned areas:

```text
domain/editor/editor_shell
apps/runenwerk_editor
engine/src/runtime
engine/src/app
engine/src/plugins/render
engine/tests
apps/runenwerk_editor/tests
docs-site/src/content/docs/design/accepted/editor-native-multi-window-presentation-design.md
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/engine/roadmaps/render-final-architecture-migration.md
docs-site/src/content/docs/engine/plugins/render/docs/roadmap.md
docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected implementation modules or nearby owners:

```text
domain/editor/editor_shell/src/workspace/window.rs
domain/editor/editor_shell/src/workspace/state.rs
domain/editor/editor_shell/src/workspace/reducer.rs
domain/editor/editor_shell/src/commands/shell_command.rs
domain/editor/editor_shell/src/composition/build_toolbar.rs
apps/runenwerk_editor/src/shell/state.rs
apps/runenwerk_editor/src/shell/dispatch_shell_command.rs
apps/runenwerk_editor/src/shell/controller.rs
apps/runenwerk_editor/src/persistence/workspace_layout.rs
apps/runenwerk_editor/src/runtime/viewport
engine/src/runtime/window.rs
engine/src/runtime/platform.rs
engine/src/app
engine/src/plugins/render/backend/surface.rs
engine/src/plugins/render/frame/packet.rs
engine/src/plugins/render/frame/view.rs
engine/src/plugins/render/runtime/frame_prepare.rs
engine/src/plugins/render/runtime/frame_submit.rs
engine/src/plugins/render/renderer/mod.rs
```

Use nearby module names if implementation shows a better local fit, but keep
the ownership boundary explicit. Do not create catch-all helper files.

## Required Contracts

The implementation must add or refine typed, inspectable contracts for:

- logical editor window identity and workspace-root ownership;
- app-owned `EditorWindowId -> NativeWindowId -> RenderSurfaceId` binding;
- native runtime window registry keyed by engine-owned native window ids;
- window-scoped platform events, focus, cursor, redraw, close, resize, and DPI;
- render surface registry keyed by render surface/native window identity;
- surface-scoped prepared render frames or packets;
- surface-scoped submit and present APIs that cannot silently target the wrong
  surface;
- per-surface resize, surface-loss recovery, and diagnostics;
- explicit UI-frame routing from editor window identity into a render surface;
- viewport/product presentation proof that prepared product surfaces are
  presented in the correct window without moving product ownership.

Product-surface manifests, prepared views, dynamic targets, uploads, aliases,
history, and product diagnostics remain product/producer-owned and use the
PM-002 through PM-005 contract path. Native windows select presentation
surface, not product truth.

## Implementation Steps

1. Add editor-domain logical window identity, records, commands, reducer
   transitions, and focused tests under
   `domain/editor/editor_shell/src/workspace` and
   `domain/editor/editor_shell/src/commands`.
2. Add app-owned window binding state in
   `apps/runenwerk_editor/src/shell/state.rs` and command dispatch in
   `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` without placing
   native handles in editor-domain state.
3. Add an engine-runtime native window registry in
   `engine/src/runtime/window.rs` and window-scoped platform events in
   `engine/src/runtime/platform.rs`.
4. Route redraw, close, cursor, resize, scale factor, focus, and input through
   native window identity.
5. Add render surface registry/state in
   `engine/src/plugins/render/backend/surface.rs` so one native window maps to
   one render surface/swapchain lifecycle.
6. Make frame prepare, packets, submit, and present surface-scoped in
   `engine/src/plugins/render/frame/packet.rs`,
   `engine/src/plugins/render/runtime/frame_prepare.rs`,
   `engine/src/plugins/render/runtime/frame_submit.rs`, and
   `engine/src/plugins/render/renderer/mod.rs`.
7. Add diagnostics that identify native window id, render surface id, frame id,
   failure kind, and message for cross-surface misuse, missing surfaces,
   resize/loss recovery, and submit/present target mismatch.
8. Wire `Window > New Window` through the editor app controller so a secondary
   native window renders shell UI and closes independently of project shutdown.
9. Prove one viewport/product surface in each of two windows can route to the
   correct surface without changing product truth, freshness, fallback,
   authority, rebuild, or residency policy.
10. Update render/editor roadmap and reference docs for the final
    multi-surface ownership model.
11. After validation passes, create closeout evidence and only then update
    `PM-RENDER-PG-006` completion metadata.

## Explicit Non-Goals

Do not implement:

- render fragments, fragment hot reload, fragment merge provenance, or
  last-good fragment fallback from `PM-RENDER-PG-007`;
- production-readiness capture/replay policy, performance budgets, final
  renderer examples, or broad inspection closeout from `PM-RENDER-PG-008`;
- product truth, product selection, source lineage, freshness, authority,
  fallback legality, rebuild policy, dependency truth, material truth, drawing
  truth, or residency policy;
- broad product-surface hardening beyond the completed PM-005 contract;
- renderer-owned editor/window policy shortcuts;
- renderer-private backend handles in editor-domain state, product producers,
  viewport systems, or preview systems;
- full workspace persistence redesign beyond the multi-window placement/layout
  data required by this milestone;
- monitor enumeration beyond the minimal policy needed for `Window > New
  Window`, unless the existing runtime abstraction already exposes it cleanly.

## Acceptance Criteria

- `Window > New Window` opens a real native OS window.
- Two editor windows can edit the same project/session.
- Each window has independent workspace focus, UI frame, input capture, native
  window state, render surface, swapchain, DPI scale, cursor state, redraw
  state, and close/surface-loss lifecycle.
- Closing a secondary window removes only that window host/layout state and
  does not close the project.
- Prepared render frames, backend surface state, submit, and present are scoped
  by native window or render surface identity.
- Submit and present cannot silently target the wrong surface.
- A viewport/product surface in each of two native windows binds to the correct
  presentation surface while product targets remain viewport/product-owned.
- Generic engine runtime and render APIs do not depend on editor concepts.
- Editor-domain state does not store native handles, swapchains, backend
  surface objects, or render-runtime private handles.
- No out-of-scope fragment, production-readiness, product-policy, material
  lowering, or renderer-owned semantic shortcut lands in WR-009.

## Validation

Focused implementation validation must include:

```text
cargo test -p editor_shell multi_window
cargo test -p engine window
cargo test -p engine --test render_multi_surface
cargo test -p runenwerk_editor window
cargo test -p runenwerk_editor --test viewport_architecture_guards
```

Add or extend tests for:

- logical editor window identity, focus, close, move, and duplicate commands;
- absence of native handles in editor-domain state;
- native runtime window registry and window-scoped platform events;
- window-scoped input, cursor, close, resize, scale factor, and redraw routing;
- render surface registry lifecycle and surface-loss isolation;
- surface-scoped prepare, submit, present, and diagnostics;
- cross-surface mismatch rejection;
- viewport/product presentation in two native windows without product-policy
  inference;
- secondary-window close preserving project/session state.

GPU/runtime proof required for final closeout when host support is available:

```text
RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored
```

If GPU smoke cannot run in the current environment, closeout must state that
limitation and include the strongest deterministic multi-surface tests that did
run.

Workflow validation:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Stop Conditions

Stop and report instead of coding if:

- `task ai:goal -- --track PT-RENDER-PG` no longer selects PM-006/WR-009;
- `task production:plan -- --milestone PM-RENDER-PG-006 --roadmap WR-009`
  no longer reports a promotable or actionable row;
- promotion fails for anything other than exact metadata repair or a required
  current-candidate switch;
- implementation needs a new ADR for global runtime/window ownership,
  platform-event ownership, or render-surface ownership policy;
- implementation needs product truth, product policy, material lowering,
  fragment assets, production-readiness capture/replay, or broad renderer
  inspection;
- engine/runtime APIs would need to depend on editor-domain concepts;
- editor-domain state would need native handles, swapchains, backend surface
  objects, or renderer-private handles;
- validation fails and cannot be repaired inside the bounded WR-009 scope.

## Closeout Requirements

Closeout evidence must be created only after implementation and validation
pass.

Closeout path:

```text
docs-site/src/content/docs/reports/closeouts/pm-render-pg-006-multi-surface-presentation/closeout.md
```

After closeout:

- archive `WR-009` with completed evidence;
- add the closeout path to WR-009 write scopes before archival;
- update `PM-RENDER-PG-006` evidence gates, completion audit, and completion
  quality;
- rerun roadmap, production, docs, planning, and goal validation.

## Perfectionist Closeout Audit

Expected completion quality is `runtime_proven` only if closeout includes
runtime/GPU or equivalent native-window evidence proving two real native
surfaces can render independently without cross-surface presentation mistakes.

Use `bounded_contract` if the slice lands typed multi-surface contracts and
deterministic tests but cannot prove the native runtime/GPU path in the current
environment. Do not claim `perfectionist_verified` while PM-RENDER-PG-007 and
PM-RENDER-PG-008 remain incomplete.

Known quality gaps expected at closeout unless later evidence proves otherwise:

- PM-RENDER-PG-007 still owns render fragments and hot reload.
- PM-RENDER-PG-008 still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
- PM-006 must not move product truth or product policy into the renderer.
