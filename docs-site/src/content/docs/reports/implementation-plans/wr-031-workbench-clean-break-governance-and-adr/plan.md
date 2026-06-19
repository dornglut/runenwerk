---
title: WR-031 Workbench Clean-Break Governance And ADR Contract
description: Design-first governance and readiness contract for accepting the Workbench clean break before PT-WB-CAP implementation removes legacy compatibility paths.
status: active
owner: workspace/editor
layer: workspace/domain/app
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
related_adrs:
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-index.md
  - ../../../workspace/roadmap-decision-register.md
related_reports:
  - ../wr-032-typed-suite-surface-profile-and-provider-handles/plan.md
  - ../wr-033-remove-legacy-tool-surface-identity/plan.md
  - ../wr-034-registry-backed-workspace-profiles/plan.md
  - ../wr-035-clean-persistence-format/plan.md
  - ../wr-036-material-lab-clean-migration-proof/plan.md
---

# WR-031 Workbench Clean-Break Governance And ADR Contract

## Goal

Establish the design-first governance package for `WR-031` under
`PM-WB-CAP-001` in the `PT-WB-CAP` production track. The row exists to make
legacy Workbench compatibility removal an explicit platform decision before
code deletes or bypasses `ToolSurfaceKind`, old workspace persistence, legacy
profile construction, provider-request fallback metadata, or Material Lab
compatibility routes.

This contract does not implement product code, promote roadmap state, mark
`WR-031` complete, or complete `PM-WB-CAP-001`. It records the governance
decision, architecture review result, exact owners, and evidence required
before downstream WR-032 through WR-036 implementation can start.

## Source Of Truth

- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-031`.
  The row is `current_candidate`, blocker `B3`, has no dependencies, and
  requires accepted ADR 0012 plus roadmap and production records that approve
  dropping legacy Workbench persisted layout support.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-WB-CAP` and active milestone `PM-WB-CAP-001`. The milestone requires
  stable-key-only Workbench identity, stable-key-only persistence, and
  full-editor plus standalone Material Lab profiles from registry-backed
  composition data.
- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  is the durable accepted decision. It says `ToolSurfaceKind` is not a
  Workbench identity, persistence, provider request, profile construction, or
  Material Lab routing authority.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  is the active implementation design for tool-suite registration, Workbench
  host composition, provider-owned routing, stable surface keys, and clean
  Workbench persistence behavior.
- `AI_GUIDE.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`,
  `GLOSSARY.md`, `TESTING.md`,
  `docs-site/src/content/docs/workspace/architecture-governance-review.md`,
  `docs-site/src/content/docs/workspace/routines/architecture-governance-review-routine.md`,
  and
  `docs-site/src/content/docs/workspace/prompt-templates/architecture-governance-review.md`
  define the architecture-governance routine used for this design-first pass.
- `docs-site/src/content/docs/reports/implementation-plans/wr-032-typed-suite-surface-profile-and-provider-handles/plan.md`
  through
  `docs-site/src/content/docs/reports/implementation-plans/wr-036-material-lab-clean-migration-proof/plan.md`
  are downstream readiness contracts only; they remain blocked for
  implementation until WR-031 has valid completion evidence.

Readiness checks completed for this contract:

- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` classified
  PM-WB-CAP-001 next legal action as `prepare_wr_promotion_contract`.
- `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-031`
  classified WR-031 next action as `design_first`.
- `task ai:architecture-governance -- --task "WR-031 Workbench Clean-Break Governance And ADR" --scope "PT-WB-CAP PM-WB-CAP-001 clean registry-owned Workbench foundation"`
  returned the architecture-governance checklist and stop conditions for this
  scope.
- `task docs:validate` passed after this contract was added.
- `task planning:validate` passed after this contract was added, covering
  roadmap validation/check, production validation/check, and docs validation.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` passed after this
  contract was added and preserved PM-WB-CAP-001 as `active` with
  `prepare_wr_promotion_contract`; WR-031 remains `current_candidate` with
  next action `design_first`.

## Architecture Governance Review

