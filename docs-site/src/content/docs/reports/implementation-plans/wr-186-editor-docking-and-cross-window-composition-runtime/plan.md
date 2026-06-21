---
title: Editor Docking And Cross-Window Composition Runtime Implementation Plan
description: Decision-complete WR-186 contract for Region Compass, transaction-only editor structure, target-scoped UI/input, native-window coordination, and real multi-surface presentation.
status: accepted
owner: editor
layer: report
canonical: false
last_reviewed: 2026-06-20
wr: WR-186
milestone: PM-UI-COMPOSITION-007
related_designs:
  - ../../../design/accepted/adaptive-ui-composition-design.md
  - ../../../design/accepted/app-neutral-ui-composition-design.md
  - ../../../design/accepted/editor-native-multi-window-presentation-design.md
  - ../../design/ui-composition-visual-direction/selection.md
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# WR-186: Editor Docking And Cross-Window Composition Runtime

## Goal

Install the selected Region Compass interaction as the editor's only live
structural editing path. Every committed editor layout change must pass through
one revision-bound `ui_composition` transaction and one atomically matched
editor-extension change. A detached tab must appear in a real OS window with a
distinct target-scoped UI frame, render surface, input/focus state, and DPI.
Closing a target must rehome its roots or veto without partial structural state.

WR-185 is runtime-proven and supplies five-zone typed intent. It does not supply
topology commands, editor policy, extension deltas, IDs, native windows, or
render surfaces. WR-186 owns those host responsibilities without moving native
window ownership into `ui_composition` or `editor_shell`.

## Readiness And Governance Decision

- WR-185 and PM-UI-COMPOSITION-006 are completed at `runtime_proven`.
- Option 2, Region Compass, is the binding visual direction.
- ADR 0013 and the three accepted designs already decide dependency direction,
  proposal-only adaptive ownership, app-owned extension policy, and native
  window ownership. No new ADR is required.
- This plan may implement. Stop for an ADR/design update if native handles enter
  domain state, adaptive code gains commit authority, render surfaces gain
  composition semantics, or the five-zone/detach vocabulary changes.

Architecture review found two gaps that must be corrected inside this WR:

1. Current core commands cannot honestly split or create a root around an
   existing unit without constructing a temporarily invalid/duplicated stack.
   Add narrow app-neutral compound commands for those two operations.
2. Existing multi-window proof stops at native-window and render-surface
   metadata. Secondary redraw does not run a target-specific frame and `Gfx`
   owns one WGPU surface. Metadata-only proof cannot satisfy real cross-window
   movement. This WR therefore includes the narrow generic multi-surface render
   correction required to present distinct editor targets.

## Architecture Ownership

### DDD owners and vocabulary

- `ui_composition`: target, root, region, mounted unit, structural command,
  transaction, revision, commit, structural history.
- `ui_adaptive_composition`: adaptive projection, drag/resize session, DockZone,
  preview, revision-bound proposal. Proposal only.
- `editor_shell`: editor docking policy, Region Compass view model, editor
  extension change set, compatibility projection, logical editor target state.
- `runenwerk_editor`: provider/session policy, content creation, ID reservation,
  app transaction coordination, target/native/surface binding, close/rehome
  policy, save blocking while coordination is pending.
- `engine::runtime`: native window lifecycle intent and normalized window-scoped
  events. No editor/composition meaning.
- `engine::render`: WGPU device and per-render-surface presentation contexts.
  No editor/composition meaning.

Team Topologies label: stream-aligned editor runtime with complicated-subsystem
UI-composition and render/windowing platform support.

### Dependency direction

```text
ui_composition <- ui_adaptive_composition
       ^                 ^
       |                 |
       +-------- editor_shell
                    ^
                    |
             runenwerk_editor ----> engine window/render contracts
```

`editor_shell` may depend on both UI domain crates but never on `engine` or the
app. Engine contracts remain generic and must not import editor, composition,
workspace, panel, or provider vocabulary.

## Source Truth And Runtime Chain

Source truth is `CompositionState` plus `EditorCompositionExtensionV1`.
`AdaptiveProjectionState`, Region Compass state, drag ghosts, previews, target
bindings, native windows, WGPU surfaces, UI trees, layout maps, and prepared
frames are derived runtime products.

