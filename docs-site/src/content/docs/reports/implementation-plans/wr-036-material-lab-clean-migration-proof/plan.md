---
title: WR-036 Material Lab Clean Migration Proof Contract
description: Promotion and implementation-readiness contract for proving Material Lab full-editor and standalone Workbench mounting without legacy metadata under PM-WB-CAP-001.
status: active
owner: apps/runenwerk_editor
layer: app/domain
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
related_adrs:
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-index.md
related_reports:
  - ../wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../wr-033-remove-legacy-tool-surface-identity/plan.md
  - ../wr-034-registry-backed-workspace-profiles/plan.md
  - ../wr-035-clean-persistence-format/plan.md
  - ../../closeouts/wr-034-registry-backed-workspace-profiles/closeout.md
  - ../../closeouts/wr-035-clean-persistence-format/closeout.md
---

# WR-036 Material Lab Clean Migration Proof Contract

## Goal

Establish implementation readiness for `WR-036` under `PM-WB-CAP-001` in the
`PT-WB-CAP` production track. The slice must prove Material Lab mounts through
the clean Workbench model in both full-editor and standalone Material Lab host
modes. Graph, inspector, preview, texture, asset, diagnostics, and console
surfaces must resolve through typed handles and provider bundles without
legacy tool-surface metadata.

This contract does not implement product code or close `WR-036`. It records
the dependency, ownership, and promotion conditions for the next Material Lab
migration proof now that WR-035 clean persistence is complete.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and active milestone `PM-WB-CAP-001`. The milestone requires
  Material Lab to mount in full-editor and standalone hosts without legacy
  tool-surface metadata.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-036`.
  The row is `ready_next`, blocker `B2`, depends on completed `WR-035`, and
  names Material Lab full-editor and standalone Workbench tests as required
  evidence.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is accepted and forbids restoring legacy surface-kind identity as a
  compatibility bridge.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is active and requires Workbench suites, profiles, provider requests, and
  hosted composition to be registry-owned.
- `docs-site/src/content/docs/reports/closeouts/wr-035-clean-persistence-format/closeout.md`
  records completed clean persistence evidence. App workspace layout reads
  reject V1-V4 workspace files, V5 reads reject legacy fallback metadata, and
  new V5 saves omit legacy fallback fields.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-001 next legal action as `prepare_wr_promotion_contract` after
  WR-035 was archived with closeout evidence.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-036`
  classified WR-036 next action as `write_promotion_contract` and reported
  promotion preflight as promotable.

## Readiness

Promotion verdict: WR-036 is promotable after this contract refresh. Product
code must not start until the roadmap row is promoted to `current_candidate`,
validation passes, and `task ai:goal -- --track PT-WB-CAP --scope non-deferred`
selects `execute_next_wr_implementation_contract`.

Blocking conditions before code starts:

- WR-035 is completed with closeout evidence, and the clean persistence gates
  passed after archival. Do not reopen WR-035 for Material Lab proof work.
- WR-036 is still `ready_next`, so implementation cannot begin until the
  roadmap promotion step and the scoped goal rerun select implementation.
- The WR-036 roadmap write scopes cover the app construction path required by
  this contract:
  `apps/runenwerk_editor/src/editor_app/state.rs` and this contract path are
  now listed alongside `apps/runenwerk_editor/src/material_lab`,
  `apps/runenwerk_editor/src/shell/workbench_host.rs`, and
  `apps/runenwerk_editor/src/shell/providers`.
- If the implementation requires editor_shell workspace persistence, profile
  construction, product/service capability contracts, host policy, or external
  component code, stop and repair the owning WR row before code starts.
- The proof may use existing domain/editor_shell APIs, but it must not move
  Material Lab semantics out of the app-owned Material Lab modules or add new
  shared domain semantics just to satisfy host wiring tests.

Architecture governance is not repeated by this contract because ADR 0012 is
accepted. Run `task ai:architecture-governance` again before code only if the
implementation changes dependency direction, host ownership, persistence
policy, provider-family authority, or the accepted clean-break contract.

## Implementation Scope

Owning modules and exact change locations:

- `apps/runenwerk_editor/src/material_lab/tool_suite.rs` module
  `material_lab::tool_suite` owns `material_lab_tool_suite` and
  `material_lab_surface`. Material Lab surfaces must remain stable-key-native
  declarations with provider families and provider-owned routes, and this file
  must not reintroduce `ToolSurfaceKind`.