Recommendation: keep WR-031 in design-first governance until the accepted ADR,
active design, roadmap row, production milestone, and closeout evidence are
linked and validated together. Do not implement product code or promote
downstream WR rows from this contract alone.

Scope: `PT-WB-CAP`, `PM-WB-CAP-001`, `WR-031`, ADR 0012, the active
tool-suite registry design, roadmap source rows, and production-track source
rows.

Owner: enabling platform governance for the editor Workbench. The bounded
context is `domain/editor/editor_shell` for app-neutral Workbench contracts,
with `apps/runenwerk_editor` as the concrete Workbench host and provider
composition owner.

Vocabulary: tool suite, stable surface key, profile, provider family, provider
bundle, Workbench host, host capability policy, workspace persistence,
provider-owned route, and unsupported old schema.

Invariants:

- `ToolSurfaceStableKey` and typed suite/profile/provider declarations are
  Workbench identity authority.
- `ToolSurfaceKind` is legacy compatibility metadata only, not identity,
  persistence, provider request, profile construction, or Material Lab routing
  authority.
- Old persisted layouts that rely on legacy surface-kind fields fail with
  clear unsupported-schema diagnostics after the clean break.
- App-owned providers and runtime bridges may compose domain contracts but must
  not move material, texture, procgen, render, or project IO semantics into
  `editor_shell`.

Translation boundaries:

- `domain/editor/editor_shell` owns app-neutral declarations, registry
  validation, provider-family contracts, workspace host contracts, and
  fail-closed diagnostics.
- `apps/runenwerk_editor` owns concrete installed suites, provider
  implementations, runtime bridges, project IO, persistence paths, and
  command execution bridges.
- Owning domains such as `domain/material_graph`, `domain/texture`, and
  `domain/procgen` own source truth, validation, product descriptors, and
  diagnostics.

Dependency direction: allowed direction is foundation to domain to engine to
apps/adapters/tools. WR-031 must preserve `editor_shell` as an app-neutral
domain crate and must not make it depend on app provider implementations,
runtime glue, renderer internals, project IO, or AI/tooling code.

ADR need: no new ADR is required for WR-031 if ADR 0012 remains accepted and
the implementation follows it. A new or amended ADR is required before any
later work changes the clean-break decision, reintroduces compatibility
migration, changes source-of-truth ownership, or accepts external dynamic
component execution.

ATAM-lite summary:

- Quality attributes in tension: compatibility versus clarity, migration
  convenience versus stable-key authority, standalone host flexibility versus
  app-owned runtime composition, and extensibility versus premature external
  plugin loading.
- Decision: favor clean stable-key authority and fail-closed diagnostics over
  transparent migration of legacy persisted layouts.
- Risk: old layouts stop loading and must produce explicit unsupported-schema
  diagnostics rather than ambiguous decode failures.
- Non-risk: Material Lab semantics remain in owning app/domain modules; the
  clean break does not make `editor_shell` a material or texture domain.
- Evidence needed: focused tests and closeouts in WR-032 through WR-036, plus
  roadmap/production render, validate, and check gates before PM-WB-CAP-001
  completion.

Migration shape: Strangler Fig applies to implementation rows, not this
governance contract. WR-032 through WR-036 must freeze legacy paths behind
named compatibility boundaries, route stable-key authority through typed
contracts, prove parity or intentional rejection, then delete or quarantine
legacy compatibility only inside the owning write scopes.

Fitness functions:

- `task docs:validate`;
- `task roadmap:validate`;
- `task production:validate`;
- `task planning:validate`;
- downstream focused Rust tests named in WR-032 through WR-036;
- source or architecture guards that prevent `ToolSurfaceKind` from returning
  as Workbench identity in normal state, profile, provider request,
  persistence, or Material Lab routing paths.

Ownership mode: enabling platform governance. Downstream rows become platform
foundation and stream-aligned product workflow implementation slices only after
this governance row has completed evidence.

Next action recommendation after this contract: create a completed WR-031
closeout and update roadmap/production evidence only after validating that ADR
0012, the active design, and the production/roadmap rows are aligned. If any
alignment gap appears, update the ADR or design before completing WR-031.

