---
title: WR-032 Typed Suite, Surface, Profile, And Provider Handles Contract
description: Promotion and implementation-readiness contract for typed Workbench suite, surface, profile, provider bundle, and composition handles.
status: active
owner: editor
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-decision-register.md
related_adrs:
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
---

# WR-032 Typed Suite, Surface, Profile, And Provider Handles Contract

## Status

WR-032 is the first implementation-readiness slice inside
`PM-WB-CAP-001`. It prepares the typed Workbench handle and composition
foundation needed before later rows can delete legacy tool-surface identity,
replace enum-backed profiles, reject old persistence, and prove Material Lab
through clean registry composition.

This contract is the promotion package for WR-032. It does not implement
product code or complete PM-WB-CAP-001.

## Goal

Replace ad hoc suite, surface, profile, provider, and composition construction
with typed Workbench handles and provider bundles.

The implementation is acceptable when `editor_shell` owns typed,
stable-key-preserving handle contracts and the app Workbench host consumes
those contracts through one `WorkbenchCompositionBuilder`, without making
legacy `ToolSurfaceKind` a suite/profile/provider authority.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and active milestone `PM-WB-CAP-001`. The milestone requires
  stable-key-only Workbench identity, profile construction, provider requests,
  persistence, and Material Lab routing across WR-031 through WR-036.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-032`.
  The row depends on completed `WR-031` and names `SuiteRef`, `SurfaceRef`,
  `ProfileRef`, `ProviderBundle`, `WorkbenchCompositionBuilder`, and
  `ToolSuiteProfileDefinition` as the next evidence. WR-030 is a separate
  product-workflow model/mesh proof and is not a semantic prerequisite for
  typed Workbench handle implementation.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is accepted and makes the clean break mandatory. It forbids
  `ToolSurfaceKind` as Workbench identity, persistence, provider request,
  profile construction, or Material Lab routing authority.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is the active design for the tool-suite registry, provider-owned routing, and
  Workbench host composition boundary.
- `domain/editor/editor_shell/src/tool_suite` owns app-neutral suite,
  surface, profile, provider-family, provider-bundle, host-policy, and
  composition contracts. It must not import app provider, material, texture,
  procgen, render, runtime, or project IO types.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` owns compiled-in
  Workbench host composition and concrete provider-registry pairing.
- `apps/runenwerk_editor/src/shell/tool_suites` and
  `apps/runenwerk_editor/src/material_lab/tool_suite.rs` own current
  compiled-in app suite declarations until a future app-support crate is
  accepted.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-001 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-032`
  classified WR-032 next action as `write_promotion_contract`.
- `task planning:validate` passed after this contract was added, covering
  roadmap validation/check, production validation/check, and docs validation.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` passed after this
  contract was added and preserved PM-WB-CAP-001 as `active` with
  `prepare_wr_promotion_contract`.

## Readiness

Promotion verdict: WR-032 can carry a decision-complete promotion and
implementation-readiness contract. WR-031 governance is complete, and WR-030
is not a Workbench typed-handle prerequisite.

Blocking conditions before code starts:

- WR-031 is completed by
  `docs-site/src/content/docs/reports/closeouts/wr-031-workbench-clean-break-governance-and-adr/closeout.md`.
  ADR 0012 and the production/roadmap governance records are now available as
  implementation authority.
- The WR-032 roadmap write scopes must cover every file the implementation
  will edit. They need the domain editor-shell tool-suite/workspace owners,
  `apps/runenwerk_editor/src/shell/workbench_host.rs`,
  `apps/runenwerk_editor/src/shell/tool_suites`, and
  `apps/runenwerk_editor/src/material_lab/tool_suite.rs`.
- The WR-032 roadmap row must stay at the implementation gate (`B2`) after
  WR-031 completion. Do not use WR-030 as a machine-visible dependency unless
  typed Workbench handles begin to depend on model/mesh pixel proof, which
  would be a separate design decision.

Architecture governance review, 2026-05-20:

- Scope reviewed:
  `PT-WB-CAP PM-WB-CAP-001 domain/editor/editor_shell tool_suite workspace
  and apps/runenwerk_editor Workbench host suite declarations`.
- DDD owner: `domain/editor/editor_shell/src/tool_suite` owns app-neutral
  suite, surface, profile, provider-family, provider-bundle, composition, and
  capability-policy vocabulary. `domain/editor/editor_shell/src/workspace`
  owns profile-facing stable-key workspace contracts. `apps/runenwerk_editor`
  owns concrete installed suite declarations, Workbench host composition,
  provider implementations, project IO, runtime/render adapters, and command
  execution bridges.
