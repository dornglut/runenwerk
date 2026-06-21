---
title: WR-034 Registry-Backed Workspace Profiles Contract
description: Promotion and implementation-readiness contract for building default Workbench profiles from installed registry composition data.
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
  - ../wr-033-remove-legacy-tool-surface-identity/plan.md
  - ../../closeouts/wr-032-typed-suite-surface-profile-and-provider-handles/closeout.md
  - ../../closeouts/wr-033-remove-legacy-tool-surface-identity/closeout.md
---

# WR-034 Registry-Backed Workspace Profiles Contract

## Status

WR-034 is the third implementation-readiness slice inside `PM-WB-CAP-001`.
It moves default Workbench profile composition from the central legacy profile
registry into installed suite/profile declarations consumed by
`WorkbenchCompositionBuilder`.

This contract is the promotion package for WR-034. It does not implement
product code, promote roadmap state, or complete PM-WB-CAP-001.

## Goal

Make full-editor and Material Lab workspace profiles registry-backed rather
than enum-backed.

The later implementation is acceptable when default workspace profiles are
formed only from installed profile declarations and registry-resolved stable
surface keys. The profile system must stop using central `ToolSurfaceKind`
lists as the source of default workspace composition.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and active milestone `PM-WB-CAP-001`. The milestone requires
  stable-key-only Workbench state, profiles, providers, persistence, and
  Material Lab routes.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-034`.
  The row is `ready_next`, blocker `B2`, depends on completed `WR-033`, and names
  installed suites and profile definitions as the default workspace authority.
- `docs-site/src/content/docs/reports/implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md`
  records the prerequisite typed profile and composition contract.
- `docs-site/src/content/docs/reports/implementation-plans/wr-033-remove-legacy-tool-surface-identity/plan.md`
  records the prerequisite removal contract for legacy enum authority.
- `docs-site/src/content/docs/reports/closeouts/wr-033-remove-legacy-tool-surface-identity/closeout.md`
  records completed WR-033 closeout evidence. Live Workbench identity,
  provider requests, shell commands, create candidates, frame artifacts,
  Material Lab routing, and runtime bridge targeting now use stable surface
  keys instead of `ToolSurfaceKind`.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  requires profile construction to stop treating `ToolSurfaceKind` as
  Workbench identity.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is the active design for registry-owned Workbench host composition.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-001 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-034`
  classified WR-034 next action as `write_promotion_contract`.
- `task planning:validate` passed after this contract was added, covering
  roadmap validation/check, production validation/check, and docs validation.

## Readiness

Promotion verdict: WR-034 can carry a decision-complete promotion and
implementation-readiness contract. WR-032 and WR-033 are completed with
closeout evidence, so the row can now be promoted to `current_candidate` after
this metadata refresh validates.

Blocking conditions before code starts:

- WR-033 must remain completed with valid closeout evidence. WR-034 depends on
  completed legacy identity removal because registry-backed profiles must not
  preserve `ToolSurfaceKind` default profile lists as a compatibility fallback.
- WR-032 typed profile and composition handles must remain completed before
  profile definitions can become the default workspace authority.
- If WR-032 does not land the necessary `ToolSuiteProfileDefinition` and
  `WorkbenchCompositionBuilder` profile validation APIs, WR-034 must stop for
  WR-032 repair instead of adding ad hoc profile types in `workspace/profile.rs`.