The complete chain is:

```text
CompositionState + editor extension
  -> target-scoped adaptive projection
  -> target-scoped editor projection/provider requests
  -> Region Compass view model + semantic actions
  -> AdaptiveProposal
  -> editor policy + reserved identities
  -> prepared core transaction + editor extension change set
  -> atomic editor runtime commit
  -> target/native-window/render-surface binding
  -> target-scoped UiFrame submission
  -> per-surface PreparedRenderFrame
  -> matching WGPU surface acquire/render/present
```

Tests must fail if the chain stops at a proposal, descriptor, binding record,
render-surface registry row, or prepared frame that is never presented.

## Core Transaction Amendments

Add two neutral commands in
`domain/ui/ui_composition/src/transaction/command.rs` and implement them in
`transaction/apply.rs`:

- `split_region_with_moved_unit`: remove an existing mounted unit from its
  current location, preserve the destination region as a fresh child, create a
  fresh stack child containing exactly that unit, and replace the destination
  with the requested split in one command.
- `create_root_with_moved_unit`: remove an existing mounted unit from its
  current location and create a fresh root plus one fresh stack region
  containing exactly that unit in one command.

Both commands carry typed IDs, region/root definitions, split axis/fraction,
and ordering. They reject duplicate IDs, missing units, non-stack payloads,
wrong root-region linkage, self-invalidating source/destination combinations,
and malformed single-unit stacks with stable `ui_composition.*` diagnostics.
They do not allocate IDs, compact source topology, decide target policy, or
touch app extension/window state.

The editor transaction planner adds ordered `merge_split`, `close_root`,
`move_root`, `attach_target`, and `detach_target` commands when source
compaction or target lifecycle requires them. Final candidate validation stays
atomic. Center placement continues to use `move_unit`; resize uses
`resize_split`.

## Editor Atomic Transaction Contract

Create `composition/structural/transaction.rs` with:

- `EditorCompositionChangeSet`: expected state revision, core transaction,
  typed extension upserts/removals/relinks, and app correlation ID;
- `PreparedEditorCompositionCommit`: private candidate core state, candidate
  extension, projected target artifacts, structural history metadata, and the
  expected source revision;
- `EditorCompositionRuntime::prepare_change`: clone the ratified state, execute
  the core transaction against supplied policies, apply/relink the extension
  change, validate exact extension coverage, project every target, and return a
  candidate without mutating live state;
- `EditorCompositionRuntime::commit_prepared`: recheck source revision and swap
  core, extension, projection products, and extension journal together;
- target-aware structural undo/redo that asks core history to revalidate and
  restores the paired extension snapshot. It never enters scene/document,
  browser, drawing, graph, terminal, or game history.

No API may expose mutable core or extension fields independently. Extension-only
policy edits such as a region lock use the same expected-revision compare-and-
swap runtime but are classified as editor extension state, not fabricated core
commands.

`EditorCompositionExtensionV1` gains a private normalized builder/change-set
application path. Every candidate is relinked to the committed definition
revision and must cover exactly the candidate units, regions, and roots before
commit. A failed core command, extension change, projection, policy decision,
or revision check leaves all live state and history unchanged.

## Identity Policy

Create `EditorCompositionIdentityAllocator` under the editor composition
subdomain. It scans installed core/extension state and allocates independent,
monotonic typed sequences for target, root, region, mounted-unit, panel,
tab-stack, host, content-instance, viewport, and editor-window identities.
Display labels never participate in identity or ordering.

An operation reserves IDs from a cloned allocator. The allocator candidate is
committed only with the prepared editor commit. A single app-owned structural
coordination slot prevents overlapping async window operations; ordinary
structural transactions reject with a stable busy diagnostic while the slot is
occupied. No global mutable allocator or ID reuse after a committed operation.

## Topology And Extension Materialization

Create `composition/docking/transaction.rs` as a pure planner over a
`CompositionSnapshot`, editor extension snapshot, `AdaptiveProposal`, and an ID
reservation. It emits `EditorCompositionChangeSet`; it never calls engine/app
APIs.

