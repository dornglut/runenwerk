---
title: WR-035 Clean Persistence Format Contract
description: Promotion and implementation-readiness contract for replacing legacy workspace persistence with stable-key-only schema under PM-WB-CAP-001.
status: active
owner: domain/editor/editor_shell
layer: domain/app
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
  - ../../closeouts/wr-034-registry-backed-workspace-profiles/closeout.md
---

# WR-035 Clean Persistence Format Contract

## Goal

Establish implementation readiness for `WR-035` under `PM-WB-CAP-001` in the
`PT-WB-CAP` production track. The slice must replace legacy workspace
persistence with a stable-key-only format: new workspace layout files round
trip through stable surface keys, and old persisted layouts that require
legacy tool-surface fields fail with clear unsupported-schema diagnostics
instead of being migrated.

This contract does not implement product code, promote roadmap state, or close
`WR-035`. It records the dependency and ownership conditions for a later
persistence cleanup pass.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and active milestone `PM-WB-CAP-001`. The milestone requires old
  persisted workspace schemas to fail with a clear unsupported-schema
  diagnostic and requires Workbench persistence to be stable-key-only.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-035`.
  The row is `ready_next`, blocker `B2`, depends on completed `WR-034`, and names
  deletion of V1-V4 migration loaders, V5 legacy fallback fields, unsupported
  old-schema tests, and stable-key-only round-trip tests as required evidence.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is accepted and explicitly rejects auto-migration of legacy Workbench
  layouts.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is active and requires old workspace layouts that depend on legacy
  surface-kind fields to fail with diagnostics rather than migrate through
  compatibility metadata.
- `docs-site/src/content/docs/reports/implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md`
  records the prerequisite registry-backed profile contract.
- `docs-site/src/content/docs/reports/closeouts/wr-034-registry-backed-workspace-profiles/closeout.md`
  records completed WR-034 closeout evidence. Full-editor and standalone
  Material Lab workspace profiles now build from installed Workbench profile
  declarations and host-owned registry-resolved stable surface keys, so WR-035
  can remove old persistence compatibility without removing the normal profile
  authority.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-001 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-035`
  classified WR-035 next action as `write_promotion_contract`.
