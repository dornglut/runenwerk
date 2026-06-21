---
title: WR-039 Multi-Host Presets Contract
description: Promotion and implementation-readiness contract for full-editor, Material Lab, headless validation, and constrained Workbench host presets under PM-WB-CAP-004.
status: active
owner: apps/runenwerk_editor
layer: app shell / domain composition
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
related_adrs:
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
related_reports:
  - ../wr-037-host-capability-policy/plan.md
  - ../wr-038-product-and-service-capability-declarations/plan.md
  - ../../closeouts/wr-037-host-capability-policy/closeout.md
  - ../../closeouts/wr-038-product-and-service-capability-declarations/closeout.md
---

# WR-039 Multi-Host Presets Contract

## Goal

Establish promotion and implementation readiness for `WR-039` under
`PM-WB-CAP-004`. The slice must make full-editor, standalone Material Lab,
headless validation, and constrained hosts explicit Workbench composition
presets instead of app forks or legacy route branches.

This contract does not implement product code or close `WR-039`. It records the
bounded host-preset work needed after `WR-037` and `WR-038`: host policy and
declaration contracts exist, so the next step is making host modes differ only
by installed suites, profiles, provider-family mappings, and capability policy.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and `PM-WB-CAP-004`. The milestone requires full-editor,
  standalone Material Lab, headless validation, and constrained hosts to share
  the same clean Workbench model.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-039`.
  The row entered implementation as `current_candidate`, blocker `B2`, depends
  on completed `WR-037`, and names `cargo test -p runenwerk_editor workbench_host` plus
  `cargo check -p runenwerk_editor` as required validation.
- `docs-site/src/content/docs/reports/closeouts/wr-037-host-capability-policy/closeout.md`
  records completed host policy enforcement before provider proposal mutation.
- `docs-site/src/content/docs/reports/closeouts/wr-038-product-and-service-capability-declarations/closeout.md`
  records completed declarative product and service needs without moving
  semantic authority into `editor_shell`.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is accepted and blocks reintroducing legacy surface-kind identity in host
  presets.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-004 next legal action as `repair_wr_promotion_metadata`.
- `task production:plan -- --milestone PM-WB-CAP-004 --roadmap WR-039`
  reported promotion preflight metadata blocked only because WR-039 was `B3`.
- WR-039 blocker metadata was repaired from `B3` to `B2`.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`,
  `task production:check`, `task docs:validate`, and
  `task planning:validate` passed after the repair.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` then classified
  PM-WB-CAP-004 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-004 --roadmap WR-039`
  reported WR-039 as promotable and classified the next action as
  `write_promotion_contract`.
- `task roadmap:promote -- --id WR-039 --state current_candidate --evidence
  "<accepted evidence>"` promoted WR-039 to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`,
  `task production:check`, `task docs:validate`, and
  `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` then classified
  PM-WB-CAP-004 next legal action as `execute_next_wr_implementation_contract`.
- `task production:plan -- --milestone PM-WB-CAP-004 --roadmap WR-039`
  reported WR-039 as the active implementation contract.
- `cargo fmt`, `cargo test -p runenwerk_editor workbench_host`, and
  `cargo check -p runenwerk_editor` passed for the implementation.
- `docs-site/src/content/docs/reports/closeouts/wr-039-multi-host-presets/closeout.md`
  records the bounded implementation closeout.

Implementation outcome:

- `apps/runenwerk_editor/src/shell/workbench_host.rs` now exposes explicit
  `RunenwerkWorkbenchComposition` variants and constructors for full editor,
  standalone Material Lab, headless validation, and constrained hosts.
- Headless validation and constrained presets use the same
  `WorkbenchCompositionBuilder` path as the existing full-editor and Material
  Lab hosts.
- Headless validation installs the editor, asset, and diagnostics suites with a
  runtime-debug profile and deny-all host policy.
- Constrained hosts install the full editor suite/profile/provider set but use
  deny-all host policy so unsupported capabilities are denied through policy
  instead of hidden through route branches.
- `explicit_workbench_presets_are_composition_data` proves suite, profile,
  provider-family, and policy differences are explicit composition data.

## Readiness

Promotion verdict: WR-039 is promotable after this contract is recorded and the
roadmap/production/doc gates pass. Product code must not start until the
roadmap row is promoted to `current_candidate`, validation passes, and the
scoped goal command selects implementation.

Existing code state:

- `apps/runenwerk_editor/src/shell/workbench_host.rs` owns
  `RunenwerkWorkbenchHost`, `RunenwerkWorkbenchComposition`, installed suite
  selection, provider-family assignment filtering, compiled workspace profile
  specs, workspace profile registry construction, provider bundle wiring, and
  host policy storage.
- `RunenwerkWorkbenchHost::new()` already builds the full editor and
  `RunenwerkWorkbenchHost::material_lab()` already builds the standalone
  Material Lab host through the same `from_composition` path.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` owns
  `WorkbenchCompositionBuilder`, suite/profile/provider validation, capability
  declaration sidecars, and `HostCapabilityPolicy` as composition data.
