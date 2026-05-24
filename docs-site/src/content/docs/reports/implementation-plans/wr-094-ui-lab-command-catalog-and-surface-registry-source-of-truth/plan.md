---
title: WR-094 UI Lab Command Catalog And Surface Registry Contract
description: Design-first implementation contract for PM-UI-LAB-002 command catalog and registry-owned surface metadata.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../design/accepted/ui-lab-command-catalog-and-surface-registry-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-094 UI Lab Command Catalog And Surface Registry Contract

## Goal

Implement `PM-UI-LAB-002` by making editor command affordances and tool-surface
metadata derive from one typed source per boundary.

This contract covers only the command catalog and surface registry source of
truth. It must not implement the visual Editor Lab shell, visual authoring
operations, project IO, preview evidence capture, or public API closeout.

Expected production outcome:

- menu, toolbar, keybinding, palette, route, availability, disabled reason, and
  dispatch projections come from `EditorCommandCatalog`;
- tool-surface label, provider family, route kind, capabilities, retention,
  persistence, and creation policy come from the installed registry;
- legacy route strings and `ToolSurfaceKind` remain compatibility edges, not
  normal extension points;
- runtime behavior is proven in the launched editor before PM-UI-LAB-002 can
  close as `runtime_proven`.

## Source Of Truth

- Production milestone: `PM-UI-LAB-002` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-094` in the roadmap source set.
- Accepted design:
  `docs-site/src/content/docs/design/accepted/ui-lab-command-catalog-and-surface-registry-design.md`.
- Productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`.
- Existing command code:
  `apps/runenwerk_editor/src/shell/command_resolution.rs`,
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`,
  `apps/runenwerk_editor/src/shell/toolbar_adapter.rs`, and
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs`.
- Existing surface code:
  `domain/editor/editor_shell/src/tool_suite/definition.rs`,
  `domain/editor/editor_shell/src/tool_suite/registry.rs`,
  `domain/editor/editor_shell/src/workspace/surface_contract.rs`,
  `domain/editor/editor_shell/src/workspace/state.rs`, and
  `apps/runenwerk_editor/src/shell/providers/mod.rs`.

## Readiness

Initial action classification from `task production:plan` was `design_first`
because `WR-094` entered the roadmap as `blocked_deferred` intake.

This contract is the design-first planning artifact that cleared the initial
policy-deferred blocker. After the accepted design and this contract existed,
`WR-094` was moved to `ready_next` with B2 metadata and then promoted to
`current_candidate`.

Current action classification from `task production:plan` is
`write_implementation_contract`. The current readiness report says:

```text
Production milestone state: active
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-004:support_only, WR-046:support_only
Next action: write_implementation_contract
```

This document is the decision-complete implementation contract requested by the
workflow. This action only writes the contract; it does not change product code.

Product code may start only in a later action after:

- this implementation contract has passed docs, roadmap, production, diagram,
  whitespace, and `task ai:goal` checks;
- any current-candidate switch or promotion metadata repair named by the
  workflow is completed;
- the implementation starts inside the exact WR-094 write scopes;
- the implementation contract remains aligned with the accepted design and the
  current source state.

## Promotion History

`WR-094` was promoted because the required design-first evidence was present and
bounded:

- the accepted PM-UI-LAB-002 design names the command catalog and surface
  registry contracts, ownership boundaries, diagnostics, migration strategy,
  and validation expectations;
- the active productization design keeps `ui_definition` behavior-free and
  keeps editor/game/app execution in their owning domains;
- PM-UI-LAB-001 closeout proves the governance and code-truth reconciliation
  prerequisite;
- the roadmap row has disjoint write scopes and only support-only dependencies;
- this contract defines exact modules, non-goals, gates, implementation steps,
  validation, runtime evidence, stop conditions, and closeout requirements;
- no product code has been changed for PM-UI-LAB-002 before promotion.

Promotion command used:

```text
task roadmap:promote -- --id WR-094 --state current_candidate --evidence "Accepted PM-UI-LAB-002 command catalog and surface registry design plus WR-094 implementation contract clear the design-first blocker; production:plan reports WR-094 promotable."
```

Post-promotion checks run:

```text
task roadmap:validate
task roadmap:render
task roadmap:check
task production:validate
task production:render
task production:check
task puml:validate
task docs:validate
task production:plan -- --milestone PM-UI-LAB-002 --roadmap WR-094
task ai:goal -- --track PT-UI-LAB --scope non-deferred
```

