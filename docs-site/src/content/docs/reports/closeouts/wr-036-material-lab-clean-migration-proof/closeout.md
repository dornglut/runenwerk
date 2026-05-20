---
title: WR-036 Material Lab Clean Migration Proof Closeout
description: Completed bounded implementation closeout for Material Lab full-editor and standalone Workbench host resolution without legacy metadata.
status: completed
owner: apps/runenwerk_editor
layer: app / shell
canonical: true
last_reviewed: 2026-05-20
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md
  - ../../closeouts/wr-035-clean-persistence-format/closeout.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
---

# WR-036 Material Lab Clean Migration Proof Closeout

## Result

WR-036 is complete as the bounded Material Lab proof slice for
`PM-WB-CAP-001`. Full-editor and standalone Material Lab app construction now
share a host-level test path that mounts the Material workspace profile from
the active `WorkspaceProfileRegistry` and `ToolSurfaceRegistry`, then resolves
graph, inspector, preview, texture, asset, diagnostics, and console surfaces
through the active `ProviderFamilyProviderMap`.

The proof asserts every mounted Material profile request carries hosted
provider-family and route metadata, resolves to the expected concrete provider,
and comes from stable-key persisted `ToolSurfaceDefinition` data. It does not
reopen WR-034 profile construction or WR-035 persistence cleanup.

This completes the last WR implementation slice linked by `PM-WB-CAP-001`.
Downstream PT-WB-CAP milestones remain dependency-gated until the production
goal command selects them.

## Changed Files

- `apps/runenwerk_editor/src/shell/workbench_host.rs`:
  `material_lab_full_editor_and_standalone_resolve_mounted_material_workspace_surfaces`
  now proves the Material profile surface chain in both host compositions,
  including provider-family selection, route metadata, stable-key persistence,
  and concrete provider resolution.
- `docs-site/src/content/docs/reports/implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md`:
  refreshed promotion and implementation-readiness evidence after WR-035
  completion and WR-036 promotion.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`: promoted WR-036
  to `current_candidate` and removed stale WR-035 dependency wording.

## Validation

Passed:

```text
cargo fmt
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

## Completion Quality

Completion quality: `bounded_contract`.

The slice proves the clean Material Lab host migration required by
`PM-WB-CAP-001`, but it is not `perfectionist_verified` for the full production
track because later non-deferred milestones still own broader capability
platform behavior:

- WR-037 still owns host command, product, and resource capability policy.
- WR-038 still owns product and service capability declarations.
- WR-039 still owns broader multi-host preset behavior.
- WR-040 remains blocked and out of the non-deferred scope for external
  component sandbox readiness.

## Closeout Decision

Archive WR-036 as completed with this closeout evidence. Rerun roadmap and
production gates, then rerun `task ai:goal -- --track PT-WB-CAP --scope
non-deferred` before starting PM-WB-CAP-002 or any downstream milestone work.
