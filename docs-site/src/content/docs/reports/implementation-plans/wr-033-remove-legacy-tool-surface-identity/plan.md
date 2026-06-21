---
title: WR-033 Remove Legacy Tool Surface Identity Contract
description: Promotion and implementation-readiness contract for removing ToolSurfaceKind as Workbench identity after typed Workbench handles land.
status: active
owner: editor
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-decision-register.md
related_adrs:
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
related_reports:
  - ../wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../../closeouts/wr-033-remove-legacy-tool-surface-identity/closeout.md
---

# WR-033 Remove Legacy Tool Surface Identity Contract

## Status

WR-033 is the second implementation-readiness slice inside
`PM-WB-CAP-001`. It removes legacy `ToolSurfaceKind` authority from the live
Workbench path after WR-032 supplies typed suite, surface, profile, provider
bundle, and composition handles.

This contract is the current-candidate implementation package for WR-033. It
does not implement product code by itself or complete PM-WB-CAP-001.

## Goal

Delete `ToolSurfaceKind` as Workbench identity instead of preserving a
compatibility enum under a new name.

The later implementation is acceptable when live Workbench state, provider
requests, shell commands, create candidates, frame artifacts, runtime bridge
queries, and Material Lab routing use stable surface keys and typed registry
data only. Legacy enum data may remain only where WR-035 intentionally rejects
old persisted layouts or reports unsupported-schema diagnostics.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and active milestone `PM-WB-CAP-001`. The milestone requires
  `ToolSurfaceKind` removal from Workbench identity, profile construction,
  provider requests, persisted layout authority, and Material Lab routing.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-033`.
  The row is `current_candidate`, blocker `B2`, depends on completed `WR-032`,
  and names removal of `ToolSurfaceKind`, stable-key reverse mapping helpers,
  and legacy profile constructors as the next evidence.
- `docs-site/src/content/docs/reports/implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md`
  and
  `docs-site/src/content/docs/reports/closeouts/wr-032-typed-suite-surface-profile-and-provider-handles/closeout.md`
  record the completed prerequisite typed-handle implementation.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is accepted and explicitly forbids `ToolSurfaceKind` as Workbench identity,
  persistence, provider request, profile construction, or Material Lab routing
  authority.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is the active design for the registry-owned Workbench host and provider-owned
  routing boundary.
- `domain/editor/editor_shell` owns app-neutral workspace, provider request,
  command, route, composition, and tool-suite contracts.
- `apps/runenwerk_editor` owns concrete shell dispatch, provider registry,
  Workbench host wiring, and runtime bridge behavior.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  WR-033 as `write_promotion_contract` after WR-032 completion and the WR-033
  blocker metadata repair.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-033`
  classified WR-033 next action as `write_promotion_contract`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after repairing the WR-033 blocker metadata from `B3` to `B2`.
