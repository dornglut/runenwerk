---
title: WR-107 UI Lab Command And Surface Source Truth Closure Contract
description: Current-candidate implementation contract for PM-UI-LAB-PERF-003 command catalog, surface registry, ownership, and module-structure source-of-truth closure.
status: active
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-command-surface-source-truth-closure-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
  - ../../../design/accepted/ui-lab-command-catalog-and-surface-registry-design.md
related_reports:
  - ../../closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md
  - ../../roadmap-intake/2026-05-25-pm-ui-lab-perf-003-command-and-surface-s/proposal.yaml
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-107 UI Lab Command And Surface Source Truth Closure Contract

## Goal

Implement `PM-UI-LAB-PERF-003` by closing command catalog, keybinding,
toolbar/menu, disabled-reason, surface registry, legacy enum, and
module-structure source-of-truth drift.

WR-107 is now a `current_candidate` row. This contract is the bounded
implementation contract, but this specific workflow action is contract-only:
no product code may be changed until `task ai:goal -- --track
PT-UI-LAB-PERFECTION` selects the next implementation action after this
contract validates.

## Source Of Truth

- Production milestone: `PM-UI-LAB-PERF-003` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-107` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM003 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-command-surface-source-truth-closure-design.md`.
- Accepted no-gap doctrine:
  `docs-site/src/content/docs/design/accepted/ui-lab-perfectionist-audit-design.md`.
- Completed PM002 runtime evidence input:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md`.
- Earlier accepted command/surface design:
  `docs-site/src/content/docs/design/accepted/ui-lab-command-catalog-and-surface-registry-design.md`.

Current implementation sources to inspect before code changes:

- `apps/runenwerk_editor/src/shell/command_catalog/mod.rs`
- `apps/runenwerk_editor/src/shell/command_resolution.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/toolbar_adapter.rs`
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs`
- `apps/runenwerk_editor/src/shell/providers/mod.rs`
- `domain/editor/editor_shell/src/tool_suite/definition.rs`
- `domain/editor/editor_shell/src/tool_suite/registry.rs`
- `domain/editor/editor_shell/src/workspace/surface_contract.rs`
- `assets/editor/ui/editor_bindings.ron`
- `apps/runenwerk_editor/src/shell/tests.rs`

## Current-Candidate Readiness

`task production:plan -- --milestone PM-UI-LAB-PERF-003 --roadmap WR-107`
reported the promotion preflight as promotable before WR-107 was promoted:

- action: `write_promotion_contract`;
- promotion preflight: `promotable`;
- dependency state: `WR-105:completed`.

The current implementation action is honest because:

- PM002 is completed with runtime-proven typed evidence closure.
- PM003 has an accepted command and surface source-truth closure design.
- WR-107 has bounded command/surface write scopes and explicit non-goals.
- WR-107 was promoted to `current_candidate` with the evidence below, so this
  contract may plan only PM003 command/surface source-truth implementation.

Recorded promotion evidence:

```text
Accepted PM-UI-LAB-PERF-003 command and surface source-truth closure design plus completed WR-105 runtime evidence platform closeout clear WR-107 for current-candidate implementation planning; command execution remains app-owned, surface metadata remains editor_shell-owned, ui_definition remains behavior-free, and legacy command/surface enums must become compatibility adapters only.
```

## Architecture Decisions

Source-truth decisions:

- `apps/runenwerk_editor` owns concrete editor command descriptors, dynamic app
  availability, app diagnostics, shell-command dispatch targets, shortcut/menu
  projection inputs, and provider execution.
- `domain/editor/editor_shell` owns app-neutral tool-suite and surface metadata:
  stable surface keys, roles, provider family, capabilities, route kind,
  persistence mode, retention, creation policy, and legacy mapping contracts.
- `domain/editor/editor_definition` may declare editor command, menu,
  shortcut, binding, and surface references, but it must not execute app
  commands or provider behavior.