- Clean Architecture check: the contract is legal only while `editor_shell`
  remains semantic-free and does not import app, provider implementation,
  material, texture, renderer, runtime, or persistence adapter types.
  Concrete Material Lab and shell suite declarations stay in
  `apps/runenwerk_editor`.
- ADR/design check: ADR 0012 and the active Workbench host design are
  sufficient for WR-032. No new ADR is required unless the implementation moves
  provider execution, material semantics, renderer authority, persistence
  ownership, external component trust, or app-owned IO into `editor_shell`.
- ATAM-lite tradeoff: strict typed composition and fail-closed registry
  diagnostics are favored over preserving loose raw-string construction.
  The cost is explicit compiled-in declaration boilerplate; that cost is
  acceptable because raw strings remain localized to declaration inputs while
  downstream composition uses typed handles.
- Strangler boundary: legacy `ToolSurfaceKind` compatibility may remain only in
  explicit legacy/migration seams such as
  `domain/editor/editor_shell/src/tool_suite/legacy.rs`; WR-033 owns deletion
  of remaining legacy authority.
- Fitness functions before completion: exact stable-key identity tests,
  duplicate/unknown suite/profile/provider diagnostics, Workbench host
  composition tests for full-editor and Material Lab presets, and source guards
  preventing `ToolSurfaceKind` from becoming suite, profile, provider, or
  Workbench composition authority.
- Current code audit: `SuiteRef`, `SurfaceRef`, `ProfileRef`,
  `ProviderBundle`, `WorkbenchCompositionBuilder`, and
  `ToolSuiteProfileDefinition` already exist in `editor_shell`, and
  `apps/runenwerk_editor/src/shell/workbench_host.rs` already consumes the
  builder for suite/provider composition. That is not enough to mark WR-032
  complete from this planning slice: the later implementation closeout still
  needs profile validation, typed helper adoption, source-guard evidence, and
  the focused validation commands listed below.
- Recommendation: do not complete WR-032 from this contract alone. Promote the
  row through the roadmap workflow, then implement only the typed-handle slice
  covered by the write scopes and run the focused validation and closeout path.

## Implementation Scope

Owning modules and exact change locations for the later implementation pass:

- `domain/editor/editor_shell/src/tool_suite/identity.rs` module:
  strengthen typed handle construction around `SuiteRef`, `SurfaceRef`,
  `ProfileRef`, `ProviderFamilyId`, and capability keys. Keep raw string
  parsing at declaration boundaries and tests only.
- `domain/editor/editor_shell/src/tool_suite/definition.rs` module:
  make suite, surface, profile, and provider-family declarations consume typed
  handles instead of direct raw-string ids or unstructured struct literals.
  Preserve stable keys exactly; do not derive keys from labels or debug text.
- `domain/editor/editor_shell/src/tool_suite/registry.rs` module:
  keep `ToolSuiteRegistry`, `ToolSurfaceRegistry`, `ProviderBundle`,
  `ProviderFamilyProviderMap`, `WorkbenchComposition`, and
  `WorkbenchCompositionBuilder` as the validation and composition boundary.
  Add any missing duplicate, unknown-family, profile-surface, provider-bundle,
  and deterministic-order tests here.
- `domain/editor/editor_shell/src/workspace/profile.rs` module:
  consume `ToolSuiteProfileDefinition` and `SurfaceRef` for the profile-facing
  default surface declarations needed by the later WR-034 registry-backed
  profile row. WR-032 may add typed definitions and tests, but it must not
  switch all live profile authority before WR-034.
- `domain/editor/editor_shell/src/workspace/state.rs` module:
  use typed registry lookup artifacts only where they are required to prove
  exact stable-key identity. Do not remove `ToolSurfaceKind` live state in
  WR-032; WR-033 owns that deletion.
- `apps/runenwerk_editor/src/shell/workbench_host.rs` module:
  build full-editor and Material Lab host compositions through
  `WorkbenchCompositionBuilder`, `ProviderBundle`, profile definitions, and
  provider-family assignments. Keep concrete provider registry construction
  app-owned.
- `apps/runenwerk_editor/src/shell/tool_suites/mod.rs` module and child suite
  modules:
  replace ad hoc raw string suite/surface/provider construction with typed
  helper constructors after the roadmap write scope is expanded.
- `apps/runenwerk_editor/src/material_lab/tool_suite.rs` module:
  declare Material Lab suite and surfaces through the same typed helper path as
  the app shell suites after the roadmap write scope is expanded.

Required implementation steps:

1. Add explicit constructor APIs for `EditorToolSuite`,
   `ProviderFamilyDefinition`, `ToolSurfaceDefinition`, and
   `ToolSuiteProfileDefinition` that accept validated typed handles. The
   constructors belong in
   `domain/editor/editor_shell/src/tool_suite/definition.rs`.