Materialization rules:

| Intent | Core transaction | Extension action |
|---|---|---|
| Center | `move_unit` to target stack and ordinal, then compact empty source | preserve unit; remove/rewrite eliminated region/root records |
| Edge | `split_region_with_moved_unit`, then compact empty source | preserved child inherits destination compatibility identity; split parent and moved-unit child receive explicit records |
| Resize | `resize_split` | no topology delta; relink revision |
| Detach to new target | `attach_target` plus `create_root_with_moved_unit`, then compact source | add target root/region records; preserve mounted-unit/provider identity |
| Move whole final root | `attach_target`, `move_root`, optional `detach_target` | preserve root/region/unit records and update app target binding |
| Close target | move every root to deterministic fallback as non-primary, detach target | preserve content records; clamp floating bounds; remove target binding |
| Activate tab | `activate_unit` | no topology delta; relink revision |
| Close tab(s) | `unmount_unit` commands plus source compaction | remove unit/provider projection records only after app close policy accepts |
| Create tab | app creates typed content record, `mount_unit` | add one mounted-unit record and provider/session identity |
| Split with new tab | `mount_unit` into target then `split_region_with_moved_unit` | add content and split records atomically |

When a source stack becomes empty, the planner deterministically merges its
parent split using the surviving sibling, or closes its root. If closing the
last root on a target, it either promotes another root on that target or detaches
the now-empty target and requests its native window close. A target always has
exactly one primary root in the committed definition.

Target-close fallback is the bound primary target unless it is closing;
otherwise use the lowest bound target ID. Last-window close follows explicit
app quit/dirty-document policy and may veto. No fallback means veto, not hidden
content or unmounting.

## Region Compass Interaction Contract

Replace the old `DockingPreviewDropTarget`/workspace-host candidate authority
with typed `RegionId`, `MountedUnitId`, `PresentationTargetId`, `DockZone`, and
source revision. Legacy widget IDs remain projection inputs only until WR-188.

Create `composition/docking/view_model.rs` and render it from
`build_editor_shell.rs`. Runtime states are:

1. `Idle`: no overlay.
2. `Armed`: explicit drag region pressed but threshold/long-press not met; no
   overlay and no mutation.
3. `ActiveNoDestination`: drag ghost and separate detach portal visible.
4. `ActiveDestination`: contextual five-zone compass anchored inside the hit
   region; all legal targets visible; invalid targets visibly and semantically
   disabled; only the focused target has an outcome preview.
5. `DetachFocused`: the explicit New Window/existing-target portal is focused;
   it is never encoded as an edge split.
6. `CommitPending`: preview is frozen while app/window coordination completes;
   cancel remains available and no second structural operation starts.
7. `Rejected`: stable actionable diagnostic, no structural change, interaction
   returns to a usable focused state.
8. `Committed` or `Cancelled`: transient overlay/session is removed; committed
   state reprojects, cancelled state exactly matches the baseline revision.

Visual constraints come directly from the selected artifact: near-black
surface, one-pixel border, zero radius, compact typography, electric-blue focus,
and dashed focused preview. Visual glyphs may remain compact, but pointer/touch
hit areas are at least 44 logical pixels. The compass is clamped inside the
destination region and never obscures more than the minimum target cluster.
Text scaling through 200% must preserve labels and the detach portal. Reduced
motion sets transition duration to zero without changing state or feedback.

Stable inspection labels name action and destination, for example “Dock Scene
Viewport left in Inspector region” and “Move Scene Viewport to new window”.
Color is never the only state signal. High-contrast focus uses border, glyph,
and state label. Deterministic spatial focus order is center, left, right, top,
bottom, detach/existing-target portals; directional focus chooses the spatial
neighbor and tab-cycle reaches every legal target.

Pointer, touch long-press, keyboard, and controller all produce the same
`UiSemanticAction` and adaptive proposal path. Escape, rollback, focus loss,
window close intent, stale revision, policy rejection, and OS/window creation
failure cancel without structural mutation.

## Target-Scoped Editor Runtime