- `task roadmap:promote -- --id WR-033 --state current_candidate --evidence
  "WR-032 typed handles completed with closeout evidence; WR-033 promotion
  contract validated and is the next PM-WB-CAP-001 dependency-order
  implementation slice."` passed.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-033`
  then classified WR-033 next action as `write_implementation_contract`.

## Readiness

Implementation verdict: WR-033 has a decision-complete current-candidate
implementation contract. Product code may start only after this contract update
is validated and `task ai:goal -- --track PT-WB-CAP --scope non-deferred`
still selects PM-WB-CAP-001 / WR-033 as the next implementation slice.

Blocking conditions before code starts:

- WR-032 is completed with closeout evidence. WR-033 must continue to rely on
  that typed composition path rather than inventing a replacement compatibility
  enum.
- The WR-033 roadmap write scopes were repaired on 2026-05-20 to cover the
  legacy helper and source-guard ownership required by this contract:
  `domain/editor/editor_shell/src/tool_suite/legacy.rs`,
  `domain/editor/editor_shell/src/tool_suite/mod.rs`,
  `domain/editor/editor_shell/src/tests.rs`, and this contract path are now
  listed alongside the workspace, provider, command, composition, app shell,
  and runtime-system scopes.
- Product code must stay bounded to the WR-033 write scopes and may not mark
  downstream WR-034, WR-035, or WR-036 evidence complete.

No new ADR is required while the implementation follows ADR 0012 and removes
the compatibility authority without moving semantics into `editor_shell`. Run
architecture governance before code if the implementation changes persistence
support, command ownership, provider execution authority, Material Lab semantic
ownership, or external component trust boundaries beyond this contract.

## Implementation Scope

Owning modules and exact change locations for the later implementation pass:

- `domain/editor/editor_shell/src/workspace/state.rs` module:
  remove `ToolSurfaceKind` as live identity from `ToolSurfaceState`,
  `WorkspaceDefaultToolSurface`, `TabStackState` lock metadata, compatibility
  reports, default workspace bootstrap helpers, and registry compatibility
  checks. Replace `ToolSurfaceState::new_legacy`,
  `ToolSurfaceState::legacy_tool_surface_kind`,
  `ToolSurfaceState::legacy_tool_surface_kind_or_error`,
  `WorkspaceDefaultToolSurface::new_legacy`,
  `WorkspaceDefaultToolSurface::legacy_tool_surface_kind`,
  `WorkspaceDefaultToolSurface::legacy_tool_surface_kind_or_error`,
  `compiled_in_legacy_tool_surface_state`,
  `compiled_in_default_tool_surface`, and
  `WorkspaceState::populate_stable_surface_keys_from_legacy` with
  stable-key/profile-definition equivalents or delete them when no longer
  needed.
- `domain/editor/editor_shell/src/workspace/reducer.rs` module:
  delete legacy mutation wrappers and legacy metadata parameters after stable
  callers exist. Target `WorkspaceMutation::add_panel_tab_legacy`,
  `WorkspaceMutation::split_tab_stack_area_legacy`,
  `WorkspaceMutation::reset_tab_stack_area_legacy`,
  `WorkspaceMutation::lock_tab_stack_area_type_legacy`,
  `WorkspaceMutation::replace_panel_tool_surface_kind_legacy`,
  `stable_key_for_legacy_surface`,
  `stable_key_for_legacy_surface_with_resolver`, and reducer branches that
  carry `legacy_tool_surface_kind` or `legacy_locked_tool_surface_kind` as live
  state.
- `domain/editor/editor_shell/src/workspace/profile.rs` module:
  remove legacy default profile constructors and default-surface lists that use
  `ToolSurfaceKind`. Keep profile construction registry-owned through
  `ToolSurfaceStableKey`, `PanelKind`, and WR-032 typed profile definitions.
- `domain/editor/editor_shell/src/workspace/definition_form.rs` module:
  remove authored-layout formation paths that reverse-map stable keys into
  `ToolSurfaceKind`. Authored Workbench layouts must form from stable keys,
  registry definitions, and panel metadata.
- `domain/editor/editor_shell/src/workspace/surface_contract.rs` module:
  replace any remaining surface-definition lookup that depends on legacy kind
  with registry-backed surface definitions or stable-key diagnostics.
- `domain/editor/editor_shell/src/surface_provider.rs` module:
  remove `SurfaceProviderRequest::legacy_tool_surface_kind`,
  `SurfaceProviderRequest::legacy_kind`,
  `SurfaceProviderRequest::legacy_kind_or_error`,
  `ResolvedSurfaceFrame::surface_kind`,
  `ToolSurfaceCreateCandidate::legacy_tool_surface_kind`,
  `EditorShellFrameModel::available_tool_surface_kinds`, and
  `EditorShellFrameModel::with_available_tool_surface_kinds` from the live
  provider and frame API.
- `domain/editor/editor_shell/src/commands/shell_command.rs` module:
  remove enum-backed shell command variants and fields:
  `SwitchPanelToolSurfaceKind`, `CreatePanelTab`,
  `SplitTabStackArea`, `ResetTabStackArea`, `LockTabStackAreaType`, and the
  `legacy_tool_surface_kind` or `legacy_locked_tool_surface_kind` fields on
  stable-key command variants.
- `domain/editor/editor_shell/src/commands/map_interactions.rs` module:
  stop mapping routes through enum-backed `RoutedShellAction` variants. Route
  tab, split, reset, lock, and create actions through stable-key commands only.
- `domain/editor/editor_shell/src/composition` module:
  remove composition helpers that derive route, label, panel, or provider
  behavior from `ToolSurfaceKind`. Keep generic graph routing provider-owned.
- `domain/editor/editor_shell/src/tool_suite/legacy.rs` module:
  remove reverse mapping helpers such as
  `tool_surface_kind_for_stable_key`,
  `stable_key_for_tool_surface_kind`,
  `stable_key_for_persisted_tool_surface_kind_v2`,
  and `stable_key_candidate_for_*` from the live Workbench API after the write
  scope is repaired. If WR-035 still needs persisted legacy decoding helpers,
  move them under persistence-only names and keep them out of provider/profile
  routing.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` module:
  update `build_tool_surface_create_candidates`,
  `mounted_surface_requests`, and `mounted_surface_requests_with_registry` so
  they derive requests, capabilities, route, label, provider family, and create
  candidates from `ToolSurfaceRegistry` and stable keys only.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` module:
  remove enum-backed command arms and legacy mutation calls, including
  dispatch for `ShellCommand::SwitchPanelToolSurfaceKind`,
  `ShellCommand::CreatePanelTab`,
  `ShellCommand::SplitTabStackArea`,
  `ShellCommand::ResetTabStackArea`, and
  `ShellCommand::LockTabStackAreaType`.
- `apps/runenwerk_editor/src/shell/applied_editor_definition` module:
  remove compatibility catalogs that expose available tool surfaces as
  `ToolSurfaceKind`; keep authored definitions expressed as stable surface keys
  and panel metadata.
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs` module:
  replace direct checks such as `ToolSurfaceKind::Viewport` with stable key or
  registry-backed surface-role checks.