2. Make `WorkbenchCompositionBuilder` validate profile definitions against the
   installed `ToolSurfaceRegistry`: duplicate profile refs, unknown default
   surfaces, and duplicate provider assignments must fail before an app host is
   constructed.
3. Keep `ProviderBundle` as the provider-family to provider-id authority. The
   Workbench host must not derive provider selection from surface labels,
   route strings, or legacy surface kinds.
4. Add exact stable-key tests in
   `domain/editor/editor_shell/src/tool_suite/identity.rs` and
   `domain/editor/editor_shell/src/tool_suite/registry.rs` proving handles
   preserve the source stable ids without normalization beyond validation.
5. Add composition tests in
   `apps/runenwerk_editor/src/shell/workbench_host.rs` proving full-editor and
   Material Lab compositions expose the expected suites, profiles, provider
   bundles, and provider-family mappings through the builder.
6. After the write-scope gate is fixed, update the app suite factories in
   `apps/runenwerk_editor/src/shell/tool_suites` and
   `apps/runenwerk_editor/src/material_lab/tool_suite.rs` to use typed helper
   constructors. Raw string literals may remain only as declaration inputs near
   the compiled-in suite constants.
7. Add source guards that prevent WR-032 from reintroducing
   `ToolSurfaceKind` as suite, profile, provider, or Workbench composition
   authority. Compatibility helpers may remain only in
   `domain/editor/editor_shell/src/tool_suite/legacy.rs` until WR-033 removes
   them.

## Non-Goals

- No product code in this contract-writing slice.
- No implementation before WR-032 is promoted to `current_candidate` with
  disjoint write scopes.
- No deletion of `ToolSurfaceKind` live workspace identity; WR-033 owns that.
- No switch to registry-only workspace profiles; WR-034 owns that.
- No persistence schema cleanup; WR-035 owns that.
- No Material Lab clean migration proof; WR-036 owns that.
- No host command/product/resource policy enforcement beyond carrying
  `HostCapabilityPolicy`; WR-037 owns enforcement.
- No product or service capability plane; WR-038 owns that.
- No external dynamic components, package trust, ABI, sandbox, or plugin
  runtime work.

## Acceptance Criteria

- `SuiteRef`, `SurfaceRef`, `ProfileRef`, provider-family ids,
  `ProviderBundle`, `ToolSuiteProfileDefinition`, and
  `WorkbenchCompositionBuilder` are the public typed path for Workbench
  composition.
- Raw strings are localized to compiled-in declarations and fallible parsing
  boundaries; downstream composition and tests use typed handles.
- Duplicate suite ids, duplicate stable surface keys, duplicate provider-family
  ids, duplicate profile refs, unknown profile default surfaces, unknown
  provider families, duplicate provider assignments, and unknown provider ids
  fail with explicit diagnostics.
- Full-editor and Material Lab Workbench host compositions are built through
  the same typed composition path.
- No WR-032 code path makes `ToolSurfaceKind` the suite/profile/provider
  authority or introduces a replacement compatibility enum.
- The app-owned provider registry remains concrete app composition. The domain
  `editor_shell` crate remains app-neutral and semantic-free.

## Validation

Contract-writing validation:

```text
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before any later WR-032 implementation closeout:

```text
cargo test -p editor_shell tool_suite
cargo test -p editor_shell workspace::profile
cargo test -p runenwerk_editor workbench_host
cargo check -p runenwerk_editor
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
```

Run `task docs:validate` as the smallest focused check when only this contract
changes. Run `task planning:validate` when the contract is finalized because
the production-track goal is planning-state sensitive.

## Stop Conditions

Stop before code changes if:

- WR-032 remains outside the current selected roadmap execution state;
- the WR-032 write scopes drift and no longer cover required app suite
  declaration files;
- implementation would delete legacy live identity, change persistence support,
  or switch profile authority beyond the WR-032 typed-handle foundation;
- `editor_shell` would import app, material, texture, procgen, renderer,
  runtime, or provider implementation types;
- provider selection would depend on labels, debug formatting, route strings,
  or legacy enum reverse mapping;
- the builder cannot produce explicit diagnostics for duplicate or unknown
  suite, surface, profile, provider-family, provider-bundle, or provider-id
  inputs.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md`;
- `task planning:validate` passed;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` passed after the
  contract-writing action;
- confirmation that no product code or production state changed;
- confirmation that roadmap evidence changed only to complete WR-031
  governance and repair WR-032 readiness write scopes;
- remaining blocker: WR-032 still needs the typed-handle implementation pass
  and implementation closeout before PM-WB-CAP-001 can advance.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-032 itself remains incomplete until implementation,
focused validation, closeout evidence, roadmap render/validate/check, and
production render/validate/check complete.

## Contract-Writing Closeout Evidence

Status as of 2026-05-20: completed for the contract-writing action only.

Changed artifact:

- `docs-site/src/content/docs/reports/implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md`

Validation:

- `task planning:validate` passed after the contract was added, after its
  validation note was recorded, and after the WR-032 roadmap row was refreshed
  with app suite declaration write scopes.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-032` was
  rerun and reported `WR-031:completed`.
