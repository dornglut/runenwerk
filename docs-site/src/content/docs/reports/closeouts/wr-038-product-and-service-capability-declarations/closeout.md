---
title: WR-038 Product And Service Capability Declarations Closeout
description: Completed bounded implementation closeout for declarative product and service capability needs in Workbench tool-suite contracts.
status: completed
owner: domain/editor_editor_shell
layer: domain / workspace design
canonical: true
last_reviewed: 2026-05-21
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-038-product-and-service-capability-declarations/plan.md
  - ../../closeouts/wr-037-host-capability-policy/closeout.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
---

# WR-038 Product And Service Capability Declarations Closeout

## Result

WR-038 is complete as the bounded product and service capability declaration
slice for `PM-WB-CAP-003`. Tool-suite composition can now carry suite-scoped
declarations for requested product capabilities and requested tool services
without turning those declarations into permission grants, product semantics,
or service execution authority.

The implementation keeps the authority split explicit: `editor_shell` owns the
typed declaration shape and registry validation, host policy remains the
permission authority, and owning domains still validate product meaning and
formed product contracts.

## Changed Files

- `domain/editor/editor_shell/src/tool_suite/identity.rs`:
  `ToolServiceKey` adds stable lowercase dotted identity for tool-service
  dependencies and shares the existing invalid-key guard coverage.
- `domain/editor/editor_shell/src/tool_suite/definition.rs`:
  `ToolSuiteCapabilityDeclaration`, `ProductCapabilityNeed`, and
  `ToolServiceNeed` model declaration-only product and service needs by suite.
- `domain/editor/editor_shell/src/tool_suite/registry.rs`:
  `ToolSuiteRegistry::new_with_capability_declarations`,
  `ToolSuiteRegistry::capability_declarations`,
  `ToolSuiteRegistry::capability_declaration`, and
  `WorkbenchCompositionBuilder::with_capability_declarations` register and
  validate declaration sidecars while rejecting unknown suites, duplicate
  suite declarations, duplicate product/service needs, and empty labels.
- `domain/editor/editor_shell/src/tool_suite/mod.rs` and
  `domain/editor/editor_shell/src/lib.rs`: export the focused public
  declaration types needed for normal Workbench composition authoring.
- `docs-site/src/content/docs/design/active/runenwerk-capability-workbench-target-architecture.md`:
  documents that product and service declarations are requests, not authority,
  and that host policy and owning domains remain separate gates.

## Validation

Passed:

```text
cargo fmt
cargo test -p editor_shell tool_suite
cargo check -p editor_shell
task docs:validate
```

Repository gates to run with archive/render closeout:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

The slice completes the product/service declaration plane required by
`PM-WB-CAP-003`, but it is not `perfectionist_verified` for the full production
track because downstream and blocked milestones still own broader behavior:

- WR-039 still owns broad full-editor, standalone Material Lab, headless
  validation, and constrained host presets.
- WR-040 remains blocked and out of the non-deferred scope for external
  component sandbox readiness.
- Product semantics and service execution remain outside `editor_shell` by
  design; this slice only declares requested needs.

## Closeout Decision

Archive WR-038 as completed with this closeout evidence. Rerun roadmap,
production, docs, and planning gates, then rerun
`task ai:goal -- --track PT-WB-CAP --scope non-deferred` before marking
`PM-WB-CAP-003` complete or starting any downstream milestone work.
