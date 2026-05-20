---
title: WR-037 Host Capability Policy Contract
description: Promotion and implementation-readiness contract for enforcing host command, product, and resource capability policy under PM-WB-CAP-002.
status: active
owner: domain/editor_editor_shell
layer: domain / app shell
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
related_adrs:
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
related_reports:
  - ../wr-036-material-lab-clean-migration-proof/plan.md
  - ../../closeouts/wr-036-material-lab-clean-migration-proof/closeout.md
---

# WR-037 Host Capability Policy Contract

## Goal

Establish promotion and implementation readiness for `WR-037` under
`PM-WB-CAP-002`. The slice must enforce host command, product, and resource
policy before provider proposals mutate app or domain state.

This contract does not implement product code or close `WR-037`. It records
the actual ownership boundary after `PM-WB-CAP-001`: capability key identity
types and `HostCapabilityPolicy` already exist, so the implementation must
wire those primitives into proposal dispatch instead of recreating them.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and `PM-WB-CAP-002`. The milestone requires provider proposals
  to pass through host policy before app or domain mutation.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-037`.
  The row is `ready_next`, blocker `B2`, depends on completed `WR-036`, and
  names `cargo test -p runenwerk_editor workbench_host` plus
  `cargo check -p runenwerk_editor` as required validation.
- `docs-site/src/content/docs/reports/closeouts/wr-036-material-lab-clean-migration-proof/closeout.md`
  records the completed clean Material Lab host proof that unblocks host
  policy.
- `docs-site/src/content/docs/adr/accepted/0012-capability-workbench-clean-break.md`
  is accepted and requires host capability policy to be the gate for command,
  product, and resource access.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is active and requires host-owned composition to remain the authority for
  installed suites, profiles, providers, and policy.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-002 next legal action as `repair_wr_promotion_metadata`.
- `task production:plan -- --milestone PM-WB-CAP-002 --roadmap WR-037`
  reported promotion preflight metadata blocked only because WR-037 was `B3`.
- WR-037 blocker metadata was repaired from `B3` to `B2`.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`, and
  `task production:check` passed after the repair.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` then classified
  PM-WB-CAP-002 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-002 --roadmap WR-037`
  reported WR-037 as promotable.
- `task roadmap:promote -- --id WR-037 --state current_candidate --evidence
  "<accepted evidence>"` promoted WR-037 to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`,
  `task production:check`, and `task docs:validate` passed after promotion.
- `cargo fmt`, `cargo test -p editor_shell requirements_`,
  `cargo test -p runenwerk_editor workbench_host`, and
  `cargo check -p runenwerk_editor` passed for the implementation.
- `docs-site/src/content/docs/reports/closeouts/wr-037-host-capability-policy/closeout.md`
  records the bounded implementation closeout.

Implementation outcome:

- `domain/editor/editor_shell/src/tool_suite/capability.rs` now provides
  `HostCapabilityRequirements`, a typed command/product/resource requirement
  set checked against `HostCapabilityPolicy`.
- `apps/runenwerk_editor/src/shell/controller.rs` now checks host policy in
  `RunenwerkEditorShellController::dispatch_commands_with_viewport_commands`
  before provider proposals are converted into shell commands.
- `apps/runenwerk_editor/src/shell/tests.rs` proves the default allow-all host
  preserves provider dispatch and a constrained host deny stops a provider
  surface-session proposal before mutation.

## Readiness

Promotion verdict: WR-037 is promotable after this contract is recorded.
Product code must not start until the roadmap row is promoted to
`current_candidate`, validation passes, and the scoped goal command selects
`execute_next_wr_implementation_contract`.

Existing code state:

- `domain/editor/editor_shell/src/tool_suite/identity.rs` already owns
  `CommandCapabilityKey`, `ProductCapabilityKey`, and `ResourceCapabilityKey`.
- `domain/editor/editor_shell/src/tool_suite/capability.rs` already owns
  `HostCapabilityPolicy` with allow-all, deny-all, explicit allow, and explicit
  deny semantics.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` already carries
  `HostCapabilityPolicy` through `WorkbenchCompositionBuilder`.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` already stores the host
  policy on `RunenwerkWorkbenchHost`; current built-in hosts use
  `HostCapabilityPolicy::allow_all()`.

Blocking conditions before code starts:

- WR-037 is promoted to `current_candidate`, but implementation cannot begin
  until the scoped goal rerun selects implementation.
- The implementation must not reintroduce legacy surface-kind identity or move
  product semantics into `editor_shell`.
- If implementation needs product/service declaration schema beyond command
  proposal policy checks, stop and hand that work to WR-038.
- If implementation needs broad host presets, headless validation modes, or
  constrained-host UX beyond a focused policy fixture/test, stop and hand that
  work to WR-039.
- If implementation needs external dynamic component permissions, package
  trust, sandboxing, unload/reload, or plugin runtime behavior, stop because
  WR-040 is blocked and out of the non-deferred scope.

## Implementation Scope

Owning modules and exact change locations:

- `domain/editor/editor_shell/src/tool_suite/capability.rs` module
  `tool_suite::capability` owns `HostCapabilityPolicy`. Extend only if the
  policy primitive needs typed query helpers for proposals; preserve explicit
  allow/deny semantics.
- `domain/editor/editor_shell/src/tool_suite/identity.rs` module
  `tool_suite::identity` owns command, product, and resource capability key
  validation. Do not recreate key types elsewhere.
- `domain/editor/editor_shell/src/surface_provider.rs` module
  `surface_provider` owns `SurfaceCommandProposal`,
  `SurfaceSessionMutationProposal`, and `EditorDomainProposal`. Add capability
  requirements here or in a clearly named adjacent contract so provider
  proposals carry the policy keys needed before app/domain mutation.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` module
  `shell::workbench_host` owns `RunenwerkWorkbenchHost::host_capability_policy`
  and host construction. Add focused allow/deny host fixtures and tests here
  if they can stay within the existing host composition boundary.
