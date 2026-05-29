---
title: WR-128 UI Designer Workbench V1 Closure Package Session Source Truth Contract
description: Design-first implementation contract for PM-UI-DESIGNER-WB-V1-CLOSURE-002 package, document, session, source-version, persistence, reload, diagnostics, and evidence source-truth closure.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-workbench-product-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
related_reports:
  - ../wr-127-ui-designer-workbench-v1-closure-track-governance/plan.md
  - ../../closeouts/pm-ui-designer-wb-v1-closure-001-governance-drift-audit-and-correction-contract/closeout.md
  - ../../roadmap-intake/2026-05-26-ui-designer-workbench-v1-closure-package/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-128 UI Designer Workbench V1 Closure Package Session Source Truth Contract

## Goal

Define the decision-complete source-truth contract for
`PM-UI-DESIGNER-WB-V1-CLOSURE-002` and `WR-128` before package/session product
implementation starts.

This contract is produced from:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-002 --roadmap WR-128
```

The command currently classifies the next action as `design_first`. This action
does not authorize product code. It records the owners, invariants, write
scopes, implementation sequence, validation, and closeout requirements needed
before `WR-128` can be promoted for implementation.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml` milestone
  `PM-UI-DESIGNER-WB-V1-CLOSURE-002`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` item `WR-128`.
- Governance prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-v1-closure-001-governance-drift-audit-and-correction-contract/closeout.md`.
- Product design gates:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`,
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`,
  and
  `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`.
- Current app session code truth:
  `apps/runenwerk_editor/src/shell/self_authoring.rs` module
  `self_authoring` owns `SelfAuthoringWorkspaceState`,
  `selected_source_version_label`, project-package save/load, last-applied
  snapshots, rollback records, and evidence-packet freshness state.
- Current app project IO truth:
  `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs` module
  `editor_lab_project` owns app-level project-package serialization,
  deserialization, load reports, save reports, activation reports, and rollback
  report vocabulary.
- Current provider projection truth:
  `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` function
  `ui_designer_workbench_view_model` projects the current source version,
  readiness, persistence, reload, rollback, and evidence summaries.

## Readiness

Initial `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-002 --roadmap WR-128`
reported:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B5
Roadmap dependencies: WR-127:completed
Milestone links WR item: yes
Next action: design_first
```

After the design-first contract and WR metadata were accepted, the current
readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-127:completed
Milestone links WR item: yes
Next action: write_promotion_contract
Promotion preflight: promotable
Suggested command: task roadmap:promote -- --id WR-128 --state current_candidate --evidence "<accepted evidence>"
```

`WR-128` is ready for promotion only as a package/session implementation row.
Before product code starts, the coordinator must run the suggested promotion
command with accepted evidence, rerun `task ai:goal`, and then follow the
implementation action reported for the current state.

After promotion, the implementation-readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-127:completed
Milestone links WR item: yes
Next action: write_implementation_contract
```

This document is the decision-complete implementation contract for the
current-candidate row. The next coding pass may implement only the
package/session source-truth slice described here, then must run focused tests,
close out PM-002, archive WR-128, and rerun `task ai:goal` before any PM-003
work.

Accepted promotion evidence:

```text
Accepted PM-UI-DESIGNER-WB-V1-CLOSURE-002 design-first package/session source-truth contract at docs-site/src/content/docs/reports/implementation-plans/wr-128-ui-designer-workbench-v1-closure-package-session-source-truth/plan.md; WR-127 governance is completed, design gates are accepted, write scopes are bounded, and production:plan reports WR-128 promotable.
```

`WR-128` must not start product implementation until:

- this active contract in WR-128 write scopes;
- accepted design gates verified;
- exact implementation scopes and non-goals recorded;
- focused validation and closeout paths recorded;
- `task roadmap:promote -- --id WR-128 --state current_candidate --evidence
  "<accepted evidence>"` passes;
- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` rerun after
  promotion.

Those conditions are satisfied for the current-candidate implementation pass
when the coordinator selects `execute_next_wr_implementation_contract`.

## Ownership And Invariants

Owners:

- `domain/ui/ui_definition` owns generic authored UI definition, document,
  Canonical UI IR, source-map, schema-version, persistence, migration, diff,
  and activation contracts.
- `domain/editor/editor_definition` owns editor-specific document and operation
  adapter vocabulary that translates editor actions into generic UI contracts.
- `domain/editor/editor_shell` owns app-neutral workbench view models and
  surface readiness state.
- `apps/runenwerk_editor/src/shell/self_authoring.rs` owns concrete
  reconstructable app session state, selected document state, save/load
  orchestration, last-applied snapshots, rollback records, and evidence packet
  cache.