Replace the single shell `UiRuntime`, tree, bounds, projection artifacts,
adaptive projection, focus, capture, and drag session with
`EditorTargetRuntimeState` keyed by `PresentationTargetId`. Global editor
project/document/provider truth remains shared. Each target state owns only
presentation/runtime products.

`project_editor_composition` becomes target-aware. It projects exactly one
primary root for the requested target plus that target's non-primary roots.
Projection artifacts carry `RegionId`, `MountedUnitId`, and target ID at every
structural route. `TabStackId`, `PanelHostId`, and old widget IDs are translated
at the projection boundary and never used to plan core topology.

Provider requests filter by target through mounted-unit location. Moving a
mounted unit preserves its provider/session/content identity. A viewport keeps
its explicit viewport/product identity while its target, native window, frame,
surface, DPI, and bounds change.

The current test-only `legacy_workspace_oracle`, writable replacement methods,
and `reduce_workspace` helpers are deleted from the app. Legacy workspace/profile
code may remain only as read-only boot/import input until WR-188. Architecture
guards reject active app imports or calls to `WorkspaceMutation`,
`reduce_workspace`, and writable `WorkspaceState` APIs.

## Native Window Coordination

Engine window records distinguish:

- `Requested`;
- `Created`;
- `CreationFailed` with actionable reason;
- `CloseIntentPending`;
- `CloseApproved`.

Platform `CloseRequested` records intent; it does not set the presentation
close flag. `approve_close` is the only policy-coordinated path that sets
`close_requested`; `veto_close` returns the record to `Created`. Programmatic
engine shutdown remains an explicit immediate-close API and is not confused
with OS close intent.

App coordination is a prepare/commit/compensate saga:

### New target or detach

1. Validate proposal/source revision and prepare the complete editor change.
2. Reserve logical/structural IDs and request a native window; publish no target
   binding or structural commit yet.
3. Winit creates the native window, attaches a WGPU surface using the existing
   device/adapter, and reports `Created`; failure reports `CreationFailed`.
4. Recheck source revision and binding uniqueness, then commit core, extension,
   target runtime, logical window, and target/native/surface binding together.
5. On failure or cancel, close/detach the created native surface/window and
   discard the prepared candidate. No structural state was committed.

### Close intent

1. Map native ID to exactly one target and freeze target-local interaction.
2. Apply dirty-document/last-window policy and prepare root rehome plus target
   detach against the current revision.
3. On veto or preparation failure, call `veto_close` and preserve all state.
4. On acceptance, commit editor structure/extension/bindings, then
   `approve_close`; engine retires the WGPU surface, render-surface row, window
   record, and native window.

Only one async structural saga may be active. Saving/promotion while a saga is
pending fails atomically with `editor_composition.coordination.pending`.
Transient adaptive projection is never saved.

## Real Multi-Surface Render Correction

`WgpuCtx` becomes one device/queue/adapter plus a map of per-`RenderSurfaceId`
surface/config records. `Gfx` attaches/detaches native windows, resizes and
acquires by surface ID, and renders a prepared frame only to the matching
surface. Surface formats remain per surface; renderer pipeline caches already
key format and must continue to do so.

`UiFrameSubmission` gains an optional concrete screen `RenderSurfaceId` scope.
The registry key is `(producer, surface scope)`, so one editor producer can
publish one frame per target without overwriting another. Non-screen routes keep
their existing semantics.

Frame prepare runs one simulation/update and builds a deterministic
`PreparedRenderFrameSet` for every registered, created, non-retired surface.
Each frame has the matching native window ID, size, surface-scoped UI
contribution, and product bindings. Frame submit validates and renders each
entry in surface-ID order. Secondary `RedrawRequested` never executes a second
simulation update; all surfaces are presented from the single frame set.

The engine layer knows only native window IDs, render surface IDs, sizes, and UI
submission scope. Target-to-surface mapping remains app-owned.

## Window-Scoped Input And Focus

Add a generic normalized `PlatformWindowEventQueueResource` in engine runtime.
Winit records window-scoped keyboard, pointer, touch, focus, resize, DPI, and
close events before applying primary legacy compatibility. Add
`PlatformEvent::Focused(bool)`.