- `apps/runenwerk_editor/src/editor_app/state.rs` module `editor_app::state`
  owns `RunenwerkEditorApp::new`,
  `RunenwerkEditorApp::new_material_lab_workbench`,
  `RunenwerkEditorApp::try_new_material_lab_workbench`,
  `RunenwerkEditorApp::with_workbench_host`,
  `RunenwerkEditorApp::surface_provider_registry`, and
  `RunenwerkEditorApp::workbench_host`. Full-editor and standalone proofs
  should use these app construction paths.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `shell::workbench_host` owns `RunenwerkWorkbenchHost::new`,
  `RunenwerkWorkbenchHost::material_lab`,
  `RunenwerkWorkbenchComposition::MaterialLab`,
  `material_lab_workbench_tool_suites`, and
  `provider_family_assignments_for_tool_suites`. This is the host composition
  boundary for proving full-editor versus standalone Material Lab suite and
  provider bundles.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` module
  `shell::providers` owns `EditorSurfaceProviderRegistry::runenwerk_default`,
  `EditorSurfaceProviderRegistry::runenwerk_material_lab_workbench`,
  `EditorSurfaceProviderRegistry::resolve_frame_with_provider_family_map`,
  `EditorSurfaceProviderRegistry::candidate_providers_for_request`,
  `mounted_surface_requests`, and `mounted_surface_requests_with_registry`.
  The proof must show mounted requests resolve through provider-family metadata
  and the host provider-family map, not by falling back to legacy kind routing.
- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs` module
  `shell::providers::material_graph_canvas` owns
  `MaterialGraphCanvasProvider`, `material_surface_action_command`, and
  `material_action_for_graph_canvas_action`. The graph surface must remain
  provider-owned and resolve from `runenwerk.material_lab.graph_canvas`.
- `apps/runenwerk_editor/src/shell/providers/material_inspector.rs` module
  `shell::providers::material_inspector` owns `MaterialInspectorProvider`.
  The inspector surface must resolve from `runenwerk.material_lab.inspector`
  without relying on legacy kind metadata.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs` module
  `shell::providers::material_preview` owns `MaterialPreviewProvider` and
  preview panel construction. The preview surface must resolve from
  `runenwerk.material_lab.preview` and preserve product-surface preview
  behavior.
- `apps/runenwerk_editor/src/shell/providers/texture_viewer.rs` and
  `apps/runenwerk_editor/src/shell/providers/volume_texture_viewer.rs` own
  texture support providers used by the standalone Material Lab composition.
- `apps/runenwerk_editor/src/shell/providers/asset_browser.rs`,
  `apps/runenwerk_editor/src/shell/providers/import_inspector.rs`,
  `apps/runenwerk_editor/src/shell/providers/m6_workspace.rs`, and
  `apps/runenwerk_editor/src/shell/providers/console/mod.rs` own the asset,
  diagnostics, and console support surfaces expected in the Material Lab host.
- `apps/runenwerk_editor/src/material_lab` runtime, state, renderer handoff,
  preview, and default-material modules may be touched only when the host proof
  needs existing Material Lab runtime setup to build frames. They must not gain
  Workbench routing or persistence ownership.

Explicit non-goals:

- Do not implement clean persistence or remove legacy persistence fields;
  WR-035 owns that.
- Do not implement host command, product, or resource policy; WR-037 owns
  that.
- Do not implement product/service capability declarations; WR-038 owns that.
- Do not implement additional multi-host presets beyond this Material Lab
  proof; WR-039 owns broader preset work.
- Do not add external dynamic components, plugins, or sandbox behavior; WR-040
  remains blocked and out of non-deferred scope.
- Do not move Material Lab semantics into `editor_shell` or another shared
  domain crate.

## Acceptance Criteria

The future WR-036 implementation is complete only when all criteria below are
true:

- Full-editor and standalone Material Lab hosts both mount Material Lab
  graph, inspector, preview, texture, asset, diagnostics, and console surfaces
  from hosted suite/profile/provider declarations.
- Mounted Material Lab requests have `provider_family_id` and route metadata
  from the hosted `ToolSurfaceRegistry`.
- Material Lab graph, inspector, and preview requests do not contain
  `legacy_tool_surface_kind` and do not rely on `ToolSurfaceKind` matching to
  select providers.
- Provider resolution for Material Lab requests uses
  `RunenwerkWorkbenchHost::provider_family_provider_map` with the concrete
  `EditorSurfaceProviderRegistry` for the active composition.
- The standalone Material Lab workbench installs only its intended support
  suites and providers: editor core, assets, diagnostics, texture, and
  Material Lab.
- Existing Material Lab graph interaction and material preview behavior remain
  intact after stable-key-only host resolution.
- No old persistence fallback or legacy workspace format is used as evidence
  for the proof.

## Implementation Steps

1. Inspect WR-035 closeout evidence and rerun
   `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-036`.
   Stop if WR-035 is not completed or if the action is no longer promotable.
2. In `apps/runenwerk_editor/src/shell/workbench_host.rs`, add focused tests
   that compare full-editor and standalone Material Lab host mounted requests
   through `mounted_surface_requests_with_registry` and the active host
   `ToolSurfaceRegistry`.
3. In `apps/runenwerk_editor/src/shell/workbench_host.rs`, assert that Material
   Lab graph, inspector, preview, texture, asset, diagnostics, and console
   requests in the Material Lab profile have provider-family metadata and can
   resolve frames through `resolve_frame_with_provider_family_map`.
4. Add negative assertions proving Material Lab graph, inspector, and preview
   mounted requests carry no `legacy_tool_surface_kind`.
5. If provider support still depends on
   `stable_key_or_legacy_kind_support`, narrow only the Material Lab proof path
   so the active requests and selected providers are stable-key-supported. Do
   not remove unrelated legacy provider fallback outside the WR-036 write
   scopes.
6. Touch `apps/runenwerk_editor/src/material_lab` runtime modules only if the
   frame-building proof needs deterministic Material Lab runtime setup.
7. Run the required validation and write closeout evidence under
   `docs-site/src/content/docs/reports/closeouts/wr-036-material-lab-clean-migration-proof/closeout.md`
   before changing roadmap or production evidence.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-036 implementation closeout:

```text
cargo test -p runenwerk_editor material_lab_workbench
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

