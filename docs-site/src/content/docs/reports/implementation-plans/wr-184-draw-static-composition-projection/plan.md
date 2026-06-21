---
title: Draw Static Composition Projection Cutover Implementation Plan
description: Decision-complete WR-184 contract for replacing Draw workspace projection authority with app-neutral composition structure and Draw-owned extension, content, and projection state.
status: accepted
owner: drawing
layer: report
canonical: false
last_reviewed: 2026-06-20
wr: WR-184
milestone: PM-UI-COMPOSITION-005
related_designs:
  - ../../../design/accepted/app-neutral-ui-composition-design.md
  - ../../../design/active/drawing-authoring-and-comic-layout-platform-design.md
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-184: Draw Static Composition Projection Cutover

## Authority And Promotion

PM-UI-COMPOSITION-004 and WR-183 are complete with runtime evidence. Region
Compass is the selected visual direction, but this checkpoint does not add its
chrome, docking, drag sessions, adaptive proposals, drawers, or cross-window
behavior. WR-184 proves that a real non-editor app consumes `ui_composition`
without depending on editor or adaptive code.

The generated draft contract was rejected because it expanded directory scopes
into unrelated editor and historical report files while omitting the Draw
runtime reader and integration tests. The governed scope now includes only the
Draw app composition boundary, the one runtime reader, the existing app-shell
test, one architecture guard, Draw docs, evidence, and closeout.

## Architecture Governance Decision

Recommendation: implement this exact cutover.

Ownership and dependency direction:

```text
ui_composition
  <- runenwerk_draw app/composition
  <- Draw app state, presentation, and runtime readers

drawing
  <- RunenwerkDrawApp document and stroke transactions
```

`ui_composition` owns structural topology and identity. `runenwerk_draw` owns
content roles, target sizing, canvas projection, provider/liveness state,
tablet diagnostics, and rendering. `drawing` continues to own document,
stroke, brush, layer, ratification, and drawing-history semantics. No
dependency points from the core into Draw, editor, engine, or windowing.

ADR 0013 already governs this ownership and clean cutover. Stop and create a
new ADR only if implementation requires moving drawing truth, native-window
ownership, adaptive behavior, or app content semantics into the core.

### ATAM-lite

Options evaluated:

1. Keep `DrawingWorkspaceProjection` as an alias over composition. Rejected:
   it retains the old public authority and creates cleanup debt.
2. Keep the current geometry constructor and attach an unused definition.
   Rejected: the composition graph would be decorative rather than causal.
3. Recurse the ratified region graph into bounds, then derive Draw canvas and
   content projection from app-owned roles. Selected: one structural source,
   no editor coupling, and a direct path to later adaptive projections.
4. Implement the existing `< 900px` support-panel hide in this checkpoint.
   Rejected: that is transient adaptive behavior and belongs to WR-187.

The accepted behavior change is deliberate: WR-184 exposes a static wide
profile at all sizes. It never silently hides a mounted region. Narrow-target
reflow, drawer conversion, and hide proposals remain WR-187 work.

## Current Authority Inventory

| Current location | Current authority | WR-184 replacement |
|---|---|---|
| `app/workspace.rs::DrawingWorkspaceProjection::canvas_first` | hard-coded topology, geometry, canvas projection, and narrow hide rule | delete file; ratified definition plus pure Draw projection |
| `app/state.rs::RunenwerkDrawApp::workspace` | live static layout DTO | `DrawingCompositionRuntime` plus `DrawingCompositionProjection` |
| `app/state.rs::workspace()` | public legacy authority accessor | remove; expose `composition_runtime()` and `composition_projection()` |
| `app/presentation.rs::build_workspace_frame*` | renders from workspace DTO | composition-named frame builders render from Draw projection |
| `runtime/ink.rs` | reads canvas bounds through `workspace()` | read `composition_projection()` |
| `tests/app_shell.rs` | validates public workspace API | validate public composition API and unchanged drawing behavior |