Required implementation steps:

1. Finish WR-032 first. Stable typed handles and the composition builder must
   be implemented, validated, and closed out before WR-033 code starts.
2. Repair WR-033 write scopes to include the legacy helper module and any
   source-guard test file needed to prove removal.
3. Replace live workspace state and mutation APIs with stable-key-only fields.
   Keep `PanelKind` as layout/chrome grouping only; do not use it as provider
   or tool identity.
4. Remove enum-backed provider request and frame fields, then update app
   provider resolution to consume registry definitions.
5. Remove enum-backed shell commands and routes, then update app dispatch to
   mutate workspace state with stable keys and typed panel metadata.
6. Delete or quarantine reverse mapping helpers so provider, profile, command,
   and Material Lab routing cannot reach `ToolSurfaceKind`.
7. Add source guards that fail if Workbench identity modules reference
   `ToolSurfaceKind`, `tool_surface_kind_for_stable_key`, or
   `stable_key_for_tool_surface_kind` outside explicitly persistence-only
   unsupported-schema handling.
8. Keep old persisted layout rejection for WR-035. WR-033 must not silently
   migrate old layouts or claim clean persistence before the persistence row
   implements unsupported-schema diagnostics.

## Non-Goals

- No product code in this contract-writing slice.
- No downstream WR-034 registry-backed profile completion, WR-035 persistence
  cleanup, or WR-036 Material Lab proof beyond the minimum required to remove
  legacy live identity from WR-033-owned paths.
- No persistence format cleanup or old-layout unsupported-schema diagnostic;
  WR-035 owns that.
- No registry-backed profile completion beyond what is required to stop using
  legacy identity; WR-034 owns profile construction.
- No Material Lab clean migration proof beyond removing legacy route metadata;
  WR-036 owns proof in both host modes.
- No host command, product, or resource policy enforcement; WR-037 owns that.
- No product/service declaration plane; WR-038 owns that.
- No replacement compatibility enum.
- No external component, plugin, package trust, ABI, or sandbox work.

## Acceptance Criteria

- Live Workbench identity paths do not carry `ToolSurfaceKind`.
- `SurfaceProviderRequest`, `ResolvedSurfaceFrame`, create candidates, shell
  route actions, shell commands, workspace mutations, and runtime bridge checks
  use stable surface keys and typed registry/profile data only.
- Material Lab routing resolves through stable-key suite/profile/provider data,
  not material-specific legacy kind metadata.
- Reverse mapping helpers are removed from the live Workbench API or quarantined
  behind persistence-only unsupported-schema handling for WR-035.
- Source guards fail if Workbench code reintroduces `ToolSurfaceKind` authority
  in workspace state, provider requests, commands, composition, app shell
  providers, or runtime systems.
- Existing graph-canvas provider-owned routing remains semantic-free and does
  not grow tool-specific shell branches.
- No implementation claims stable-key-only persistence until WR-035 lands.

## Validation

Contract-writing validation:

```text
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-033 implementation closeout:

```text
cargo test -p editor_shell tool_suite
cargo test -p editor_shell workspace::profile
cargo test -p runenwerk_editor workbench_host
cargo test -p runenwerk_editor material_lab_workbench
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

The implementation validation must include a source guard for the banned live
Workbench references and focused tests for provider request construction,
create-surface candidates, shell command dispatch, Material Lab provider
resolution, and runtime bridge viewport targeting.

## Stop Conditions

Stop before code changes if:

- WR-032 is not completed with valid closeout evidence;
- WR-033 write scopes drift and again omit
  `domain/editor/editor_shell/src/tool_suite/legacy.rs`,
  `domain/editor/editor_shell/src/tool_suite/mod.rs`,
  `domain/editor/editor_shell/src/tests.rs`, or this contract path;
- implementation would leave `ToolSurfaceKind` in provider request, frame,
  command, mutation, or runtime bridge authority;