- `apps/runenwerk_editor/src/shell/editor_lab_project/mod.rs` owns app-level
  project-package IO and reports. It must not become generic UI source truth.

Invariants:

- Package and document ids, schema versions, source versions, draft snapshots,
  applied snapshots, and rollback points are explicit and testable.
- Session state is reconstructable from the open package plus app preferences.
- Provider/runtime cache state is derived and cannot become source truth.
- Reload and rollback produce typed diagnostics and preserve invalid input
  evidence when parsing or migration fails.
- Evidence packets reference package/document/source-version provenance and are
  stale when they no longer match the selected source version.

## Implementation Scope

Allowed for a later promoted `WR-128` implementation:

- add or refine typed source-version/package/session contracts in the owning
  domain or app modules above;
- route standalone and embedded workbench session state through one
  reconstructable package/session model;
- make save, load, reload, invalid-load preservation, apply snapshot, rollback,
  diagnostics, and evidence freshness behavior explicit;
- add focused tests for round trip, reload, rollback, invalid input
  preservation, stale evidence detection, and source-version projection;
- update docs and closeout evidence for PM-002 only.

Forbidden under `WR-128`:

- recipe catalog insertion;
- hierarchy/canvas/inspector authoring beyond source-version projection;
- operation diff/apply/undo/redo parity beyond package/session prerequisites;
- scenario matrix, game.runtime compatibility workflow, performance baselines,
  or final runtime-proven closeout;
- moving generic UI source truth into `apps/runenwerk_editor`;
- implementing concrete game HUD runtime behavior.

## Implementation Steps

1. Inspect current `SelfAuthoringWorkspaceState`,
   `EditorLabProjectPackage`, provider projections, and focused tests before
   editing.
2. Define any missing source-version, package/session, reload, rollback,
   invalid-input, and evidence-freshness contract gaps.
3. Keep generic UI truth in `domain/ui`; add only app-owned orchestration in
   `apps/runenwerk_editor`.
4. Update standalone and embedded workbench initialization only if needed to
   share the same package/session state.
5. Add or update behavior tests covering package round trip, reload,
   last-applied snapshot restore, rollback record, invalid package
   preservation, and stale evidence reporting.
6. Update usage/docs/closeout evidence only for PM-002.

## Validation

Focused validation for a later implementation:

```text
cargo test -p runenwerk_editor self_authoring
cargo test -p runenwerk_editor editor_lab_project
cargo test -p runenwerk_editor ui_designer
```

Planning and closeout validation:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-002 --roadmap WR-128
task roadmap:promote -- --id WR-128 --state current_candidate --evidence "<accepted evidence>"
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task puml:validate
git diff --check
task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE
```

Use `./quiet_full_gate.sh` only if the implementation broadens beyond the
focused package/session source-truth slice.

## Acceptance Criteria

This design-first contract action is complete when:

- this file exists with `status: active`;
- `WR-128` write scopes include this contract path;
- `PM-UI-DESIGNER-WB-V1-CLOSURE-002` links `WR-128`;
- `task production:plan` reports `write_promotion_contract` and a promotable
  preflight before promotion;
- accepted design gates, owner boundaries, non-goals, validation, and closeout
  requirements are recorded;
- roadmap, production, docs, planning, PUML, and whitespace validations pass.

A later implementation closeout is complete only when:

- package/document/source-version/session state is reconstructable;
- reload and rollback prove authored state survival;
- invalid load preserves typed diagnostics and original invalid source;
- evidence freshness is source-version-aware;
- standalone and embedded hosts consume the same package/session contract;
- closeout evidence links focused tests and planning validation.

## Stop Conditions

Stop instead of implementing when:

- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` no longer selects
  PM-002 as the next non-completed legal action;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-002 --roadmap WR-128`
  does not report a design-first, promotion, or implementation action matching
  the current state;
- `task roadmap:promote` fails for anything other than exact WR-128 metadata;
- package/session ownership requires generic UI truth to move into app code;
- implementation needs recipe insertion, operation parity, scenario evidence,
  performance baselines, or game-runtime HUD behavior;
- validation fails and the failure is not an exact metadata repair for WR-128
  or PM-002.

## Closeout Requirements

The later PM-002 closeout must include:

- this contract path;
- exact code files/modules/functions changed;
- focused test results;
- package/session source-truth proof;
- reload and rollback proof;
- invalid-input diagnostic proof;
- source-versioned evidence freshness proof;
- roadmap/production render, validate, and check results;
- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` rerun showing the next
  legal action.

Expected completion quality for PM-002: `runtime_proven` if the package/session
workflow is proven through app/runtime tests; otherwise `bounded_contract` with
explicit known gaps. `perfectionist_verified` is not in scope.
