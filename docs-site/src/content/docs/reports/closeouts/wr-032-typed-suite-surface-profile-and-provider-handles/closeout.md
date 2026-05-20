---
title: WR-032 Typed Suite, Surface, Profile, And Provider Handles Closeout
description: Completed bounded implementation closeout for typed Workbench suite, surface, profile, and provider composition handles.
status: completed
owner: editor
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
---

# WR-032 Typed Suite, Surface, Profile, And Provider Handles Closeout

## Result

WR-032 is complete as a bounded platform-foundation implementation slice for
`PM-WB-CAP-001`. The implementation adds typed constructor paths for Workbench
suite, provider-family, surface, and profile declarations, validates profile
definitions in `WorkbenchCompositionBuilder`, and makes the full-editor and
standalone Material Lab hosts expose validated typed profile data through the
same composition builder.

This does not complete `PM-WB-CAP-001`. The milestone still depends on WR-033,
WR-034, WR-035, and WR-036 before Workbench identity, profile construction,
persistence, and Material Lab routing can be claimed stable-key-only.

## Changed Files

- `domain/editor/editor_shell/src/tool_suite/definition.rs`: added typed
  constructors for `EditorToolSuite`, `ProviderFamilyDefinition`, and
  `ToolSurfaceDefinition`.
- `domain/editor/editor_shell/src/tool_suite/registry.rs`: added
  `WorkbenchCompositionBuilder` validation for duplicate profile refs and
  unknown profile default surfaces, plus focused guard tests.
- `apps/runenwerk_editor/src/shell/workbench_host.rs`: stores typed profile
  definitions on `RunenwerkWorkbenchHost`, feeds them through
  `WorkbenchCompositionBuilder`, and tests full-editor and Material Lab profile
  exposure.
- `apps/runenwerk_editor/src/shell/tool_suites/mod.rs`: routes compiled-in
  shell suite and stable surface declarations through the typed constructors.
- `apps/runenwerk_editor/src/material_lab/tool_suite.rs`: routes the Material
  Lab suite declaration through the typed constructors.

## Validation

Passed:

```text
cargo test -p editor_shell tool_suite
cargo test -p editor_shell workspace::profile
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
```

Additional roadmap and production validation is required after this closeout is
recorded in roadmap metadata:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

## Completion Quality

Completion quality: `bounded_contract`.

The slice is runtime-integrated enough to validate builder consumption and app
host profile exposure, but it is not `perfectionist_verified` because known
downstream gaps remain by design:

- WR-033 still owns removal of legacy `ToolSurfaceKind` identity from
  Workbench state, provider requests, frame artifacts, commands, and source
  guards.
- WR-034 still owns registry-backed workspace profile authority.
- WR-035 still owns stable-key-only persistence and unsupported-schema
  diagnostics for old workspace files.
- WR-036 still owns the final Material Lab clean migration proof across
  full-editor and standalone hosts.
- Host capability policy, product/service declarations, multi-host presets,
  and external component readiness remain outside WR-032.

## Closeout Decision

Archive WR-032 as completed with this closeout evidence. Continue
`PM-WB-CAP-001` with WR-033 in dependency order after rerunning the scoped
production goal command.