- The WR-034 roadmap write scopes were repaired on 2026-05-20 to include the
  app shell bootstrap, profile dispatch, provider, and source-guard ownership
  that this contract already requires:
  `apps/runenwerk_editor/src/shell/state.rs`,
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`,
  `apps/runenwerk_editor/src/shell/providers/mod.rs`,
  `apps/runenwerk_editor/src/shell/tests.rs`,
  `domain/editor/editor_shell/src/tests.rs`, and this contract path are now
  listed alongside the profile, state, and host composition owners.

No new ADR is required while the implementation follows ADR 0012, keeps
`editor_shell` app-neutral, and limits app host profile selection to
`apps/runenwerk_editor/src/shell/workbench_host.rs`. Run architecture
governance before code if profile declarations move source semantics,
provider execution, persistence format decisions, or app IO authority into a
new owner.

## Implementation Scope

Owning modules and exact change locations for the later implementation pass:

- `domain/editor/editor_shell/src/workspace/profile.rs` module:
  replace `default_workspace_profile_registry`,
  `compiled_in_legacy_workspace_profile`, `m6_workspace_profile`,
  `m6_workspace_profile_with_default_surfaces`,
  `compiled_in_legacy_default_surface`, and
  `WorkspaceProfile::legacy_default_tool_surfaces` as profile authority.
  Build `WorkspaceProfile` values from typed profile definitions and installed
  `ToolSurfaceRegistry` data. Preserve `WorkspaceProfileId`, labels, modes,
  document filters, and layout templates as profile metadata, not surface
  identity.
- `domain/editor/editor_shell/src/workspace/profile.rs::WorkspaceProfile::build_default_workspace_state_with_registry`:
  remove the compatibility step that calls
  `WorkspaceState::populate_stable_surface_keys_from_legacy`. The method must
  build from stable keys supplied by registry-backed profile declarations and
  fail closed when a profile references an uninstalled surface.
- `domain/editor/editor_shell/src/workspace/profile.rs::validate_tool_surface_registry_compatibility`:
  report unknown stable keys and profile declaration errors without
  classifying missing entries as "legacy surfaces" once WR-033 removes legacy
  identity.
- `domain/editor/editor_shell/src/workspace/state.rs` module:
  keep `WorkspaceState::bootstrap_tool_workspace_layout_with_stable_surfaces`
  as the layout builder for profile default surfaces, but remove any need for
  legacy enum metadata in the default path. Keep `PanelKind` as chrome/layout
  grouping only.
- `domain/editor/editor_shell/src/tests.rs` module:
  keep or add source guards proving central legacy default profile lists and
  `ToolSurfaceKind` profile construction do not return as profile authority.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` module:
  supply the full-editor and Material Lab profile definition bundles to
  `WorkbenchCompositionBuilder`. Expose profile data through the host so
  `RunenwerkEditorShellState` can bootstrap from the selected host
  composition instead of calling a global profile registry.
- `apps/runenwerk_editor/src/shell/state.rs` module:
  replace `default_workspace_profile_registry()` calls in
  `RunenwerkEditorShellState::new`,
  `RunenwerkEditorShellState::new_with_tool_surface_registry`, and
  `RunenwerkEditorShellState::new_for_workspace_profile_with_tool_surface_registry`
  with host/profile data passed from `RunenwerkWorkbenchHost` or an equivalent
  app-owned composition object.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` module:
  replace profile load and close dispatch that currently resolves profiles from
  `default_workspace_profile_registry()` with host-selected profile data.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` module:
  replace provider request paths that depend on the global profile registry
  with mounted surface/profile data from the active host composition.
- `apps/runenwerk_editor/src/shell/tests.rs` module:
  cover shell command/profile interactions that must continue to work after the
  global default profile registry stops being the app shell authority.

Required implementation steps:

1. Finish WR-032 and WR-033 first. WR-034 must not provide fallback typed
   handles or keep legacy enum identity alive to make profiles build.
2. Ensure `WorkbenchCompositionBuilder` validates profile definitions against
   installed surface definitions and rejects duplicate profile refs or unknown
   default surfaces before shell state is built.
3. Move the default surface lists for the full editor and Material Lab host
   into profile declarations owned by the installed Workbench composition.
4. Replace global profile-registry access in app shell bootstrap with
   host-selected profile data.
5. Update profile tests to prove full-editor default workspace and standalone
   Material Lab workspace mount from registry-backed profile declarations.
6. Keep `PanelKind` only as layout/chrome metadata paired with registry surface
   definitions. Do not infer provider, product, or route identity from
   `PanelKind`.
7. Add source guards that reject central legacy default profile lists,
   `compiled_in_legacy_workspace_profile`, and profile construction through
   `ToolSurfaceKind`.

## Non-Goals

- No product code in this contract-writing slice.
- No implementation before WR-032 and WR-033 complete.
- No persistence format cleanup or old-layout rejection; WR-035 owns that.
- No Material Lab clean migration proof beyond profile mount readiness;
  WR-036 owns proof across full-editor and standalone hosts.
- No host capability policy enforcement; WR-037 owns that.
- No product/service capability declarations; WR-038 owns that.
- No external dynamic components, package trust, ABI, or sandbox work.
- No global mutable profile registry.

## Acceptance Criteria

- Default profile construction is owned by installed suite/profile declarations
  consumed through the Workbench composition model.
- Full-editor and Material Lab hosts can select profile definitions from their
  installed composition without using central `ToolSurfaceKind` default lists.