## Readiness

Promotion verdict: WR-031 can carry this design-first governance contract, but
it cannot honestly be marked completed by the contract alone.

Blocking conditions before WR-031 completion:

- There is no completed closeout yet at
  `docs-site/src/content/docs/reports/closeouts/wr-031-workbench-clean-break-governance-and-adr/closeout.md`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` still lists
  WR-031 as `current_candidate`. Completed roadmap items must move to the
  archive source with valid closeout evidence, completion quality, and rendered
  roadmap updates; do not change that in this contract action.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` still lists
  PM-WB-CAP-001 as `active`. The milestone cannot complete until WR-032
  through WR-036 implementation evidence is valid.
- Downstream contracts WR-032 through WR-036 are readiness artifacts only.
  They do not prove the Workbench foundation is implemented.

Readiness conditions already satisfied:

- ADR 0012 exists and is accepted.
- The active tool-suite registry and Workbench host design links ADR 0012 and
  names the clean break.
- `PT-WB-CAP` and `PM-WB-CAP-001` exist in production tracks.
- WR-031 through WR-040 are sequenced in roadmap source and generated index
  docs.

## Implementation Scope

Owning files and exact change locations for a later WR-031 closeout pass:

- `docs-site/src/content/docs/adr/superseded/0012-capability-workbench-clean-break.md`
  module `ADR: Capability Workbench Clean Break`, sections `Status`,
  `Decision`, and `Consequences`. These sections own the durable decision to
  drop legacy Workbench compatibility. Update them only if the decision itself
  changes.
- `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`
  module `Editor Tool Suite Registry And Workbench Host Design`, sections
  `Purpose`, `Decision`, `Ownership Rules`, `Stable Surface Keys And
  Persistence`, `Workbench Host Model`, and `Migration Plan`. These sections
  own the implementation path that downstream WR rows must follow.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` row `WR-031`.
  A later closeout pass may move the item out of active roadmap source only
  through the repository roadmap completion routine and must include valid
  closeout evidence, completion quality, and rendered docs.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` track
  `PT-WB-CAP`, milestone `PM-WB-CAP-001`. The milestone must stay `active`
  until the non-deferred foundation rows have implementation evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-031-workbench-clean-break-governance-and-adr/closeout.md`
  is the expected future closeout artifact for WR-031 governance completion.
- `docs-site/src/content/docs/workspace/production-track-index.md`,
  `docs-site/src/content/docs/workspace/production-milestone-register.md`,
  `docs-site/src/content/docs/workspace/roadmap-index.md`,
  `docs-site/src/content/docs/workspace/roadmap-decision-register.md`, and
  generated roadmap diagrams are generated outputs. Update them only through
  `task production:render` and `task roadmap:render` after source evidence
  changes.

Explicit non-goals:

- Do not edit Rust product code in WR-031.
- Do not implement typed handles, legacy identity deletion, registry-backed
  profiles, clean persistence, or Material Lab proof work; WR-032 through
  WR-036 own those slices.
- Do not mark PM-WB-CAP-001 completed from governance evidence alone.
- Do not restore legacy persisted layout migration or compatibility fallback.
- Do not introduce external dynamic component support.
- Do not move app-owned provider or runtime semantics into `editor_shell`.

## Acceptance Criteria

The later WR-031 completion pass is complete only when all criteria below are
true:

- ADR 0012 remains accepted and explicitly approves dropping legacy Workbench
  persisted layout support.
- The active Workbench design remains aligned with ADR 0012 and names the same
  source-of-truth, dependency direction, migration, and enforcement rules.
- Roadmap and production source records link WR-031 to PT-WB-CAP and
  PM-WB-CAP-001 without claiming implementation completion.
- A completed WR-031 closeout records architecture governance, source records,
  validation output, no-product-code confirmation, and downstream blockers.
- `task docs:validate`, `task roadmap:validate`, `task production:validate`,
  and `task planning:validate` pass after any source evidence updates.

## Implementation Steps

1. Re-run `task production:plan -- --milestone PM-WB-CAP-001 --roadmap WR-031`
   before changing governance source files.
2. Re-read ADR 0012 and the active Workbench design. Stop if they diverge on
   legacy persistence, `ToolSurfaceKind`, provider request authority, or
   Material Lab routing.
3. If the ADR or design is missing a durable decision, update the owning
   document first and validate docs before touching roadmap source.
4. Create
   `docs-site/src/content/docs/reports/closeouts/wr-031-workbench-clean-break-governance-and-adr/closeout.md`
   with completed governance evidence.
5. Move WR-031 through the repository roadmap completion routine only if the
   closeout exists, all validation gates pass, and generated roadmap docs can
   be rendered.
6. Keep PM-WB-CAP-001 `active` unless and until WR-032 through WR-036
   implementation and closeout evidence also complete.
7. Rerun `task ai:goal -- --track PT-WB-CAP --scope non-deferred` after the
   closeout/source update to identify the next legal action.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

Required validation before a later WR-031 completion closeout:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-WB-CAP --scope non-deferred
```