- `apps/runenwerk_editor/src/shell/tool_suites` and
  `apps/runenwerk_editor/src/material_lab/tool_suite.rs` own compiled suite
  declarations. They should be reused by presets instead of duplicating surface
  lists.

Blocking conditions before code starts:

- WR-039 must be promoted to `current_candidate`.
- If implementation needs new product semantics, product graph formation,
  service execution, or domain validation, stop. Those remain outside
  Workbench host preset ownership.
- If implementation needs external dynamic components, plugin package trust,
  sandboxing, permissions, unload/reload, or runtime process isolation, stop.
  WR-040 is blocked and out of the non-deferred scope.
- If implementation needs to restore `ToolSurfaceKind` as runtime identity,
  stop. ADR 0012 forbids that compatibility path.

## Implementation Scope

Owning modules and exact change locations:

- `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `shell::workbench_host`: add explicit preset constructors and composition
  variants for full editor, standalone Material Lab, headless validation, and
  constrained hosts. The presets must select installed suites, profile specs,
  provider registry/provider-family assignments, and host policy through the
  existing composition path.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` tests:
  prove each preset exposes the expected composition id, suite ids, profile
  ids, provider-family mappings, and host policy. Tests should assert that the
  constrained preset denies unsupported capabilities through
  `HostCapabilityPolicy` rather than hiding routes through legacy identity.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` module
  `tool_suite::registry`: extend `WorkbenchCompositionBuilder` only if preset
  construction needs a small typed composition helper. Preserve existing
  validation for suites, profiles, provider assignments, capability
  declarations, and host policy.
- `domain/editor/editor_shell/src/tool_suite/mod.rs` and
  `domain/editor/editor_shell/src/lib.rs`: export only focused public types if
  new preset-facing composition contracts must be part of the normal API.

Explicit non-goals:

- Do not implement external component sandboxing, dynamic plugin loading,
  process isolation, package trust, permissions, unload/reload, or plugin
  runtime behavior.
- Do not move product truth, product graph formation, product validation, or
  service execution semantics into `editor_shell` or app host presets.
- Do not change provider UI behavior merely to make a constrained host pass.
  Unsupported work must be denied by host policy or omitted by explicit preset
  composition data.
- Do not restore legacy `ToolSurfaceKind` routing as host identity.
- Do not redesign the Workbench composition model beyond the narrow preset
  constructors and any minimal helper required to keep the presets typed.

## Acceptance Criteria

WR-039 implementation is complete only when all criteria below are true:

- Full editor, standalone Material Lab, headless validation, and constrained
  Workbench hosts are explicit presets built from the same composition model.
- Presets differ by suite/profile/provider bundle and `HostCapabilityPolicy`,
  not by forked app-specific compatibility paths.
- The default full-editor and standalone Material Lab behavior remains
  compatible with current Workbench host tests.
- Headless validation can construct a registry-backed host composition without
  requiring app runtime startup or external component infrastructure.
- Constrained hosts deny unsupported command/product/resource capabilities
  through host policy.
- Tests prove suite/profile/provider/policy composition for every preset.

## Implementation Steps

1. Rerun `task production:plan -- --milestone PM-WB-CAP-004 --roadmap WR-039`.
   Stop if WR-039 is no longer promotable or if another current candidate
   blocks promotion.
2. Promote WR-039 to `current_candidate`, validate roadmap and production
   state, and rerun `task ai:goal -- --track PT-WB-CAP --scope non-deferred`.
3. Inspect current `RunenwerkWorkbenchHost::from_composition`,
   `tool_suite_profile_definitions_for_composition`,
   `workspace_profile_registry_for_composition`, and
   `WorkbenchCompositionBuilder` before editing.
4. Add explicit preset contracts with the smallest durable API that keeps all
   host modes on the existing composition path.
5. Add focused host preset tests in `apps/runenwerk_editor/src/shell/workbench_host.rs`.
6. Run required validation and write closeout evidence under
   `docs-site/src/content/docs/reports/closeouts/wr-039-multi-host-presets/closeout.md`
   before changing roadmap or production completion evidence.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-039 implementation closeout:

```text
cargo fmt
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

## Stop Conditions

- Stop before product code if WR-039 is not `current_candidate` or the scoped
  goal command does not select implementation.
- Stop if ADR 0012 is not accepted or the active Workbench host design is not
  active.
- Stop if implementation requires write scopes outside
  `apps/runenwerk_editor/src/shell/workbench_host.rs` or
  `domain/editor/editor_shell/src/tool_suite`.
- Stop if headless or constrained hosts require external dynamic component
  sandboxing, plugin permissions, package trust, process isolation, unload,
  reload, or runtime plugin behavior.
- Stop if implementing a preset requires product semantic validation or service
  execution authority in `editor_shell`.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-039-multi-host-presets/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` rerun result;
- confirmation that no Rust product code, active design content, or production
  state changed for this contract-writing action.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-039 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.