The post-promotion goal output now reports
`execute_next_wr_implementation_contract`; this document is that implementation
contract update.

## Promotion Readiness Evidence

Evidence supporting promotion from `ready_next` to `current_candidate`:

- accepted design:
  `docs-site/src/content/docs/design/accepted/ui-lab-command-catalog-and-surface-registry-design.md`;
- active productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`;
- PM-UI-LAB-001 closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-001-productization-governance/closeout.md`;
- this decision-complete WR-094 contract;
- production milestone `PM-UI-LAB-002` is active and links `WR-094`;
- roadmap dependencies `WR-004` and `WR-046` are support-only context;
- validation passed after metadata repair:
  `task roadmap:validate`, `task roadmap:render`,
  `task production:validate`, `task production:render`,
  `task production:check`, `task roadmap:check`, `task docs:validate`,
  and `task puml:validate`.

Promotion evidence text to use:

```text
Accepted PM-UI-LAB-002 command catalog and surface registry design plus WR-094 implementation contract clear the design-first blocker; production:plan reports WR-094 promotable.
```

## Implementation Contract Decisions

The implementation must use a Strangler migration. Existing command and surface
entry points may remain as compatibility adapters while the first slice moves
normal projections behind catalog and registry contracts. The adapters must not
remain independent source tables after the slice closes.

Source truth decisions:

- `apps/runenwerk_editor/src/shell/command_catalog/` owns concrete Runenwerk
  editor command descriptors, aliases, dispatch targets, projections,
  availability, labels, disabled reasons, and typed command diagnostics.
- `domain/editor/editor_shell` owns app-neutral shell model projection types,
  route action data, and tool-surface registry contracts.
- `domain/editor/editor_definition` may declare editor-owned references to
  commands, menus, workspaces, themes, bindings, and surfaces, but does not
  execute commands or mount providers.
- `domain/ui/ui_definition` remains generic and behavior-free.
- `apps/runenwerk_editor` owns runtime dispatch, provider mounting, dynamic
  availability, diagnostics display, and launched-editor evidence.

Complete source-to-runtime chain:

1. Catalog descriptors define command keys, aliases, labels, availability, route
   targets, dispatch targets, keybinding/menu/toolbar/palette projections, and
   typed diagnostics.
2. Binding and authored UI data resolve command references through the catalog.
3. Shell-frame composition receives route actions from catalog-backed model
   data, not from command-specific fallback logic.
4. Runtime dispatch consumes catalog dispatch targets and reports typed
   unavailability diagnostics.
5. Tests and launched-editor evidence prove menu, toolbar, workspace switching,
   disabled reasons, and unavailable-command states consume the catalog.
6. Registry definitions define surface identity, provider family, capabilities,
   retention, creation policy, route kind, persistence, and display metadata.
7. Workspace helpers and providers consume installed registry definitions before
   legacy `ToolSurfaceKind` compatibility.
8. Tests and launched-editor evidence prove representative surfaces mount from
   registry metadata.

Forbidden completion shortcuts:

- descriptor-only catalog data with dispatch still using an independent truth
  table;
- status-panel-only evidence without launched-editor command interaction;
- fallback-only route resolution that hides missing catalog descriptors;
- provider mounting that still depends on hardcoded legacy kind tables for
  normal surfaces;
- generic string diagnostics such as `[ui] command unavailable` where a typed
  catalog diagnostic is possible;
- moving editor command behavior, provider behavior, activation behavior, or
  project IO into `ui_definition`.

Architecture guard tests must fail if:

- a command appears in menus, toolbar, palette, bindings, or routes without a
  catalog descriptor;
- a route action can only be produced by `toolbar_action_for_route_slot` for a
  normal catalog command;
- dispatch emits only the old generic unavailable message for a catalog-known
  command;
- an installed surface lacks registry-owned provider family, capability set,
  retention class, route kind, persistence, or creation policy;
- normal provider support requires `ToolSurfaceKind` instead of stable keys and
  provider-family metadata.

Public API, migration, and persistence impact:

- Keep focused public entry points. Do not glob-export internal command catalog
  or registry implementation modules as the normal user path.
- Preserve legacy persisted `ToolSurfaceKind` and route string migration paths,
  but treat them as compatibility edges with typed failure diagnostics.
- Update design, production, roadmap, and closeout docs if implementation finds
  that any accepted ownership boundary is wrong.

## Non-Goals