No Rust validation is required for WR-031 unless the scope changes and product
code is touched, which should be treated as a stop condition for this roadmap
row.

## Stop Conditions

- Stop before product code; WR-031 is governance only.
- Stop if ADR 0012 is not accepted or no longer says old legacy-dependent
  layouts are unsupported.
- Stop if the active Workbench design conflicts with ADR 0012.
- Stop if completing WR-031 would require claiming WR-032 through WR-036
  behavior that has not been implemented and validated.
- Stop if roadmap completion requires a closeout artifact that does not exist.
- Stop if completing WR-031 would mark PM-WB-CAP-001 complete before
  implementation evidence exists.
- Stop if a proposed path reintroduces `ToolSurfaceKind` as Workbench identity,
  provider request authority, profile construction authority, persistence
  authority, or Material Lab routing authority.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-031-workbench-clean-break-governance-and-adr/plan.md`;
- `task docs:validate` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` rerun result;
- confirmation that no product code, roadmap state, or production state
  changed;
- remaining blockers: completed WR-031 closeout and roadmap/production
  evidence update before downstream implementation promotion.

The expected completion-quality tier for the contract-writing action is
`bounded_contract`. WR-031 itself remains incomplete until governance closeout
evidence, roadmap render/validate/check, production render/validate/check, and
goal rerun complete.

## Contract-Writing Closeout Evidence

Status as of 2026-05-20: completed for the contract-writing action only.

Changed artifact:

- `docs-site/src/content/docs/reports/implementation-plans/wr-031-workbench-clean-break-governance-and-adr/plan.md`

Validation:

- `task docs:validate` passed after the contract was added.
- `task planning:validate` passed after the contract was added, covering
  roadmap validation/check, production validation/check, and docs validation.
- `task ai:goal -- --track PT-WB-CAP --scope non-deferred` was rerun after
  validation and still reported PM-WB-CAP-001 next legal action as
  `prepare_wr_promotion_contract`.

Closeout result:

- No Rust product code changed.
- No roadmap state changed.
- No production-track state changed.
- WR-031 remains `current_candidate` until a completed governance closeout and
  source evidence update exist.
- WR-032 through WR-036 remain blocked for implementation by WR-031 completion.

## Perfectionist Closeout Audit

This contract-writing action cannot honestly be `perfectionist_verified`
because it deliberately does not implement or complete the clean Workbench
foundation. The known quality gap is intentional: WR-031 governance completion
and WR-032 through WR-036 implementation evidence remain future work.

The later WR-031 closeout must guard against:

- treating accepted ADR text as sufficient without a completed closeout;
- completing WR-031 while roadmap or production generated docs are stale;
- marking PM-WB-CAP-001 complete from governance evidence alone;
- allowing downstream implementation before WR-031 completion evidence exists;
- accepting a compatibility migration path that contradicts ADR 0012;
- losing the `editor_shell` versus `apps/runenwerk_editor` ownership boundary.
