---
title: Editor Static Composition Projection Cutover Implementation Plan
description: Decision-complete WR-183 contract for editor composition import, static shell projection, MountedUnitId-keyed providers and sessions, app extension persistence, and the legacy WorkspaceState read-only gate.
status: accepted
owner: editor
layer: report
canonical: false
last_reviewed: 2026-06-19
wr: WR-183
milestone: PM-UI-COMPOSITION-004
related_designs:
  - ../../../design/accepted/app-neutral-ui-composition-design.md
  - ../../../design/active/editor-ui-workspace-tool-surface-architecture.md
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-183: Editor Static Composition Projection Cutover

## Authority And Promotion

PM-UI-COMPOSITION-003 and WR-182 are complete with `runtime_proven`
deterministic persistence evidence. WR-183 is promotable after this accepted
contract validates. The visual-direction gate is also complete: Region Compass
is selected. This checkpoint does not implement Region Compass chrome,
adaptive reflow, drag sessions, docking proposals, cross-window movement, or
native-window behavior.

The governed write scope was corrected before this plan was accepted:

- `domain/editor/editor_shell/src/composition` already exists, so it is an
  existing scope rather than `new:` output;
- both consumer `Cargo.toml` files and `Cargo.lock` are authorized for the new
  `ui_composition` dependency;
- runtime resources, viewport lifecycle, frame submission, input/picking,
  mounted-surface registries, session pruning, and startup guards are included
  because they currently read or mutate `WorkspaceState` outside `shell/`;
- overlapping editor-shell child scopes were replaced by the existing
  `domain/editor/editor_shell/src` ownership boundary.

Implementation discovery also proved that the mounted-unit identity cutover
crosses the app facade, inspector/viewport dispatch, shared provider fallback,
material graph provider, and provider contract tests. Those exact files are
declared in `plan.contract.yaml`; no broader app subtree authority is implied.
The editor diagnostic conversion adds only the existing foundation
`diagnostics` contract dependency to `editor_shell`.

## Architecture Governance Decision

Recommendation: implement.

Scope: editor structural projection, editor app extension/persistence, provider
and session identity, runtime read consumers, and the read-only legacy gate.

Owner: `domain/editor/editor_shell` is the stream-aligned editor projection and
extension owner. `apps/runenwerk_editor` owns provider behavior, sessions,
storage roots, profile selection, runtime target bindings, and command
execution. `domain/ui/ui_composition` remains the platform structural owner.

Dependency direction:

```text
ui_composition
  <- editor_shell structural import/projection/extension contracts
  <- runenwerk_editor providers/sessions/persistence/runtime wiring
```

No dependency points from `ui_composition` into editor, app, runtime,
`ui_surface`, or windowing code.

ADR need: none. ADR 0013 already decides the structural authority, clean
cutover, branch coexistence, editor ownership, and checkpoint order. A new ADR
is required only if implementation discovers that provider/session ownership,
native-window ownership, or structural authority must move.

Ownership mode: stream-aligned editor integration consuming the UI composition
platform, with an enabling architecture/fitness-function review.

### ATAM-lite

Quality attributes in tension:

- single structural authority and deterministic persistence;
- preservation of current static render/provider behavior;
- branch-local implementation safety;
- avoiding duplicated legacy mutation semantics before Region Compass runtime;
- future deletion cost.

Options evaluated:

1. Keep `WorkspaceState` and `CompositionState` writable and synchronize them.
   Rejected: dual authority, non-atomic failure modes, and permanent drift risk.
2. Translate all 26 legacy `WorkspaceMutation` variants into composition now.
   Rejected: duplicates the old reducer, preempts WR-186 adaptive mechanics,
   and increases deletion cost without improving the final Region Compass path.
3. Import legacy/profile structure once, store only composition plus editor
   extension state, project statically, and freeze structural affordances until
   WR-186. Selected: it establishes authority cleanly and keeps the temporary
   limitation branch-local because final merge remains blocked on later gates.