- `apps/runenwerk_editor/src/shell/controller.rs` module `shell::controller`
  owns provider-local action and interaction proposal resolution in
  `RunenwerkEditorShellController::dispatch_commands_with_viewport_commands`.
  This is the preferred app boundary for checking host policy before converted
  provider proposals become shell commands.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` module
  `shell::dispatch_shell_command` owns app/domain mutation execution. Add a
  defensive policy check here only if proposal-time enforcement cannot cover
  every mutation path.
- `apps/runenwerk_editor/src/shell/providers` owns provider `map_action`
  implementations. Providers may declare capability keys on proposals, but
  they must not decide host policy.

Explicit non-goals:

- Do not implement product/service capability declaration workflows; WR-038
  owns that.
- Do not implement broad multi-host presets; WR-039 owns that.
- Do not implement external component sandboxing or dynamic plugins; WR-040 is
  blocked.
- Do not move Material Lab, asset, viewport, or SDF product semantics into
  `editor_shell` to satisfy policy checks.
- Do not rely on legacy `ToolSurfaceKind` routing as a policy authority.

## Acceptance Criteria

The future WR-037 implementation is complete only when all criteria below are
true:

- Provider proposals that mutate app state, domain state, shell state, product
  access, or resource access carry typed capability requirements.
- `RunenwerkWorkbenchHost::host_capability_policy()` is checked before such a
  proposal is converted into a `ShellCommand` or dispatched to app/domain
  mutation.
- A constrained host denies at least one command capability before mutation,
  with a clear `EditorMutationError` or diagnostic consistent with existing
  shell dispatch errors.
- The default full-editor and standalone Material Lab hosts preserve their
  current behavior through allow-all policy.
- Tests prove deny wins over allow, allow-all preserves existing dispatch, and
  denied provider proposals do not mutate app/domain state.
- The implementation does not add product/service declaration semantics that
  belong to WR-038.

## Implementation Steps

1. Rerun `task production:plan -- --milestone PM-WB-CAP-002 --roadmap WR-037`.
   Stop if WR-037 is no longer promotable or if another current candidate
   blocks promotion.
2. Promote WR-037 to `current_candidate`, validate roadmap and production
   state, and rerun `task ai:goal -- --track PT-WB-CAP --scope non-deferred`.
3. Inspect current proposal dispatch in
   `apps/runenwerk_editor/src/shell/controller.rs` and mutation execution in
   `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`.
4. Add typed capability requirements to provider proposals in
   `domain/editor/editor_shell/src/surface_provider.rs` or a focused adjacent
   module with explicit command/product/resource ownership.
5. Gate proposal conversion through `RunenwerkWorkbenchHost::host_capability_policy()`
   before app/domain mutation.
6. Add focused tests under `apps/runenwerk_editor/src/shell/workbench_host.rs`
   or `apps/runenwerk_editor/src/shell/tests.rs` proving denied command
   proposals do not mutate state and default hosts still allow existing flows.
7. Run required validation and write closeout evidence under
   `docs-site/src/content/docs/reports/closeouts/wr-037-host-capability-policy/closeout.md`
   before changing roadmap or production completion evidence.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-037 implementation closeout:

```text
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

- Stop before product code if WR-037 is not `current_candidate` or the scoped
  goal command does not select `execute_next_wr_implementation_contract`.
- Stop if ADR 0012 is not accepted or the active Workbench host design is not
  active.
- Stop if implementation requires write scopes outside
  `domain/editor/editor_shell/src/tool_suite` or
  `apps/runenwerk_editor/src/shell`.
- Stop if enforcing policy requires product/service declarations that belong
  to WR-038.
- Stop if host-policy behavior starts implementing multi-host presets that
  belong to WR-039.
- Stop if external component permissions, sandboxing, package trust, unload,
  reload, or plugin runtime behavior becomes necessary.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-037-host-capability-policy/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` rerun result;
- confirmation that no Rust product code or production state changed;
- remaining blocker before product code: WR-037 must be selected by the next
  scoped goal rerun as the current implementation contract.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-037 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.