The editor drains the queue once per frame, maps native ID through the app-owned
target binding, and routes input into that target's `UiRuntime` and adaptive
session. Focus/capture in one target cannot consume another target's events.
Unknown/stale native IDs emit diagnostics and do not fall back to the primary
target. Abstract controller events enter through `UiSemanticAction`, never raw
gamepad semantics.

## App Diagnostics

Extend `editor_composition.*` with stable codes for stale proposal, invalid
dock target, identity exhaustion, extension change mismatch, concurrent
coordination, missing/duplicate target binding, native creation failure,
surface attach failure, close veto, no rehome target, target event mismatch,
projection failure, and save-while-pending. Every rejection records severity,
stage, subject ID, actionable message, and relevant revision/target context.

Engine render/window diagnostics remain generic and app policy diagnostics
remain editor-owned. No rejection logs success or silently falls back to the
primary target.

## Public API And Module Scope

Add editor subdomains:

```text
domain/editor/editor_shell/src/composition/
|-- docking/
|   |-- mod.rs
|   |-- intent.rs
|   |-- policy.rs
|   |-- transaction.rs
|   |-- view_model.rs
|   `-- accessibility.rs
`-- structural/
    |-- identity.rs
    `-- transaction.rs
```

Extend app composition runtime:

```text
apps/runenwerk_editor/src/shell/composition_runtime/
|-- adaptive.rs
|-- coordination.rs
|-- policy.rs
|-- target_runtime.rs
`-- window_binding.rs
```

Existing `mod.rs` files expose only the common host workflow. No `utils`,
`helpers`, `_internal`, `include!`, universal app-host trait, or mutable public
field bags.

Expected existing-file scope includes the roadmap-listed editor shell/app,
engine window/platform/winit, generic render backend/UI/frame/runtime modules,
core transaction modules/tests, app window/composition tests, and editor docs.
Do not touch Draw, `ui_surface`, `ui_program_hosts`, game/runtime product
semantics, or persistence schemas outside editor layout activation.

## Implementation Sequence And Checkpoint Gates

1. **Core command gate**: add the two compound commands, stable diagnostics,
   atomicity/property/history tests, and no app vocabulary.
2. **Editor transaction gate**: add identity reservation, extension change set,
   prepare/commit/undo/redo, source compaction, and projection tests.
3. **Target runtime gate**: make projection/provider/UI runtime target-scoped;
   remove writable legacy oracle/helpers; add architecture guards.
4. **Generic window/render gate**: pending close lifecycle, creation failure,
   event queue, real WGPU surface registry, surface-scoped UI submissions,
   prepared frame set, one-update/many-present tests.
5. **App coordination gate**: new target, detach, cross-target move, close rehome,
   veto, compensation, save blocking, and binding invariants.
6. **Region Compass gate**: replace old docking candidates, render selected
   visual states, wire semantic pointer/touch/keyboard/controller parity,
   accessibility inspection, cancel/reject behavior, and resize transactions.
7. **Structural command cutover gate**: replace every
   `reject_static_composition_mutation` branch with transaction/extension
   execution or a typed app-policy rejection. Remove the rejection helper.
8. **Runtime/visual proof gate**: two real windows, distinct frames/surfaces/DPI,
   tab and viewport identity movement, close rehome/veto, selected-image
   comparison, performance probes, docs, locked evidence, and phase drift check.

Run focused validation after every internal gate. Do not close WR-186 if any old
structural mutation path remains or secondary rendering is metadata-only.

## Acceptance Criteria

- Center, four edges, reorder, resize, create/close/close-other/split/duplicate/
  reset/lock operations use revision-bound editor composition transactions.
- A stale or rejected operation changes no core, extension, projection,
  provider, binding, history, or allocator state.
- Region Compass shows all legal roles and one focused preview, with detach
  separate from split targets.
- Pointer, long-press touch, keyboard, and controller have semantic parity;
  Escape/focus loss/window close rolls back.
- Focus is visible/deterministic; labels, high contrast, 200% text scale,
  reduced motion, and 44-pixel hit targets pass inspection.
- `Window > New Window` creates a real target with one fresh viewport content
  instance and a real native/WGPU surface; it does not alias mounted-unit,
  viewport, panel, or provider-session identity.