- Do not implement the app-hosted Editor Lab surface shell; that is
  `PM-UI-LAB-003`.
- Do not implement operation-driven visual authoring; that is
  `PM-UI-LAB-004`.
- Do not implement Editor Lab project IO, diff/apply, activation, or rollback;
  that is `PM-UI-LAB-005`.
- Do not implement preview scenario capture or runtime evidence infrastructure
  outside the evidence needed for PM-UI-LAB-002 closeout; the broader matrix is
  `PM-UI-LAB-006`.
- Do not broaden public API/docs/examples beyond command catalog and surface
  registry usage needed for this slice; final API closeout is
  `PM-UI-LAB-007`.
- Do not move editor command behavior, provider behavior, activation behavior,
  or project IO into `domain/ui/ui_definition`.

## Implementation Scope

### Command Catalog

Add `apps/runenwerk_editor/src/shell/command_catalog/` with:

- `EditorCommandCatalog`;
- `EditorCommandDescriptor`;
- `EditorCommandKey`;
- `EditorCommandAlias`;
- `EditorCommandAvailability`;
- `EditorCommandProjection`;
- typed catalog diagnostics.

Refactor `apps/runenwerk_editor/src/shell/command_resolution.rs` so
`KnownEditorCommand::{from_key,key,all,to_routed_shell_action,to_shell_command}`
and `active_route_actions_by_target` delegate to the catalog or become catalog
compatibility adapters. `KnownEditorCommand` may remain temporarily, but it must
not remain an independent truth table.

Refactor `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` so
`dispatch_toolbar_command` consumes catalog dispatch targets or a catalog-owned
command enum. Availability and disabled reasons must not live only in dispatch.

Refactor `domain/editor/editor_shell/src/composition/build_editor_shell.rs` so
normal command routes are supplied by
`EditorShellFrameModel::route_actions_by_route_target`. Remove normal reliance
on `toolbar_action_for_route_slot`; if compatibility fallback remains during
migration, tests must prove all catalog routes resolve before fallback and the
fallback is marked as legacy-only.

Validate `assets/editor/ui/editor_bindings.ron` against the catalog. Every
route, menu item command, and dynamic availability reference must resolve to a
catalog descriptor or fail with a typed diagnostic.

### Surface Registry

Extend `domain/editor/editor_shell/src/tool_suite/definition.rs::ToolSurfaceDefinition`
to carry registry-owned metadata required by PM-UI-LAB-002:

- `SurfaceCapabilitySet`;
- `SessionRetentionClass`;
- creation policy;
- target-profile or host compatibility metadata where current profiles need it;
- legacy compatibility key only where migration still needs it.

Extend `domain/editor/editor_shell/src/tool_suite/registry.rs::ToolSurfaceRegistry`
with lookup and iteration APIs for the new metadata. Registry validation must
reject missing capabilities, missing retention class, invalid provider family,
duplicate stable key, and invalid creation policy.

Refactor `domain/editor/editor_shell/src/workspace/surface_contract.rs` so these
functions are registry-first and legacy-only as fallback:

- `editor_surface_definitions`;
- `tool_surface_definition_id`;
- `tool_surface_display_metadata`;
- `tool_surface_capability_set`;
- `tool_surface_session_retention_class`;
- definition-key and semantic-key conversion helpers.

Keep `domain/editor/editor_shell/src/workspace/state.rs::ToolSurfaceKind` only
for legacy persistence and migration. Normal APIs must carry
`ToolSurfaceStableKey` and registry definitions.

Refactor `apps/runenwerk_editor/src/shell/providers/mod.rs` so provider support
uses stable keys and registry provider-family metadata. Rename
`stable_key_or_legacy_kind_support` and `stable_keys_or_legacy_kind_support`
unless they actually perform legacy migration.

## Acceptance Criteria

- All authored command routes in `assets/editor/ui/editor_bindings.ron` resolve
  through `EditorCommandCatalog`.
- Menu, toolbar, keybinding, palette, route action, dispatch, availability, and
  disabled reason projections use the same descriptor data.
- Stale disabled reasons are removed or replaced with catalog diagnostics.
- The shell frame builder no longer needs command-specific route fallback for
  normal catalog routes.
- Surface definitions, capability sets, retention classes, provider families,
  creation policy, and display metadata resolve from `ToolSurfaceRegistry`.
- `ToolSurfaceKind` is still accepted only at legacy persistence/migration
  boundaries.