4. Re-form `CompositionState` from a reduced legacy clone after every command.
   Rejected: the legacy reducer remains the real mutation authority and
   structural history/revision semantics are lost.

Sensitivity points:

- stable identity translation must preserve mounted-unit/provider/session
  continuity;
- extension and core state must be validated together at every import/load;
- projection parity must not reconstruct `WorkspaceState` as a hidden cache;
- floating bounds and viewport restore state must remain editor/app extension
  data rather than enter core structure;
- structural controls must be visibly disabled with stable diagnostics rather
  than fail silently during the branch-local static interval.

Accepted risk: structural editor interactions are unavailable between this
checkpoint and WR-186 on the clean-cutover branch. This is not releasable state
and cannot merge. Static shell rendering, provider content, product actions,
viewport observation, focus, and non-structural session behavior must remain
operational.

## Current Writer And Consumer Inventory

Every current production writer is mapped before implementation:

| Current writer | Current behavior | WR-183 replacement |
|---|---|---|
| `apps/runenwerk_editor/src/shell/state.rs::replace_workspace_state` | replaces live structural authority | remove; install a validated `EditorCompositionRuntime` bundle |
| `apps/runenwerk_editor/src/shell/state.rs::apply_workspace_mutation*` | invokes `reduce_workspace` and replaces state | remove; structural commands return a stable static-gate rejection until WR-186 |
| `apps/runenwerk_editor/src/shell/state.rs::update_*split*` | commits legacy split fractions | remove from static authority; resize is deferred to adaptive transactions |
| `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` structural arms | tab, split, close, duplicate, reset, lock, drag/drop, profile replacement | profile replacement installs composition; structural arms are disabled/rejected explicitly |
| `apps/runenwerk_editor/src/runtime/systems/viewport_lifecycle.rs` | writes viewport IDs/settings through workspace mutations | update editor extension/runtime binding keyed by `MountedUnitId` |
| `apps/runenwerk_editor/src/runtime/resources.rs` | restores/replaces workspace state | load/install a validated composition bundle or built-in composition profile |
| `apps/runenwerk_editor/src/persistence/workspace_layout.rs::write_workspace_layout*` | writes V5 workspace RON | delete active writer APIs; save atomic composition bundle generations |
| editor self-authoring/tool-suite preview paths | replace workspace during previews/tests | project/import a candidate without changing live legacy authority |

Read consumers are also redirected:

| Current consumer | WR-183 source |
|---|---|
| `composition/build_editor_shell.rs` | `EditorCompositionProjectionArtifact` |
| `shell/providers/mod.rs::mounted_surface_requests_with_registry` | mounted units plus editor extension bindings |
| `shell/surface_session.rs` | `MountedUnitId` |
| `editor_app/state.rs::prune_surface_sessions_for_workspace` | live mounted-unit set |
| `runtime/systems/frame_submit.rs` | composition projection and mounted-unit registry |
| runtime input/picking/viewport modules | mounted-unit and region bindings from projection artifacts |
| workbench/tool-suite inspectors | composition/extension inspection DTOs, never persisted V5 reconstruction |

Tests may continue to construct and reduce `WorkspaceState` inside
`editor_shell` to prove the read-only importer and temporary parity oracle.
Active app production code may not call the reducer or retain `WorkspaceState`.

## Runtime State And Ownership

Add an `EditorCompositionRuntime` owned by `RunenwerkEditorShellState`:

```text
EditorCompositionRuntime
  composition: CompositionState
  extension: EditorCompositionExtensionV1
  target_bindings: app-owned PresentationTargetId bindings
  projection_epoch: editor runtime fact
```

The live shell stores no `WorkspaceState`. Legacy/profile state exists only as
an input argument to `import_legacy_workspace`, is validated, converted, and
dropped before the runtime is returned.