- `domain/ui/ui_definition` remains behavior-free and may describe authored
  UI structure or route slots only.
- Legacy `KnownEditorCommand`, `ToolbarCommandKind`, `ToolbarMenuKind`,
  `ToolSurfaceKind`, and `PanelKind` are compatibility edges, not normal
  source truth after the implementation closes.

Forbidden shortcuts:

- moving editor command semantics, shortcuts, provider family authority, or
  surface execution into `ui_definition`;
- treating labels, disabled reasons, route actions, toolbar/menu projection,
  keybindings, or dispatch as independently owned fallback tables;
- treating `ToolSurfaceKind` or `PanelKind` switches as the normal metadata
  source for new surface behavior;
- starting PM004 UX, PM005 API/persistence, or PM006 final certification work.

## Implementation Scope

Expected implementation files:

- `apps/runenwerk_editor/src/shell/command_catalog/`
- `apps/runenwerk_editor/src/shell/command_resolution.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/toolbar_adapter.rs`
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs`
- `apps/runenwerk_editor/src/shell/providers/`
- `domain/editor/editor_shell/src/tool_suite/`
- `domain/editor/editor_shell/src/workspace/surface_contract.rs`
- `assets/editor/ui/editor_bindings.ron`
- `apps/runenwerk_editor/src/shell/tests.rs`

Expected evidence and docs:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md`
- Roadmap and production metadata/rendered docs after closeout.

Expected contract changes after promotion:

- Add or extend catalog audits for every normal authored command route.
- Prove toolbar, menu, keybinding, route action, disabled reason, and dispatch
  projections use one catalog descriptor.
- Add or extend surface registry audits for every normal provider support path.
- Prove legacy surface enum mappings are explicit adapters with typed
  diagnostics.
- Preserve module structure under command, provider, dispatch, tool-suite, and
  workspace subdomains.

## Acceptance Criteria

- Every normal authored command route resolves to exactly one catalog command.
- Toolbar, menu, keybinding, palette/search, route action, disabled reason, and
  dispatch projections use the same descriptor data.
- Shell fallback tables are not required for normal Editor Lab command routes.
- Every normal surface provider support path is declared by registry metadata.
- Legacy command and surface enums remain only at compatibility, persistence,
  import, test, or migration edges.
- `ui_definition` remains behavior-free and does not gain concrete editor
  command or provider behavior.

## Validation

Implementation validation:

```text
cargo fmt
cargo test -p runenwerk_editor command_catalog
cargo test -p runenwerk_editor command_source_truth
cargo test -p runenwerk_editor surface_source_truth
cargo test -p editor_shell tool_suite
cargo test -p editor_shell surface_contract
task docs:validate
task puml:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
git diff --check
task ai:goal -- --track PT-UI-LAB-PERFECTION
```

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md`
only after focused tests and metadata gates pass.

The closeout must include:

- reproduction commands;
- command catalog and surface registry audit results;
- exact legacy adapter boundaries that remain;
- proof that `ui_definition` remains behavior-free;
- validation output;
- remaining known quality gaps that belong to PM004 through PM006;
- roadmap archive and production milestone updates only after validation.

## Perfectionist Closeout Audit

WR-107 may close PM003 at `runtime_proven` only if tests prove normal command
and surface behavior derive from one owner per boundary. It must not claim
`perfectionist_verified`.

Remaining gaps after PM003 are expected:

- direct-manipulation UX closure remains PM004;
- persistence/API/examples ergonomics closure remains PM005;
- final no-gap certification remains PM006.

## Stop Conditions

Stop implementation if:

- command or surface authority still needs two normal sources of truth;
- closure requires breaking persisted legacy surface data without an accepted
  migration plan;
- implementation requires moving editor command semantics or provider execution
  into `ui_definition`;
- implementation requires a reusable cross-domain command/surface authority
  without an ADR or accepted design update;
- the row starts PM004, PM005, or PM006 scope.
