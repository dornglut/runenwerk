---
title: WR-037 Host Capability Policy Closeout
description: Completed bounded implementation closeout for host capability policy enforcement before provider proposal mutation.
status: completed
owner: domain/editor_editor_shell
layer: domain / app shell
canonical: true
last_reviewed: 2026-05-20
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-037-host-capability-policy/plan.md
  - ../../closeouts/wr-036-material-lab-clean-migration-proof/closeout.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
---

# WR-037 Host Capability Policy Closeout

## Result

WR-037 is complete as the bounded host policy implementation slice for
`PM-WB-CAP-002`. Provider-local action and interaction proposals now pass
through `RunenwerkWorkbenchHost::host_capability_policy()` before they are
converted into `ShellCommand` values and before app, shell, or domain mutation
can run.

The implementation adds a reusable typed requirement set for command, product,
and resource capability keys. The current provider proposal boundary classifies
surface-session, editor-domain, and shell-command proposals as command
requirements; product and resource declaration workflows remain explicitly
deferred to WR-038, but the policy check path already evaluates those planes
when requirements are present.

## Changed Files

- `domain/editor/editor_shell/src/tool_suite/capability.rs`:
  `HostCapabilityRequirements` and `DeniedHostCapabilityRequirement` now model
  typed command/product/resource requirements and check them against
  `HostCapabilityPolicy`.
- `domain/editor/editor_shell/src/tool_suite/mod.rs` and
  `domain/editor/editor_shell/src/lib.rs`: export
  `HostCapabilityRequirements` with the existing Workbench capability policy
  contracts.
- `apps/runenwerk_editor/src/shell/controller.rs`:
  `RunenwerkEditorShellController::dispatch_commands_with_viewport_commands`
  enforces host policy on provider proposals before proposal conversion and
  dispatch.
- `apps/runenwerk_editor/src/shell/workbench_host.rs`:
  adds a test-only policy fixture hook on `RunenwerkWorkbenchHost` for
  constrained host assertions.
- `apps/runenwerk_editor/src/shell/tests.rs`:
  adds allow-all and explicit-deny provider proposal tests proving the default
  host preserves existing dispatch and a constrained host rejects before
  session mutation.

## Validation

Passed:

```text
cargo fmt
cargo test -p editor_shell requirements_
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
```

## Completion Quality

Completion quality: `bounded_contract`.

The slice completes the host policy gate required before provider proposals can
mutate app/domain state, but it is not `perfectionist_verified` for the full
production track because downstream milestones still own broader platform
behavior:

- WR-038 still owns product and service capability declaration workflows.
- WR-039 still owns broad full-editor, standalone Material Lab, headless
  validation, and constrained host presets.
- WR-040 remains blocked and out of the non-deferred scope for external
  component sandbox readiness.

## Closeout Decision

Archive WR-037 as completed with this closeout evidence. Rerun roadmap and
production gates, then rerun `task ai:goal -- --track PT-WB-CAP --scope
non-deferred` before marking `PM-WB-CAP-002` complete or starting any
downstream milestone work.
