---
title: WR-035 Clean Persistence Format Closeout
description: Completed bounded implementation closeout for stable-key-only Workbench workspace persistence.
status: completed
owner: domain/editor/editor_shell
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-035-clean-persistence-format/plan.md
  - ../../implementation-plans/wr-034-registry-backed-workspace-profiles/plan.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
---

# WR-035 Clean Persistence Format Closeout

## Result

WR-035 is complete as a bounded persistence cleanup slice for
`PM-WB-CAP-001`. App workspace layout reads now reject V1-V4 workspace files as
unsupported, and V5 layout reads reject legacy fallback metadata such as
`legacy_tool_surface_kind` and `legacy_locked_tool_surface_kind`. New V5 saves
write stable surface keys without legacy fallback fields.

This does not complete `PM-WB-CAP-001`. WR-036 still owns the final Material
Lab clean migration proof across full-editor and standalone hosts.

## Changed Files

- `domain/editor/editor_shell/src/workspace/persisted.rs`: V5 decode rejects
  legacy fallback metadata for tool surfaces and tab-stack locks, while V5
  serialization remains stable-key-only.
- `apps/runenwerk_editor/src/persistence/workspace_layout.rs`: production app
  workspace layout reads reject V1-V4 versions with unsupported-version
  diagnostics and keep V5 round trips registry-backed.
- `docs-site/src/content/docs/reports/implementation-plans/wr-035-clean-persistence-format/plan.md`:
  refreshed promotion readiness after WR-034 completion.

## Validation

Passed:

```text
cargo fmt
cargo test -p editor_shell workspace::profile
cargo test -p editor_shell workspace::persisted
cargo test -p runenwerk_editor workbench_host
cargo test -p runenwerk_editor workspace_layout
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

The slice enforces the clean persistence break at the app workspace layout
boundary and rejects V5 legacy fallback metadata. It is not
`perfectionist_verified` because one downstream proof remains:

- WR-036 still owns final Material Lab clean migration evidence across
  full-editor and standalone hosts.
- Host capability policy, product/service declarations, multi-host presets,
  and external component readiness remain outside WR-035.

## Closeout Decision

Archive WR-035 as completed with this closeout evidence. Continue
`PM-WB-CAP-001` with WR-036 only after roadmap and production gates pass and
the scoped production goal command is rerun.
