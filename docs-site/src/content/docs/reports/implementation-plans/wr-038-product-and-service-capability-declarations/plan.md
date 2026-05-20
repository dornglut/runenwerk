---
title: WR-038 Product And Service Capability Declarations Contract
description: Promotion and implementation-readiness contract for declarative product and service capability needs under PM-WB-CAP-003.
status: active
owner: domain/editor_editor_shell
layer: domain / workspace design
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
related_adrs:
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
related_reports:
  - ../wr-037-host-capability-policy/plan.md
  - ../../closeouts/wr-037-host-capability-policy/closeout.md
---

# WR-038 Product And Service Capability Declarations Contract

## Goal

Establish promotion and implementation readiness for `WR-038` under
`PM-WB-CAP-003`. The slice must add declarative product and service capability
needs to Workbench tool-suite contracts without moving product semantics,
service execution authority, or domain validation into `editor_shell`.

This contract does not implement product code or close `WR-038`. It records the
bounded design and domain-contract work needed after `WR-037`: host policy is
now enforced before provider proposals mutate state, so the next step is
declaration shape and validation for product/service needs.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and `PM-WB-CAP-003`. The milestone requires product and service
  needs to be declarative Workbench capability data while product semantics stay
  in owning domains and formed product contracts.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-038`.
  The row entered implementation as `current_candidate`, blocker `B2`, depends
  on completed `WR-037`, and names `cargo test -p editor_shell tool_suite`
  plus `task docs:validate` as required validation.
- `docs-site/src/content/docs/reports/closeouts/wr-037-host-capability-policy/closeout.md`
  records the completed provider proposal host policy gate that unblocks
  product/service declarations.
- `docs-site/src/content/docs/design/active/runenwerk-capability-workbench-target-architecture.md`
  is active and defines the long-term rule: tools request, hosts allow, and
  domains validate.
- `docs-site/src/content/docs/adr/accepted/0012-capability-workbench-clean-break.md`
  is accepted and forbids restoring legacy Workbench identity.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-003 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-003 --roadmap WR-038`
  reported WR-038 as promotable and classified the next action as
  `write_promotion_contract`.
- `task roadmap:promote -- --id WR-038 --state current_candidate --evidence
  "<accepted evidence>"` promoted WR-038 to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, `task roadmap:check`,
  `task production:render`, `task production:validate`,
  `task production:check`, `task docs:validate`, and
  `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` then classified
  PM-WB-CAP-003 next legal action as `execute_next_wr_implementation_contract`.
- `task production:plan -- --milestone PM-WB-CAP-003 --roadmap WR-038`
  reported WR-038 as the active implementation contract.
- `cargo fmt`, `cargo test -p editor_shell tool_suite`,
  `cargo check -p editor_shell`, and `task docs:validate` passed for the
  implementation.
- `docs-site/src/content/docs/reports/closeouts/wr-038-product-and-service-capability-declarations/closeout.md`
  records the bounded implementation closeout.

Implementation outcome:

- `domain/editor/editor_shell/src/tool_suite/definition.rs` now provides
  `ToolSuiteCapabilityDeclaration`, `ProductCapabilityNeed`, and
  `ToolServiceNeed` as declarative sidecar contracts keyed by suite.
- `domain/editor/editor_shell/src/tool_suite/identity.rs` now owns
  `ToolServiceKey` with the same stable lowercase dotted identity validation
  used by other Workbench keys.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` now accepts and
  validates suite-scoped capability declarations, rejects unknown suites,
  duplicate declarations, duplicate product/service needs, and empty labels,
  and exposes lookup methods by suite.
- `docs-site/src/content/docs/design/active/runenwerk-capability-workbench-target-architecture.md`
  now records that product and service declarations are requests, not
  permission grants or semantic authority.

## Readiness

Promotion verdict: WR-038 is promotable after this contract is recorded and the
roadmap/production/doc gates pass. Product code must not start until the
roadmap row is promoted to `current_candidate`, validation passes, and the
scoped goal command selects implementation.

Existing code state:

- `domain/editor/editor_shell/src/tool_suite/identity.rs` already owns
  `ProductCapabilityKey` and `ResourceCapabilityKey`; implementation must reuse
  those keys instead of introducing stringly typed product/resource names.
- `domain/editor/editor_shell/src/tool_suite/capability.rs` already provides
  `HostCapabilityPolicy` and `HostCapabilityRequirements`, including product
  and resource requirement checks.
- `domain/editor/editor_shell/src/tool_suite/definition.rs` owns declarative
  tool-suite and surface contracts. This is the expected owner for product and
  service need declarations.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` owns suite and
  composition validation. This is the expected owner for rejecting duplicate,
  invalid, or semantically misplaced declaration data.

Blocking conditions before code starts:

- WR-038 must be promoted to `current_candidate`.
- If implementation needs app shell enforcement, provider proposal routing, or
  mutation dispatch changes, stop. WR-037 owns the dispatch gate and any new
  enforcement boundary must be deliberately scoped.
- If implementation needs domain product semantics, product descriptors, formed
  product graph nodes, service processes, IO permissions, or runtime execution,
  stop. Those are outside `editor_shell` declaration ownership.
- If implementation needs multi-host presets or headless/constrained host
  constructors, stop and hand that work to WR-039.
- If implementation needs external component sandboxing, package trust,
  permissions, unload/reload, or plugin runtime behavior, stop because WR-040
  is blocked and out of the non-deferred scope.

## Implementation Scope

Owning modules and exact change locations:

- `domain/editor/editor_shell/src/tool_suite/definition.rs` module
  `tool_suite::definition`: add declarative product and service requirement
  fields or adjacent value types on `EditorToolSuite`, `ToolSurfaceDefinition`,
  or a clearly named capability declaration contract. The declaration must
  contain identity and intent only; no product semantics or runtime service
  handles.
- `domain/editor/editor_shell/src/tool_suite/capability.rs` module
  `tool_suite::capability`: extend requirement helpers only if declaration
  validation needs a typed query surface over product/resource/service needs.
  Preserve existing allow/deny semantics.
- `domain/editor/editor_shell/src/tool_suite/identity.rs` module
  `tool_suite::identity`: add a stable service identity type only if the
  declaration cannot honestly reuse existing product/resource keys. The type
  must follow existing lowercase dotted identity validation.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` module
  `tool_suite::registry`: validate declaration references during
  `ToolSuiteRegistry` or `WorkbenchCompositionBuilder` construction.
- `domain/editor/editor_shell/src/tool_suite/mod.rs` and
  `domain/editor/editor_shell/src/lib.rs`: export only the focused public types
  required for normal declaration authoring.
- `docs-site/src/content/docs/design/active/runenwerk-capability-workbench-target-architecture.md`:
  update the active design with the exact product/service declaration boundary,
  examples, non-goals, and domain-owned semantic validation rule.

Explicit non-goals:

- Do not implement app provider changes, shell dispatch changes, host preset
  constructors, headless hosts, or external components.
- Do not move product truth, product validation, product graph formation, or
  service execution semantics into `editor_shell`.
- Do not use declarations as permission grants. They are requested needs; host
  policy and owning domains remain separate authorities.
- Do not replace existing command policy keys or host policy enforcement from
  WR-037.

## Acceptance Criteria

WR-038 implementation is complete only when all criteria below are true:

- Tool-suite contracts can declare product and service needs with typed stable
  identities.
- Registry/composition validation rejects invalid declaration data and preserves
  domain ownership boundaries.
- The active capability Workbench design explains product/service declarations,
  host policy, and domain validation as separate authorities.
- Tests under `domain/editor/editor_shell/src/tool_suite` prove normal
  declarations are discoverable, invalid declarations are rejected, and command
  policy behavior from WR-037 remains compatible.
- No implementation adds product semantics, service process orchestration, app
  host presets, or external component runtime behavior.

## Implementation Steps

1. Rerun `task production:plan -- --milestone PM-WB-CAP-003 --roadmap WR-038`.
   Stop if WR-038 is no longer promotable or if another current candidate
   blocks promotion.
2. Promote WR-038 to `current_candidate`, validate roadmap and production
   state, and rerun `task ai:goal -- --track PT-WB-CAP --scope non-deferred`.
3. Inspect current `tool_suite` identity, capability, definition, and registry
   modules before editing.
4. Add the smallest durable declaration contract that can express product and
   service needs without carrying semantic validation.
5. Add registry validation and focused tests in `domain/editor/editor_shell`.
6. Update the active target architecture with the declaration boundary and
   examples.
7. Run required validation and write closeout evidence under
   `docs-site/src/content/docs/reports/closeouts/wr-038-product-and-service-capability-declarations/closeout.md`
   before changing roadmap or production completion evidence.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-038 implementation closeout:

```text
cargo fmt
cargo test -p editor_shell tool_suite
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

## Stop Conditions

- Stop before product code if WR-038 is not `current_candidate` or the scoped
  goal command does not select implementation.
- Stop if implementation requires write scopes outside
  `domain/editor/editor_shell/src/tool_suite` or the active target architecture
  design.
- Stop if product semantics, product graph formation, service execution,
  runtime IO, app shell dispatch, host presets, or external sandbox behavior
  becomes necessary.
- Stop if an ADR is needed because the design would change ownership of product
  semantics or service process authority.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-038-product-and-service-capability-declarations/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` rerun result;
- confirmation that no Rust product code, active design content, or production
  state changed for this contract-writing action.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-038 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.