`CompositionState` is the only structural source. The editor extension is not
a second structural graph. It may contain only editor-owned associations and
restore metadata required to interpret the core graph:

- active editor profile and display metadata;
- `MountedUnitId` to legacy panel/surface compatibility identity while those
  product contracts remain on the branch;
- stable editor content key/provider-family route metadata;
- `RegionId` to tab-stack chrome metadata and optional lock policy;
- root/floating presentation metadata and bounds;
- viewport instance/settings restore metadata;
- app-owned target binding references, never native handles.

It must not duplicate split topology, region parentage, unit order, active unit,
root-to-target structure, or mounted-content references. Those are read from
`CompositionState` only.

Formation validates exact coverage: every mounted unit has one extension
record; every referenced region/root exists; no extension record points to a
missing core identity; editor compatibility IDs are unique; content keys and
provider profiles agree; and target metadata does not claim OS ownership.

## Module Ownership And Public Surface

Add `domain/editor/editor_shell/src/composition/structural/` with explicit
subdomain files:

| Module | Responsibility |
|---|---|
| `mod.rs` | narrow editor composition public surface |
| `diagnostic.rs` | `editor_composition.*` codes, stages, typed subjects, rejection ordering, and foundation conversion |
| `extension.rs` | closed `EditorCompositionExtensionV1` schema, deterministic typed codec, and validation |
| `legacy_import.rs` | one-way read-only `WorkspaceState`/profile to core plus extension conversion |
| `projection.rs` | pure composition plus extension to editor shell projection and route artifacts |
| `runtime.rs` | paired validated runtime state, installation, projection epoch, and read APIs |

Move the reusable editor projection DTO ownership from
`workspace/projection.rs` into this structural composition subsystem. The
temporary `project_workspace_for_shell` function remains only as a read-only
parity oracle inside `editor_shell`; production shell construction calls
`project_editor_composition_for_shell`.

`composition/build_editor_shell.rs` accepts a prepared projection artifact
instead of `&WorkspaceState`. Existing convenience constructors that accept a
workspace are restricted to crate tests and are not re-exported as active APIs.

`surface_provider.rs` and app provider request contracts gain
`MountedUnitId`. `ToolSurfaceInstanceId` may remain temporarily as editor
extension compatibility metadata, but it cannot be the request, session,
runtime-binding, liveness, or route-map key.

No catch-all module, alias facade, generic payload map, or `include!` is added.

## Legacy Import Contract

The importer is one-way and deterministic:

- composition definition ID derives from the editor profile identity, never a
  display label;
- primary `PresentationTargetId` is a stable editor target ID;
- each legacy host maps to one `RegionId` using its typed raw identity;
- split hosts map to fixed-point `RegionKind::Split` values;
- tab-stack hosts map to `RegionKind::Stack`, ordered by panel order and active
  mounted unit;
- each mounted tool/content instance maps to one `MountedUnitId` using the
  compatibility instance raw ID;
- stable editor content keys form typed `MountedContentRef` owner/profile/
  instance references;
- floating non-empty hosts become additional structural roots on the editor
  presentation target; bounds remain extension data;
- empty non-root stacks, panels without content, duplicate mounts, invalid
  fractions, unsupported stable keys, and incomplete registry mappings reject
  with actionable diagnostics rather than being guessed;
- capabilities map to stable namespaced `CapabilityId` values;
- unavailable content defaults to fallback projection; hide is permitted only
  by an explicit editor content profile declaration.

All built-in workspace profiles pass old-projection versus new-projection
parity tests for region shape, unit order, active content, static routes, and
mounted provider requests. Parity compares meaning, not old numeric widget IDs
when those are not public identity.

## Editor Extension Persistence

The editor extension profile is `runenwerk.editor.layout` schema version 1.
The app compatibility profile is `runenwerk.editor` with an explicit accepted
schema range.