- `task docs:validate` passed after this contract was added.
- `task planning:validate` passed after this contract was added, covering
  roadmap validation/check, production validation/check, and docs validation.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` was rerun after
  WR-034 closeout and selected PM-WB-CAP-001 next legal action as
  `repair_wr_promotion_metadata`; WR-035 is now the dependency-order candidate
  for promotion metadata repair.

## Readiness

Promotion verdict: WR-035 can be promoted to `current_candidate` after this
metadata refresh validates. WR-034 is completed with closeout evidence, ADR
0012 remains accepted, and the active Workbench host design remains active.

Blocking conditions before code starts:

- PM-WB-CAP-001 is still `active`, not completed.
- WR-035 is a breaking persistence cleanup. The implementation must keep ADR
  0012 as the authority for rejecting old layouts instead of introducing an
  implicit migration path.
- The WR-035 roadmap write scopes were repaired on 2026-05-20 to cover the
  persistence diagnostic owner required by this contract:
  `domain/editor/editor_shell/src/workspace/state.rs` and this contract path
  are now listed alongside
  `domain/editor/editor_shell/src/workspace/persisted.rs` and
  `apps/runenwerk_editor/src/persistence/workspace_layout.rs`.
- If the implementation needs profile, Workbench host, provider, command, or
  other app persistence modules beyond those owners, stop and repair the owning
  WR row before code starts.

Architecture governance is not repeated by this contract because ADR 0012 is
accepted. Run `task ai:architecture-governance` again before code only if the
implementation changes the accepted persistence break, changes dependency
direction, creates a new migration policy, or moves persistence authority into
app/runtime code.

## Implementation Scope

Owning modules and exact change locations:

- `domain/editor/editor_shell/src/workspace/persisted.rs` module
  `workspace::persisted` owns persisted DTOs and workspace structural
  persistence semantics. The implementation must remove normal support for
  `PersistedWorkspaceStateV1`, `PersistedWorkspaceStateV2`,
  `PersistedWorkspaceStateV3`, `PersistedWorkspaceStateV4`,
  `PersistedToolSurfaceStateV1`, `PersistedToolSurfaceStateV2`,
  `PersistedToolSurfaceStateV3`, `PersistedToolSurfaceKindV1`,
  `PersistedToolSurfaceKindV2`, and legacy `locked_tool_surface_kind` inputs
  from production decode paths.
- `domain/editor/editor_shell/src/workspace/persisted.rs` functions
  `WorkspaceState::to_persisted_v5` and `WorkspaceState::from_persisted_v5`
  own the new stable-key-only round-trip. `to_persisted_v5` must stop writing
  `legacy_tool_surface_kind` and `legacy_locked_tool_surface_kind` as fallback
  data, and `from_persisted_v5` must reject V5 layouts that rely on legacy
  aliases or missing registry-backed stable keys.
- `domain/editor/editor_shell/src/workspace/persisted.rs` functions
  `WorkspaceState::from_persisted_v1`,
  `WorkspaceState::from_persisted_v2`,
  `WorkspaceState::from_persisted_v3`, and
  `WorkspaceState::from_persisted_v4` are the old migration loaders. They must
  be removed from production load behavior or converted to explicit
  unsupported-schema diagnostics only.
- `domain/editor/editor_shell/src/workspace/state.rs` enum
  `WorkspaceStateError` already owns diagnostics such as
  `PersistedVersionUnsupported`, `PersistedSchemaViolation`,
  `PersistedStableSurfaceKeyUnknown`, and
  `PersistedTabStackLockStableKeyUnknown`. WR-035 may use those existing
  diagnostics through the allowed persistence owners. Adding a narrowly scoped
  `WorkspaceStateError` variant is now in scope only if the existing variants
  cannot distinguish the intentional clean break from accidental corruption.
- `apps/runenwerk_editor/src/persistence/workspace_layout.rs` module
  `persistence::workspace_layout` owns app read/write boundaries:
  `write_workspace_layout`, `write_workspace_layout_for_profile`,
  `write_workspace_layout_with_profile`,
  `read_workspace_layout_with_metadata_and_registry`,
  `read_workspace_layout_with_metadata_legacy_no_registry`,
  `read_workspace_layout_legacy_no_registry`, and
  `read_workspace_layout_with_optional_registry`. Production read paths must
  reject V1-V4 and legacy-dependent V5 layouts with the new diagnostic.
  Legacy/test-only readers must be removed or clearly confined to fixtures if
  they remain necessary for proving unsupported behavior.

Explicit non-goals:

- Do not change profile construction or default profile authority; WR-034 owns
  registry-backed profiles.
- Do not change Material Lab provider routing or mounting; WR-036 owns the
  Material Lab clean migration proof.
- Do not implement host command/product/resource policy; WR-037 owns that.
- Do not add compatibility migration, best-effort repair, silent fallback, or
  legacy enum replacement authority.
- Do not add external dynamic components or plugin loading.

## Acceptance Criteria

The future WR-035 implementation is complete only when all criteria below are
true:

- New app workspace layout saves write a stable-key-only V5 shape and do not
  serialize `legacy_tool_surface_kind`, `legacy_locked_tool_surface_kind`, or
  legacy `tool_surface_kind` fallback fields.
- App read paths reject V1, V2, V3, and V4 workspace layouts with a clear
  unsupported-schema diagnostic that explains old layouts are unsupported by
  the clean Workbench break.
- App read paths reject V5 layouts that depend on legacy aliases such as
  `locked_tool_surface_kind` or `legacy_tool_surface_kind`.
- Stable-key-only V5 layouts round trip through
  `read_workspace_layout_with_metadata_and_registry` using the real hosted
  `ToolSurfaceRegistry`.
- Unknown stable keys and unknown lock stable keys fail closed with diagnostics
  rather than falling back to legacy surface kinds.
- Tests that previously proved legacy V1-V4 migration are replaced with tests
  proving unsupported old-schema behavior.

## Implementation Steps

1. Inspect WR-034 closeout evidence and rerun
   `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-035`.
   Stop if WR-034 is not completed or if the action is no longer promotable.
2. Select an existing stable unsupported-schema diagnostic from
   `domain/editor/editor_shell/src/workspace/state.rs::WorkspaceStateError`
   where possible. Add a narrowly scoped variant only if existing diagnostics
   cannot represent the accepted clean-break behavior clearly.
3. In `domain/editor/editor_shell/src/workspace/persisted.rs`, remove legacy
   fallback serialization from `WorkspaceState::to_persisted_v5`.
4. In `domain/editor/editor_shell/src/workspace/persisted.rs`, make
   `WorkspaceState::from_persisted_v5` reject legacy alias/fallback fields and
   require registry-backed stable key validation.
5. In `apps/runenwerk_editor/src/persistence/workspace_layout.rs`, make
   `read_workspace_layout_with_optional_registry` reject V1-V4 versions before
   decoding into migration DTOs.
6. Remove or quarantine legacy no-registry readers so production code cannot
   accidentally load old layouts without the hosted registry.
7. Replace old migration tests with focused unsupported-schema tests and add a
   stable-key-only V5 app round-trip through the registry-backed read path.
8. Run the required validation and write closeout evidence under
   `docs-site/src/content/docs/reports/closeouts/wr-035-clean-persistence-format/closeout.md`
   before changing roadmap or production evidence.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-035 implementation closeout:

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

Implementation validation must include unsupported V1-V4 schema tests,
legacy-dependent V5 rejection tests, unknown stable-key rejection tests, and a
stable-key-only V5 round-trip through the app registry-backed reader.

## Stop Conditions

- Stop before product code if WR-034 is not completed with valid closeout
  evidence.
- Stop before product code if ADR 0012 is not accepted or the active design is
  not active.
- Stop if implementation requires write scopes outside WR-035.
- Stop if old-schema diagnostics would require error ownership outside
  `domain/editor/editor_shell/src/workspace/state.rs`.
- Stop if a proposed solution migrates or repairs old workspace layouts instead
  of rejecting them.
- Stop if stable-key-only V5 cannot round trip through the real app
  registry-backed reader.
- Stop if legacy no-registry readers remain reachable from production app load
  paths.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-035-clean-persistence-format/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` rerun result;