- implementation would silently migrate old persisted layouts instead of
  preserving WR-035 unsupported-schema work;
- implementation would replace `ToolSurfaceKind` with another compatibility
  enum;
- `editor_shell` would import app provider, Material Lab workflow, texture,
  procgen, render, runtime, or project IO types;
- Material Lab would route through app-specific shell branches instead of typed
  suite/profile/provider data.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md`;
- `task planning:validate` passed;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` passed after the
  contract-writing action and still reports PM-WB-CAP-001 next legal action as
  `execute_next_wr_implementation_contract`;
- confirmation that no product code or production state changed during this
  contract-writing action;
- remaining work: execute the WR-033 implementation slice and close it out
  before WR-034 can become dependency-complete.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-033 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.

## Contract-Writing Closeout Evidence

Status as of 2026-05-20: completed for the promotion-contract action and
current-candidate implementation-contract update only.

Changed artifact:

- `docs-site/src/content/docs/reports/implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md`

Validation:

- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after the WR-033 blocker metadata was repaired to `B2`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-033`
  reported `write_promotion_contract`, `WR-032:completed`, and a promotable
  preflight with the suggested `task roadmap:promote` command.
- `task roadmap:promote -- --id WR-033 --state current_candidate --evidence
  "WR-032 typed handles completed with closeout evidence; WR-033 promotion
  contract validated and is the next PM-WB-CAP-001 dependency-order
  implementation slice."` passed.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-033`
  reported `write_implementation_contract` for the current-candidate row.

Closeout result:

- No Rust product code changed.
- Roadmap metadata changed to repair WR-033 from `B3` to `B2` and then promote
  it to `current_candidate`.
- No production-track state changed.
- WR-033 is ready for its bounded implementation pass. Its roadmap write scopes
  include the legacy helper and source-guard ownership needed by the pass.
- Downstream PM-WB-CAP milestones remain dependency-waiting and must not be
  implemented from this contract alone.

## Perfectionist Closeout Audit

This promotion-contract action cannot honestly be `perfectionist_verified`
because it deliberately does not remove legacy identity. The known quality gap
is intentional: `ToolSurfaceKind` remains live until a later WR-033
implementation pass executes this contract after roadmap promotion.

The later implementation closeout must guard against:

- deleting the enum from one layer while provider requests or commands still
  carry it;
- leaving reverse mapping helpers reachable from normal Workbench code;
- treating `PanelKind` as replacement tool identity;
- claiming Material Lab clean routing while provider tests still require legacy
  material kinds;
- preserving old persistence migration behavior while claiming stable-key-only
  identity;
- descriptor-only source guards that miss app shell or runtime system call
  sites.

## Implementation Closeout Evidence

Status as of 2026-05-20: completed for the bounded WR-033 implementation
slice.

Changed closeout artifact:

- `docs-site/src/content/docs/reports/closeouts/wr-033-remove-legacy-tool-surface-identity/closeout.md`

Implementation evidence:

- Live Workbench state, provider requests, create candidates, frame artifacts,
  shell commands, routed actions, app dispatch, and runtime bridge lookups now
  use `ToolSurfaceStableKey` instead of `ToolSurfaceKind` as live identity.
- Legacy enum mapping remains only in explicit compatibility boundaries needed
  for authored or persisted legacy data and downstream rows.
- Material Lab provider routing resolves through stable suite/profile/provider
  metadata; provider requests no longer carry material-specific legacy kind
  fields.
- Source guards in `domain/editor/editor_shell/src/tests.rs` and
  `apps/runenwerk_editor/tests/viewport_architecture_guards.rs` now fail if
  the removed live legacy command, request, or chrome paths are reintroduced.

Validation:

- `cargo fmt` passed.
- `cargo test -p editor_shell tool_suite` passed with 31 matching tests.
- `cargo test -p editor_shell workspace::profile` passed with 26 matching
  tests.
- `cargo test -p editor_shell tool_surface_kind_usage_is_boundary_only_guard`
  passed.
- `cargo test -p runenwerk_editor --test viewport_architecture_guards` passed
  with 56 tests.
- `cargo test -p runenwerk_editor workbench_host` passed with 39 matching
  tests.
- `cargo test -p runenwerk_editor material_lab_workbench` passed with 8
  matching tests.
- `cargo check -p runenwerk_editor` passed.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after WR-033 was archived with closeout evidence.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after WR-033 was archived with closeout
  evidence.

Closeout result:

- WR-033 is ready to archive as `completed` with completion quality
  `bounded_contract`.
- PM-WB-CAP-001 remains active. WR-034, WR-035, and WR-036 still own the
  remaining clean-foundation work.