- The WR-032 roadmap row now treats WR-030 as unrelated product-workflow work,
  not as typed Workbench handle readiness.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` was rerun after
  validation.

Focused validation audit, 2026-05-20:

- `cargo test -p editor_shell tool_suite` passed with 29 matching tests. This
  proves the current typed identity, registry, provider-bundle, composition
  builder, host policy, and legacy-boundary tests compile and pass.
- `cargo test -p editor_shell workspace_profile` passed with zero matching
  tests. That filter was stale and is not valid evidence for workspace profile
  coverage.
- `cargo test -p editor_shell workspace::profile` passed with 26 matching
  tests. The roadmap rows and contracts were repaired to use this real profile
  filter.
- `cargo test -p runenwerk_editor workbench_host` passed with 37 matching
  tests. This proves the current app host uses the Workbench composition path,
  provider-family map, stable-key Material Lab mounting, and registry-aware
  default profile construction covered by existing tests.
- `cargo check -p runenwerk_editor` passed.
- These validation results strengthen promotion-readiness evidence only. They
  do not complete WR-032 because the typed-handle implementation and closeout
  still need to run.

Closeout result:

- No Rust product code changed.
- Roadmap evidence changed for WR-031 completion and WR-032 readiness. WR-032
  now includes `apps/runenwerk_editor/src/shell/tool_suites`,
  `apps/runenwerk_editor/src/material_lab/tool_suite.rs`, and this contract
  path in its write scopes.
- No production-track state changed.
- WR-032 remains incomplete until the typed-handle implementation pass,
  focused validation, closeout, and roadmap/production checks complete.
- Downstream PM-WB-CAP milestones remain dependency-waiting and must not be
  implemented from this contract alone.

## Implementation Closeout Evidence

Status as of 2026-05-20: completed for the bounded WR-032 implementation
slice.

Changed artifacts:

- `domain/editor/editor_shell/src/tool_suite/definition.rs`
- `domain/editor/editor_shell/src/tool_suite/registry.rs`
- `apps/runenwerk_editor/src/shell/workbench_host.rs`
- `apps/runenwerk_editor/src/shell/tool_suites/mod.rs`
- `apps/runenwerk_editor/src/material_lab/tool_suite.rs`
- `docs-site/src/content/docs/reports/closeouts/wr-032-typed-suite-surface-profile-and-provider-handles/closeout.md`

Implementation evidence:

- `EditorToolSuite`, `ProviderFamilyDefinition`, and
  `ToolSurfaceDefinition` now have typed constructor paths.
- `WorkbenchCompositionBuilder` rejects duplicate profile refs and unknown
  profile default surfaces before building a host composition.
- `RunenwerkWorkbenchHost` carries typed profile definitions for full-editor
  and Material Lab host modes and feeds them through the same composition
  builder used for suites, provider bundles, and host policy.
- Shell suite and Material Lab declarations now route through typed
  constructors instead of direct struct literals.

Validation:

- `cargo test -p editor_shell tool_suite` passed with 31 matching tests.
- `cargo test -p editor_shell workspace::profile` passed with 26 matching
  tests.
- `cargo test -p runenwerk_editor workbench_host` passed with 39 matching
  tests.
- `cargo check -p runenwerk_editor` passed.

Closeout result:

- WR-032 is ready to archive as `completed` with completion quality
  `bounded_contract`.
- PM-WB-CAP-001 remains active. WR-033, WR-034, WR-035, and WR-036 still own
  the remaining clean-foundation work.

## Perfectionist Closeout Audit

The completed WR-032 implementation cannot honestly be
`perfectionist_verified` because it deliberately leaves the clean-break chain
to downstream rows. Typed Workbench composition is now implemented for the
bounded suite/profile/provider foundation, but legacy identity removal,
registry-only profile authority, clean persistence, and Material Lab clean
migration proof remain visible WR-033 through WR-036 work.

The later implementation closeout must guard against:

- descriptor-only types claimed as live Workbench composition;
- stable-key strings copied through the system without typed handle validation;
- provider bundles that are built but not consumed by the app host;
- profile definitions that are carried by the builder but not validated against
  installed surface definitions;
- source guards that only check Material Lab while other suite declarations
  continue using ad hoc construction;
- `editor_shell` gaining semantic ownership of material, texture, procgen,
  render, runtime, or app provider behavior.