- Detach preserves `MountedUnitId` and provider/session identity while moving
  the unit to a real target/window/surface.
- Two windows receive distinct target-scoped UI frames; resize/DPI/focus/input
  on one does not mutate the other's target runtime.
- Closing a secondary target rehomes all roots or vetoes; last-window and dirty
  policy are explicit. No content disappears to satisfy close.
- Engine generic APIs contain no editor/composition vocabulary.
- Active app source has no writable legacy workspace authority and no direct
  structural mutation outside the editor composition runtime.

## Fitness Functions And Validation

Focused tests:

```text
cargo fmt --all --check
cargo test -p ui_composition
cargo test -p editor_shell composition
cargo test -p editor_shell docking
cargo test -p runenwerk_editor docking
cargo test -p runenwerk_editor window
cargo test -p runenwerk_editor --test composition_architecture_guards
cargo test -p engine window
cargo test -p engine --test render_multi_surface
task ui:dependencies
task docs:validate
task planning:validate
```

Native/GPU proof when supported on the local macOS main thread:

```text
RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored --nocapture
```

Add deterministic probes for Region Compass pointer update, transaction
materialization/commit, per-target frame build, and multi-surface prepare/submit.
Pointer updates retain the WR-185 zero-full-graph-clone rule. No frame path may
clone the full composition once per surface; all target projections share the
ratified snapshot and rebuild only target-local runtime products.

Visual evidence must capture idle, center, each edge, invalid, detach,
keyboard-focus, touch, high-contrast, 200% text, reduced-motion, rejected, and
committed states at the selected 1440x1024 reference viewport plus a secondary
window. Compare the selected Region Compass artifact and actual runtime capture
together at the same viewport/state; a screenshot alone is not visual QA.

## Non-Goals

- Draw adaptive drawers/reflow (WR-187).
- `ui_surface` deletion, `ui_hosts` rename, or final legacy file/schema removal
  (WR-188).
- Final zero-finding perfectionist track closure (WR-189).
- Monitor placement, platform-native tabbing, session sharing across processes,
  remote windows, or persisted transient drag/window-coordination state.
- Moving document, scene, drawing, browser, terminal, graph, viewport product,
  or game-state authority into composition.

## Stop Conditions

Stop and revise governance instead of implementing if:

- adaptive code needs `&mut CompositionState` or materializes commands;
- editor/domain code needs native/render handles;
- engine code needs editor, panel, provider, workspace, or composition types;
- an operation requires a temporary second structural authority or writable
  legacy workspace;
- core and editor extension cannot prepare/commit/undo atomically;
- secondary windows can be claimed only from metadata/registry records;
- one native event can fall back to or mutate the wrong target;
- window creation/close cannot compensate without partial structure;
- visual implementation departs from Region Compass target vocabulary or theme;
- accessibility, cancellation, revision, surface scoping, or performance proof
  is missing;
- any required change falls outside the amended governed output scope.

## Closeout Requirements

The locked execution writer must produce resolver-backed evidence for:

- core/editor transaction and structural-command runtime tests;
- real target/native/surface cross-window runtime proof;
- Region Compass visual comparison and accessibility matrix;
- architecture/migration guards proving transaction-only mutation and generic
  engine ownership;
- diagnostics, compensation, clone, and timing probes.

Closeout quality is `runtime_proven`, not `perfectionist_verified`, because Draw
adaptive proof, legacy deletion/rename, and independent final audit remain
WR-187 through WR-189. The closeout must list any remaining repository-wide
Clippy or native-platform limitations and run `quiet_full_gate.sh` as the phase
drift check. PM-UI-COMPOSITION-007 and WR-186 may complete only after direct
legacy structural mutation is absent and real multi-surface presentation is
proven.

## Perfectionist Closeout Audit

WR-186's audit is bounded to this checkpoint. It must report zero findings for
the contracts above, but it must not erase later-track gaps. The final
perfectionist audit remains
`docs-site/src/content/docs/reports/closeouts/pm-ui-composition-010-perfectionist-verification-and-closeout/closeout.md`
and may claim `perfectionist_verified` only when WR-187, WR-188, and every final
full-gate finding are closed with current evidence.