No drawing-domain writer changes. `DrawingTransaction`, `DrawingCommand`, and
the document revision remain the only committed drawing mutation path.

## Runtime And Four-Part State Model

WR-184 uses two parts of the governed state model directly:

- `CompositionDefinitionV1`: built-in saved/authored wide Draw layout;
- `CompositionState`: ratified structural source held by
  `DrawingCompositionRuntime`.

`AdaptiveProjectionState` is absent and forbidden here. `LayoutPromotion` and
filesystem persistence are also out of scope, but the typed Draw extension
must already produce a canonical `CanonicalExtensionPayload` and form a
validated linked `CompositionBundleCandidate`. This proves that later saving
will use atomic core/extension metadata and hashes instead of inventing a Draw
format.

The runtime pair is installed atomically:

```text
DrawingCompositionRuntime
  composition: CompositionState
  extension: DrawingCompositionExtensionV1

RunenwerkDrawApp
  composition_runtime: DrawingCompositionRuntime
  composition_content: DrawingCompositionContentState
  composition_projection: DrawingCompositionProjection
  document and ink state: unchanged app/drawing ownership
```

Window size is an app-owned target projection input. Native windows, monitor
bounds, DPI, restore policy, render surfaces, and OS vetoes do not enter core
composition or the extension.

## Built-in Definition

The built-in definition contains one `runenwerk.draw.wide` target, one primary
root, and explicit split/mount regions for:

- top bar;
- left tool rail;
- canvas;
- right support/tablet panel.

IDs are non-zero compile-time constants. `MountedContentRef` values use the
`runenwerk.draw.*` owner/profile/instance namespace and remain opaque to the
core. Split fractions describe the reference wide layout; the projector
recurses `RegionKind` and may not reconstruct an independent topology.

The static projector calculates all region bounds from the structural root and
target size. The canvas view is then derived from the canvas mounted-unit
bounds and app-owned paper margin. No mounted unit is deleted or hidden because
of target width.

## Draw Extension And Determinism

`DrawingCompositionExtensionV1` contains only app-owned interpretation:

- layout ID, core schema version, definition revision, extension schema
  version, and `runenwerk.draw` compatibility profile;
- exactly one sorted `MountedUnitId -> DrawingContentRole` record per core
  mounted unit;
- app unavailable-projection policy for each role.

It does not duplicate region topology, split fractions, root/target links,
mounted-content refs, canvas/document state, tablet session state, or native
handles.

Validation rejects core metadata mismatch, unsupported schema/profile,
missing/extra/duplicate mounted-unit records, duplicate roles, and a role whose
expected content profile does not match the opaque core reference. Rejection is
atomic.

Canonical RON uses schema field order, normalized `MountedUnitId` ordering, LF
line endings, one trailing newline, unknown-field rejection, and decode/re-
encode byte identity. Display labels are absent from identity and ordering.
The extension is converted to `CanonicalExtensionPayload`; core persistence
forms and validates the shared layout/revision/schema/compatibility metadata
and `blake3:` core/extension links.

## Content Liveness And Fallback

`DrawingCompositionContentState` is app-owned, keyed by `MountedUnitId`, and
contains exactly one neutral liveness observation per mounted unit. All seven
states are supported: resolved, missing, loading, suspended, denied,
unsupported profile, and crashed.

Projection uses the required order:

1. Draw-provided unavailable-content projection;
2. neutral diagnostic placeholder;
3. hidden only if both the core mounted-unit policy and host policy permit it.

The built-in Draw host keeps a neutral placeholder available, so unresolved
content remains structurally visible. Tests exercise every state and the
explicit hide permission edge case. Liveness changes never alter the
composition definition, state revision, document, or stroke history.

## Diagnostics

Every Draw composition rejection or unavailable projection emits a stable
record with code, severity, stage, typed subject ID, actionable message, and
deterministically ordered context. Codes use only `draw_composition.*`.
Formation and persistence failures retain their owning
`ui_composition.*`/`composition_persistence.*` diagnostics and are wrapped only
at the app boundary when Draw context is needed.