`EditorCompositionExtensionV1` uses `#[serde(deny_unknown_fields)]`, sorted
typed records, LF endings, one trailing newline, and decode/re-encode byte
identity. Display labels do not establish identity or ordering.

`EditorCompositionSnapshotPort` implements
`CompositionExtensionSnapshotPort` and snapshots the complete validated editor
extension in one call. Save forms a `LayoutPromotion`, snapshots the extension,
and activates it through `CompositionBundleRepository` with an expected active
generation. It never writes core or extension documents independently.

Profile layout storage becomes a directory scope such as:

```text
editor-scenes/compositions/profile-<typed-profile-id>/
  active-generation.ron
  generations/...
```

The exact root remains app policy. The old `*.workspace.ron` path is never an
automatic fallback.

Loading validates the complete bundle and editor typed extension before
installing either state. A mismatch, unsupported extension schema, unknown
content profile, incompatible app profile, or invalid target binding rejects
atomically and leaves the current runtime unchanged.

## Legacy Persistence Read-only Gate

V1 through V5 workspace sources are unsupported. The app may read bytes only
to call `probe_composition_source` and produce a stable migration/reset
diagnostic. It must not deserialize a V5 workspace into live state, rewrite,
rename, migrate, delete, touch timestamps, or silently load a default.

Active `write_workspace_layout*`, `read_workspace_layout*`, and
`replace_workspace_state` APIs are removed. Old persisted DTOs and reducers are
restricted to `editor_shell` crate compatibility tests/import internals and are
not re-exported as active public mutation APIs. Final deletion remains WR-188.

The compile/source fitness guard rejects these terms in active app production
paths except the named read-only probe module:

- `WorkspaceMutation`;
- `reduce_workspace`;
- `replace_workspace_state`;
- `write_workspace_layout`;
- live `.workspace_state()` access.

## Provider, Session, And Content Liveness

Provider registries remain app-owned. Their requests, selected-provider facts,
sessions, route maps, viewport bindings, and pruning sets are keyed by
`MountedUnitId`.

Provider resolution reports all neutral liveness states:

| State | Editor cause example |
|---|---|
| `Resolved` | provider returned a valid frame |
| `Missing` | mounted unit has no app binding/provider registration |
| `Loading` | provider product or document is pending |
| `Suspended` | app session intentionally paused |
| `Denied` | lifecycle/capability policy rejects access |
| `UnsupportedProfile` | no provider supports the mounted content profile |
| `Crashed` | provider returned an execution failure |

Projection applies the required fallback order for every state:

1. app-provided unavailable-content frame;
2. neutral diagnostic placeholder carrying an inspection label and stable
   `editor_composition.*` diagnostic;
3. hidden only when the mounted-unit policy and host policy both permit it.

Unresolved content never changes or invalidates the structural graph.

## Static Interaction Policy

This checkpoint preserves current static chrome and content presentation. It
does not preserve legacy structural mutation on the branch because that would
retain a writable authority. Structural controls and docking/drop interaction
records are disabled with a visible/static-gate reason. Dispatching a stale
structural command returns `editor_composition.static.mutation_deferred` and
changes neither core nor extension state.

Non-structural behavior remains live: provider-local actions, document/domain
commands, toolbar product actions, viewport observation, focus, text input,
menus unrelated to structural mutation, and app sessions.

Viewport lifecycle may update viewport binding/settings in editor extension or
app runtime state; it may not mutate core structure. Profile switching installs
a complete built-in or persisted composition bundle atomically.

WR-186 restores structural interaction with Region Compass mechanics and
`ui_composition` transactions. This checkpoint adds no temporary mutation
adapter that WR-186 would need to delete.

## Diagnostics

Every rejection emits a stable `EditorCompositionDiagnosticRecord` with
`editor_composition.*` code, severity, stage, typed subject ID, actionable
message, and deterministic context ordering.

Required families cover:

- legacy import identity/topology/profile/fraction failures;
- missing/extra/duplicate extension records;
- unsupported extension schema and app compatibility;
- provider liveness and fallback selection;
- static structural mutation deferral;
- target-binding mismatch;
- bundle load/install atomicity;
- read-only V1-V5 rejection.

App/windowing policy failures remain app-owned and do not use the editor
composition namespace.

## Fitness Functions And Tests

`editor_shell` tests:

- every built-in profile imports into a valid `CompositionState`;
- legacy and composition static projections are meaning-equivalent;
- deterministic extension bytes are insertion-order independent and round-trip
  byte-identical;
- malformed extension coverage and core mismatch reject;
- projection does not form an intermediate `WorkspaceState`;
- static structural actions are omitted/disabled;
- legacy reducers/persisted writers are not active public re-exports.

`runenwerk_editor` tests:

- provider requests, sessions, routes, viewport bindings, and pruning are keyed
  by `MountedUnitId`;
- all seven liveness states follow fallback order without changing structure;
- shell startup and frame submission render from composition;
- built-in profile switching atomically installs composition;
- save/load uses linked generation bundles and editor typed extensions;
- corrupt or mismatched extension/core generations leave current runtime
  unchanged;
- V1-V5 source bytes, metadata, and timestamps remain unchanged;
- source guards prove no active app workspace reducer/writer/live-state access;
- startup/render smoke remains green with the intentional structural freeze.

Validation:

```text
cargo fmt --all --check
cargo test -p editor_shell composition
cargo test -p editor_shell workspace
cargo test -p runenwerk_editor composition
cargo test -p runenwerk_editor --test startup_render_smoke
task ui:dependencies
task docs:validate
task planning:validate
```

The runtime closeout additionally renders/checks production and roadmap
artifacts.

## Implementation Sequence

1. Add declared path dependencies and editor composition diagnostics.
2. Implement the deterministic editor extension schema and validator.
3. Implement one-way legacy/profile import with full built-in profile tests.
4. Move prepared editor projection DTO ownership and add composition projection
   parity tests.
5. Install `EditorCompositionRuntime` in shell state and route shell building
   from prepared composition projection.
6. Cut provider requests, sessions, route context, viewport bindings, mounted
   registries, and pruning to `MountedUnitId`.
7. Redirect runtime frame/input/picking/viewport consumers to composition
   projection artifacts.
8. Replace V5 writes/loads with editor composition bundle save/load and atomic
   install; reject V1-V5 read-only.
9. Remove active workspace mutators/re-exports and disable structural chrome
   and command dispatch with stable diagnostics.
10. Add architecture guards, liveness matrix, extension mismatch tests, and
    startup/render smoke proof.
11. Update editor app usage/current-state docs, run all gates, and create
    resolver-backed runtime, migration, and diagnostics evidence.

Run the phase-completion drift check before WR-184.

## Rollback And Stop Conditions

Rollback is checkpoint rejection: retain the previous checkpoint artifacts and
do not promote WR-184. Do not restore dual writes or automatically accept V1-V5
files.

Stop if:

- any live shell/runtime field retains `WorkspaceState` after import;
- any active app path invokes `WorkspaceMutation`, `reduce_workspace`, or a V5
  writer;
- projection reconstructs `WorkspaceState` from composition;
- core and editor extension can install partially;
- provider/session/runtime binding identity remains keyed only by
  `ToolSurfaceInstanceId`;
- a V1-V5 file is changed or accepted;
- static provider/product behavior regresses outside the declared structural
  interaction freeze;
- implementation requires `ui_surface`, adaptive, engine/windowing, native
  window, or redesigned docking changes;
- two writable structural authorities exist at any point after closeout;
- required scopes, tests, or evidence cannot prove the claims.

Closeout must report the intentional branch-local structural freeze and the
remaining WR-186/WR-188 work. It may claim `runtime_proven`, not final runtime
or perfectionist completion.