- confirmation that no product code, roadmap planning state, or production
  state changed; roadmap evidence changed only to repair WR-035 write scopes;
- remaining blockers: WR-034 completion before WR-035 code implementation.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-035 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.

## Contract-Writing Closeout Evidence

Status as of 2026-05-20: completed for the contract-writing action only.

Changed artifact:

- `docs-site/src/content/docs/reports/implementation-plans/wr-035-clean-persistence-format/plan.md`

Validation:

- `task docs:validate` passed after the contract was added.
- `task planning:validate` passed after the contract was added, covering
  roadmap validation/check, production validation/check, and docs validation.
  It was rerun after the WR-035 roadmap write scopes were repaired to include
  workspace diagnostic ownership and this contract path.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` was rerun after
  validation and still reported PM-WB-CAP-001 next legal action as
  `prepare_wr_promotion_contract`.

Closeout result:

- No Rust product code changed.
- Roadmap evidence changed only to repair WR-035 write scopes. The row remains
  `ready_next`.
- No production-track state changed.
- WR-035 remained blocked for implementation by WR-034 completion when this
  contract-writing action was originally completed.
- Downstream PM-WB-CAP milestones remain dependency-waiting and must not be
  implemented from this contract alone.

## Promotion Readiness Refresh

Status as of 2026-05-20: refreshed after WR-034 completion.

Readiness evidence:

- WR-034 has completed closeout evidence at
  `docs-site/src/content/docs/reports/closeouts/wr-034-registry-backed-workspace-profiles/closeout.md`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-035`
  reports `write_promotion_contract` and a promotable preflight for WR-035.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` reports
  PM-WB-CAP-001 next legal action as `repair_wr_promotion_metadata`, with
  WR-035 as the ready-next dependency-order row.

Promotion action:

- Promote WR-035 to `current_candidate` with evidence that WR-034 is completed
  and this promotion contract has been refreshed.
- Run roadmap and production render/validate/check gates after promotion.
- Rerun `task ai:goal -- --track PT-WB-CAP --scope non-deferred` before any
  WR-035 product-code implementation.

## Perfectionist Closeout Audit

This contract-writing action cannot honestly be `perfectionist_verified`
because it deliberately does not change persistence behavior. The known quality
gap is intentional: old workspace schemas still load until the later WR-035
implementation pass after WR-034 completion.

The later implementation closeout must guard against:

- saving a V5 file that still includes legacy fallback metadata;
- rejecting V1-V4 in one app path while legacy no-registry readers still load
  them from another production path;
- reporting generic decode failure instead of the accepted unsupported-schema
  diagnostic;
- using registry-free loading for stable-key-only files;
- preserving V1-V4 migration tests as evidence of the new clean break;
- claiming clean persistence before Material Lab's WR-036 no-legacy-metadata
  proof has run.