Implementation validation must include full-editor and standalone Material Lab
host tests, provider-family request resolution tests, no-legacy-metadata
assertions for Material Lab graph/inspector/preview requests, and a check that
texture, asset, diagnostics, and console support surfaces resolve in the
standalone host.

## Stop Conditions

- Stop before product code if WR-035 is not completed with valid closeout
  evidence.
- Stop before product code if ADR 0012 is not accepted or the active design is
  not active.
- Stop if implementation requires write scopes outside WR-036.
- Stop if WR-036 write scopes drift and again omit Material Lab runtime, app
  construction, host composition, provider, or this contract path ownership.
- Stop if the proof needs changes in `domain/editor/editor_shell/src/workspace`
  persistence or profile code. That means WR-035 or WR-034 is incomplete or
  under-scoped.
- Stop if provider resolution still requires legacy `ToolSurfaceKind`
  metadata on active Material Lab requests.
- Stop if full-editor and standalone host behavior diverge through forked
  Material Lab routing code instead of suite/profile/provider composition.
- Stop if the proof depends on old persisted layouts, compatibility migration,
  or no-registry workspace loading.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` rerun result;
- confirmation that no product code or production state changed;
- confirmation that roadmap state changed only through the WR-036 promotion
  preflight command, if promotion is executed after this contract refresh;
- remaining blocker before product code: WR-036 must be selected by the next
  scoped goal rerun as the current implementation contract.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-036 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.

## Contract-Writing Closeout Evidence

Status as of 2026-05-20: refreshed for the promotion-contract action only.

Changed artifact:

- `docs-site/src/content/docs/reports/implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md`

Validation:

- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-036`
  reported WR-035 completed, WR-036 `ready_next`, and promotion preflight
  status `promotable`.
- `task roadmap:promote -- --id WR-036 --state current_candidate --evidence
  "<accepted evidence>"` promoted WR-036 to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`,
  `task production:check`, and `task docs:validate` passed after promotion.
- `task planning:validate` and
  `task ai:goal -- --track PT-WB-CAP --scope non-deferred` are required after
  this closeout evidence update.

Closeout result:

- No Rust product code changed.
- Roadmap evidence changed only to promote WR-036 from `ready_next` to
  `current_candidate` using the accepted promotion evidence from
  `task production:plan` and to remove stale WR-035 dependency wording from
  the WR-036 row.
- No production-track state changed.
- WR-036 remains incomplete until the scoped goal command selects
  `execute_next_wr_implementation_contract`.
- Downstream PM-WB-CAP milestones remain dependency-waiting and must not be
  implemented from this contract alone.

## Perfectionist Closeout Audit

This contract-writing action cannot honestly be `perfectionist_verified`
because it deliberately does not change Material Lab host behavior. The known
quality gap is intentional: Material Lab still needs a later implementation
pass after WR-035 proves clean persistence.

The later implementation closeout must guard against:

- proving only standalone Material Lab while the full-editor host still uses a
  different route;
- checking provider-family metadata without actually resolving frames through
  the active provider-family map;
- leaving Material Lab graph, inspector, or preview active requests with
  legacy kind metadata;
- treating legacy provider fallback support as proof that active requests are
  clean;
- omitting texture, asset, diagnostics, or console support surfaces from the
  standalone host proof;
- relying on old persisted layouts or registry-free workspace loading as
  evidence.