Required Draw families cover extension schema/core/coverage/profile mismatch,
missing role bounds, invalid target bounds, content liveness, and unavailable
fallback exhaustion.

## Module And API Cutover

Create `apps/runenwerk_draw/src/app/composition/`:

| Module | Responsibility |
|---|---|
| `mod.rs` | narrow Draw composition public surface |
| `definition.rs` | stable IDs and built-in wide definition |
| `diagnostic.rs` | `draw_composition.*` typed records/rejections |
| `extension.rs` | typed deterministic extension and bundle payload |
| `content.rs` | mounted-unit keyed liveness and fallback decisions |
| `projection.rs` | graph recursion, bounds, canvas view, and projection DTOs |
| `runtime.rs` | atomic state/extension pairing and built-in formation |

Delete `app/workspace.rs`. Rename `DrawingWorkspaceProjection` to
`DrawingCompositionProjection`, `toolbar_bounds` to `tool_rail_bounds`, and
`layer_panel_bounds` to `support_panel_bounds`. Remove `workspace()` with no
alias. Presentation builders become `build_composition_frame*`; private helper
names follow the same terminology. Existing drawing and ink public APIs remain
unchanged.

## Fitness Functions And Tests

Tests prove:

- the built-in definition forms and its root graph causally produces top bar,
  tool rail, canvas, and support bounds;
- definition/extension records are deterministic and linked bundle validation
  succeeds;
- malformed extension metadata, coverage, roles, and profiles reject without
  partial installation;
- all seven liveness states preserve structure and follow fallback order;
- resize and liveness changes do not mutate drawing documents, strokes,
  drawing revision, or composition state revision;
- startup, pointer routing, ink publication, and app-shell behavior remain
  green;
- active Draw source contains no `DrawingWorkspaceProjection`, public
  `workspace()`, editor import, adaptive import, or native handle in the core
  definition/extension;
- the narrow target keeps the support mounted unit structurally present; no
  responsive drawer/hide behavior lands early.

Validation:

```text
cargo fmt --all --check
cargo test -p runenwerk_draw --test app_shell
cargo test -p runenwerk_draw --test composition_architecture_guards
cargo test -p runenwerk_draw composition
task ui:dependencies
task docs:validate
task planning:validate
```

## Implementation Sequence

1. Add the direct `ui_composition`, `serde`, and `ron` dependencies.
2. Add diagnostics, built-in definition, deterministic extension, and atomic
   runtime formation.
3. Add mounted-unit liveness and the pure recursive static projector.
4. Replace `RunenwerkDrawApp` workspace state with runtime, content, and
   projection state.
5. Rename frame builders and redirect presentation/runtime readers.
6. Delete `app/workspace.rs` and cut tests to the new public API.
7. Add architecture, extension, liveness, and document-authority guards.
8. Update Draw architecture/usage docs, run all gates, and create resolver-
   backed runtime, fixture, and diagnostics evidence.

Run the phase-completion drift check before WR-185.

## Rollback And Stop Conditions

Rollback rejects the checkpoint and retains PM-004 artifacts. It must not add
an alias, dual projection authority, or hidden compatibility writer.

Stop if:

- any projection bound is produced by an independent hard-coded topology;
- `DrawingWorkspaceProjection` or public `workspace()` remains active;
- drawing document, stroke, brush, layer, or history authority moves;
- extension state duplicates core topology or mounted-content references;
- core and Draw extension can install partially;
- unavailable content invalidates or mutates structural state;
- narrow width silently removes/hides a mounted region;
- implementation requires editor, adaptive, engine, windowing, native-window,
  persistence repository, or Region Compass runtime changes;
- required scopes, tests, diagnostics, or evidence cannot prove the claims.

Closeout must name adaptive Draw behavior as remaining WR-187 scope. This
checkpoint may claim `runtime_proven`, not final runtime or perfectionist
completion.