- `WorkspaceProfile::build_default_workspace_state_with_registry` builds from
  stable keys and registry definitions only.
- Missing or duplicate profile declarations fail with explicit diagnostics.
- Material Lab standalone profile mounts graph, inspector, preview, texture,
  asset, diagnostics, and console surfaces from host profile data.
- Source guards prevent central legacy default profile constructors from
  returning as profile authority.

## Validation

Contract-writing validation:

```text
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-034 implementation closeout:

```text
cargo test -p editor_shell workspace::profile
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

The implementation validation must include focused coverage for full-editor
profile bootstrapping, Material Lab standalone profile bootstrapping, duplicate
profile rejection, unknown default surface rejection, and no central legacy
profile-list authority.

## Stop Conditions

Stop before code changes if:

- WR-032 or WR-033 is not completed with valid closeout evidence;
- WR-034 write scopes drift and again omit app shell bootstrap, dispatch,
  provider, test, or this contract path ownership required by this contract;
- profile declarations would require ad hoc typed handle APIs not supplied by
  WR-032;
- the implementation would keep `ToolSurfaceKind` default lists as fallback
  authority;
- shell state bootstrap cannot receive host-selected profile data without a
  global mutable registry;
- profile construction would silently drop unknown surfaces instead of failing
  closed;
- Material Lab standalone profile construction would depend on full-editor-only
  app internals instead of the host composition boundary.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task planning:validate` passed;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` passed after the
  contract-writing action and still reports PM-WB-CAP-001 next legal action as
  `prepare_wr_promotion_contract`;
- confirmation that no product code, roadmap planning state, or production
  state changed during the contract-writing action; roadmap evidence changed
  only to repair WR-034 write scopes;
- remaining work: promote WR-034 to `current_candidate`, rerun
  `task ai:goal -- --track PT-WB-CAP --scope non-deferred`, and execute the
  bounded implementation slice only if the generated coordinator selects it.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-034 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.

## Contract-Writing Closeout Evidence

Status as of 2026-05-20: completed for the contract-writing action only.

Changed artifact:

- `docs-site/src/content/docs/reports/implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md`

Validation:

- `task planning:validate` passed after the contract was added and after this
  closeout evidence was recorded, covering roadmap validation/check, production
  validation/check, and docs validation. It was rerun after the WR-034 roadmap
  write scopes were repaired to include app shell bootstrap, profile dispatch,
  provider, test/source-guard, and contract ownership.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` was rerun after
  validation and still reported PM-WB-CAP-001 next legal action as
  `prepare_wr_promotion_contract`.

Closeout result:

- No Rust product code changed.
- Roadmap evidence changed only to repair WR-034 write scopes. The row remains
  `ready_next`.
- No production-track state changed.
- WR-034 remained blocked for implementation by WR-033 completion at the time
  this contract was originally written.
- Downstream PM-WB-CAP milestones remain dependency-waiting and must not be
  implemented from this contract alone.

## Promotion Readiness Refresh

Status as of 2026-05-20: refreshed after WR-033 completion.

Readiness evidence:

- WR-032 has completed closeout evidence at
  `docs-site/src/content/docs/reports/closeouts/wr-032-typed-suite-surface-profile-and-provider-handles/closeout.md`.
- WR-033 has completed closeout evidence at
  `docs-site/src/content/docs/reports/closeouts/wr-033-remove-legacy-tool-surface-identity/closeout.md`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-034`
  reports `write_promotion_contract` and a promotable preflight for WR-034.

Promotion action:

- Promote WR-034 to `current_candidate` with evidence that WR-032 and WR-033
  are completed and this promotion contract has been refreshed.
- Run roadmap and production render/validate/check gates after promotion.
- Rerun `task ai:goal -- --track PT-WB-CAP --scope non-deferred` before any
  WR-034 product-code implementation.

## Perfectionist Closeout Audit

This contract-writing action cannot honestly be `perfectionist_verified`
because it deliberately does not change profile construction. The known quality
gap is intentional: central legacy profile authority remains until WR-032 and
WR-033 complete and a later WR-034 implementation pass executes this contract.

The later implementation closeout must guard against:

- profile definitions that are declared but not consumed by shell bootstrap;
- registry-backed tests that only inspect metadata and never build a workspace;
- `PanelKind` becoming replacement profile identity;
- Material Lab standalone tests using the full-editor default profile by
  accident;
- fallback profile lists that silently keep old `ToolSurfaceKind` authority
  alive.