- Provider support and provider-family assignment are stable-key and
  registry-driven.
- The launched editor proves normal menu/toolbar/workspace commands and
  provider surface mounting still work after the refactor.

## Tests And Validation

Implementation steps after the workflow authorizes code:

1. Add the app-owned `EditorCommandCatalog` modules and descriptor diagnostics
   in `apps/runenwerk_editor/src/shell/command_catalog/`.
2. Convert `command_resolution.rs` command lookup, route projection, aliases,
   and legacy adapters to catalog-backed data.
3. Convert dispatch and toolbar/menu/keybinding projection to consume catalog
   descriptors and typed availability results.
4. Remove normal command fallback from the shell frame builder, keeping any
   temporary fallback explicitly legacy-only and test-covered.
5. Add binding validation that fails unresolved routes, menu commands, and
   availability references with typed diagnostics.
6. Extend `ToolSurfaceDefinition` and `ToolSurfaceRegistry` with the
   registry-owned metadata named by this contract.
7. Refactor workspace surface contract helpers to read metadata from the
   installed registry before legacy compatibility paths.
8. Refactor provider support to use stable keys and registry provider-family
   metadata.
9. Add focused unit and integration tests for catalog projection, dispatch
   diagnostics, registry validation, provider mounting, and legacy boundaries.
10. Launch the editor and capture PM-UI-LAB-002 runtime evidence before
    closeout.

Focused tests:

- catalog descriptor uniqueness and alias uniqueness;
- every `editor_bindings.ron` route, menu command, and availability reference
  resolves to exactly one descriptor;
- `active_route_actions_by_target` produces the expected route actions from
  catalog data;
- shell frame tests fail if normal command routes depend on
  `toolbar_action_for_route_slot`;
- dispatch tests prove unavailable commands surface typed diagnostics instead
  of only appending `[ui] command unavailable`;
- registry tests prove every installed surface has provider family,
  capability set, retention class, route kind, persistence, and creation
  policy;
- provider tests prove stable-key support and provider-family matching;
- legacy tests prove old `ToolSurfaceKind` data maps or fails closed with a
  typed diagnostic.

Required commands:

```text
cargo test -p editor_shell
cargo test -p runenwerk_editor command
cargo test -p runenwerk_editor surface
task docs:validate
task production:validate
task production:render
task production:check
task roadmap:validate
task roadmap:check
```

Runtime evidence before PM-UI-LAB-002 closeout:

- launch the editor;
- exercise File/Edit/Window/Workspace command affordances;
- switch workspaces, including Editor Design and Material workspaces;
- verify disabled commands show catalog reasons or diagnostics;
- mount representative Scene, Editor Design, Material Lab, and diagnostics
  surfaces from registry metadata;
- capture screenshots or equivalent runtime artifacts for success and
  unavailable-command states.

## Stop Conditions

Stop before product code if:

- `WR-094` remains `blocked_deferred`;
- `task production:plan` does not classify the row as promotable or
  implementable;
- implementation would require `ui_definition` to own editor command behavior;
- implementation would make `ToolSurfaceKind` the normal source of truth again;
- catalog and registry paths require a new ADR not covered by the accepted ADRs;
- validation requires editing unrelated roadmap rows or adjacent production
  milestones;
- the launched editor cannot be used to prove command and surface behavior.

## Closeout Requirements

PM-UI-LAB-002 may close only after:

- code, tests, docs, roadmap, and production metadata are updated;
- runtime evidence is captured and linked from a completed closeout report;
- `task ai:goal -- --track PT-UI-LAB --scope non-deferred` reports
  PM-UI-LAB-003 as the next incomplete milestone;
- remaining known gaps are listed truthfully.

Expected completion quality: `runtime_proven` if the editor runtime evidence is
captured. If runtime evidence cannot be captured, the milestone must not close
as complete.

## Perfectionist Closeout Audit

Do not claim `perfectionist_verified` for WR-094 or PM-UI-LAB-002. The
perfectionist/no-gap audit remains a later `PT-UI-LAB` follow-up after the
Editor Lab product track is runtime-proven.

Known gaps that may remain after PM-UI-LAB-002:

- visual Editor Lab shell remains PM-UI-LAB-003;
- operation-driven visual authoring remains PM-UI-LAB-004;
- project IO, diff/apply, and rollback remain PM-UI-LAB-005;
- preview evidence matrix remains PM-UI-LAB-006;
- API/docs/examples closeout remains PM-UI-LAB-007.
