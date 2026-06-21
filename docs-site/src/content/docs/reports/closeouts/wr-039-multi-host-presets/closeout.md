---
title: WR-039 Multi-Host Presets Closeout
description: Completed bounded implementation closeout for explicit full-editor, Material Lab, headless validation, and constrained Workbench host presets.
status: completed
owner: apps/runenwerk_editor
layer: app shell / domain composition
canonical: true
last_reviewed: 2026-05-21
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-039-multi-host-presets/plan.md
  - ../../closeouts/wr-037-host-capability-policy/closeout.md
  - ../../closeouts/wr-038-product-and-service-capability-declarations/closeout.md
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
---

# WR-039 Multi-Host Presets Closeout

## Result

WR-039 is complete as the bounded multi-host preset implementation slice for
`PM-WB-CAP-004`. The full editor, standalone Material Lab, headless validation,
and constrained host modes are now explicit Workbench composition presets built
through the same host construction path.

The implementation keeps host-mode differences as data: installed suites,
profile specs, provider-family assignments, provider registries, and
`HostCapabilityPolicy`. It does not restore legacy surface-kind routing, move
product semantics into `editor_shell`, or implement external dynamic component
behavior.

## Changed Files

- `apps/runenwerk_editor/src/shell/workbench_host.rs`:
  `RunenwerkWorkbenchComposition` now includes `HeadlessValidation` and
  `Constrained`, with `RunenwerkWorkbenchHost::headless_validation()` and
  `RunenwerkWorkbenchHost::constrained()` constructors beside the existing
  full-editor and Material Lab presets.
- `apps/runenwerk_editor/src/shell/workbench_host.rs`:
  headless validation installs editor, asset, and diagnostics suites with the
  runtime-debug profile and deny-all host policy; constrained hosts install the
  full-editor suite/profile/provider set with deny-all host policy.
- `apps/runenwerk_editor/src/shell/workbench_host.rs`:
  `explicit_workbench_presets_are_composition_data` proves each preset exposes
  the expected composition id, suite/profile/provider-family shape, and host
  policy behavior.
- `docs-site/src/content/docs/reports/implementation-plans/wr-039-multi-host-presets/plan.md`:
  records the implementation outcome and validation evidence.

## Validation

Passed:

```text
cargo fmt
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
```

Repository gates to run with archive/render closeout:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task docs:validate
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

The slice completes the non-deferred multi-host preset milestone, but it is not
`perfectionist_verified` for future external component work:

- WR-040 remains blocked and out of the non-deferred scope for external
  component sandbox readiness.
- Headless validation is a registry-backed host composition proof, not a
  general process runner or plugin sandbox.
- Constrained behavior is enforced through `HostCapabilityPolicy`; richer
  capability UX and diagnostics remain future bounded work.

## Closeout Decision

Archive WR-039 as completed with this closeout evidence. Rerun roadmap,
production, docs, and planning gates, then rerun
`task ai:goal -- --track PT-WB-CAP --scope non-deferred` before marking
`PM-WB-CAP-004` complete or claiming the non-deferred production-track scope.
