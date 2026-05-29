---
title: WR-131 UI Designer Workbench V1 Closure Scenario Matrix Game Runtime Evidence And Performance Baselines Contract
description: Design-first implementation contract for PM-UI-DESIGNER-WB-V1-CLOSURE-005 scenario matrix, game.runtime compatibility descriptors, source-versioned evidence packets, and measured performance baselines.
status: active
owner: editor
layer: domain/ui / domain/editor / app
canonical: true
last_reviewed: 2026-05-26
related_designs:
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-production-readiness-and-evidence-design.md
  - ../../../design/active/game-runtime-ui-projection-and-hud-platform-design.md
  - ../../../design/accepted/ui-designer-workbench-product-design.md
related_reports:
  - ../wr-130-ui-designer-workbench-v1-closure-operation-diff-apply-rollback-parity/plan.md
  - ../../closeouts/pm-ui-designer-wb-v1-closure-004-operation-diff-apply-rollback-parity/closeout.md
  - ../../roadmap-intake/2026-05-26-ui-designer-workbench-v1-closure-scenari/proposal.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-131 UI Designer Workbench V1 Closure Scenario Matrix Game Runtime Evidence And Performance Baselines Contract

## Goal

Define the decision-complete design-first contract for
`PM-UI-DESIGNER-WB-V1-CLOSURE-005` and `WR-131` before any scenario/evidence
product code starts.

`WR-131` may close only the scenario matrix, game.runtime compatibility
descriptor, source-versioned evidence packet, read-only fixture/binding
descriptor, validated intent descriptor, and measured performance baseline
slice. It must not claim final V1 closeout, no-gap certification, or concrete
game HUD runtime behavior.

The architecture-governance kickoff was run for:

```text
domain/ui/ui_definition; domain/editor/editor_definition; domain/editor/editor_shell; apps/runenwerk_editor/src/shell; apps/runenwerk_editor/src/runtime; docs-site/src/content/docs/workspace/production-tracks.yaml
```

No ADR is required while implementation preserves the owner boundaries below.
An ADR or accepted design update is required before adding a game UI owner
crate, making editor shell own game.runtime semantics, or turning evidence
packets into source truth.

After the design-first contract and WR metadata were accepted, the current
readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-130:completed
Milestone links WR item: yes
Next action: write_promotion_contract
Promotion preflight: promotable
Suggested command: task roadmap:promote -- --id WR-131 --state current_candidate --evidence "<accepted evidence>"
```

Accepted promotion evidence:

```text
Accepted PM-UI-DESIGNER-WB-V1-CLOSURE-005 design-first scenario matrix game-runtime evidence and performance baseline contract at docs-site/src/content/docs/reports/implementation-plans/wr-131-ui-designer-workbench-v1-closure-scenario-matrix-game-runtime-evidence-and-performance-baselines/plan.md; WR-130 prerequisite is completed, design gates are accepted, write scopes are bounded, and production:plan reports WR-131 promotable.
```

After promotion, the implementation-readiness report is:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-130:completed
Milestone links WR item: yes
Next action: write_implementation_contract
```

This document is the decision-complete implementation contract for the
current-candidate row. The next coding pass may implement only the PM-005
scenario/evidence/performance slice described here, then must run focused
tests, close out PM-005, archive WR-131, update production evidence, and rerun
`task ai:goal` before any PM-006 final closeout work.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml` milestone
  `PM-UI-DESIGNER-WB-V1-CLOSURE-005`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml` item `WR-131`
  after accepted intake is applied.
- Completed prerequisite:
  `WR-130` operation diff/apply/rollback parity closeout.
- Accepted scenario matrix and target-profile design:
  `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`.
- Accepted readiness and evidence design:
  `docs-site/src/content/docs/design/accepted/ui-designer-production-readiness-and-evidence-design.md`.
- Active game.runtime boundary design:
  `docs-site/src/content/docs/design/active/game-runtime-ui-projection-and-hud-platform-design.md`.

## Ownership And Invariants

Owners:

- `domain/ui/ui_definition` owns generic scenario descriptors, target-profile
  compatibility axes, fixture/readiness/evidence packet contracts, and
  fail-closed diagnostic vocabulary.
- `domain/editor/editor_shell` owns app-neutral workbench view models and
  target-profile-aware projection of scenario/evidence state.
- `apps/runenwerk_editor` owns concrete scenario orchestration, preview
  execution, evidence capture, source-version freshness, and measured
  product-path baselines.
- `PT-GAME-RUNTIME-UI` owns future concrete game HUD runtime behavior; WR-131
  may prove only descriptor compatibility.

Invariants:

- Evidence packets are derived from package/document/source-version state and
  become stale when provenance no longer matches.
- game.runtime compatibility is descriptor, fixture, binding, intent, and
  evidence workflow proof only.
- Read-only fixture/binding descriptors must not mutate runtime state.
- Validated intent descriptors may record proposed commands, but must not
  execute game-runtime commands under this row.
- Performance baselines must be measured through product paths, not synthetic
  status rows.

## Implementation Scope

Allowed for a later promoted `WR-131` implementation:

- add or refine generic UI scenario/evidence/readiness contracts in
  `domain/ui/ui_definition` when existing contracts cannot represent PM-005
  evidence without app-owned shortcuts;
- `domain/ui/ui_definition/src/production_readiness/mod.rs` may own generic
  readiness request/report/freshness diagnostics and evidence packet contract
  refinements;
- `domain/ui/ui_definition/src/preview_fixture/mod.rs` may own generic
  target-profile fixture matrix descriptors and replayable fixture axes;
- `domain/ui/ui_definition/src/component_recipe/mod.rs` may be refined only
  for recipe target-profile compatibility descriptors already owned by
  `domain/ui`;
- project scenario matrix and readiness state through
  `domain/editor/editor_shell/src/surfaces/editor_definition.rs` and
  `domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs`;
- implement app-owned scenario orchestration, evidence packet capture,
  freshness policy, unsupported checks, and measured baselines in
  `apps/runenwerk_editor/src/shell/self_authoring.rs` and
  `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`;
- `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` may project
  only app-owned evidence state into app-neutral workbench view models;
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` may add explicit
  capture commands only if they route to app-owned evidence capture and do not
  mutate game-runtime state;
- prove `editor.workbench` and `game.runtime` target profiles with compatible
  and incompatible recipe/fixture/binding/intent descriptors;
- add focused tests for source-version freshness, fail-closed diagnostics,
  read-only fixtures, validated intents, and measured performance baselines.

Forbidden under `WR-131`:

- concrete game HUD runtime behavior;
- final V1 product closeout or no-gap certification;
- making evidence packets source truth;
- moving game.runtime semantic ownership into editor shell or app provider
  projection.

## Implementation Steps

1. Inspect the current scenario/evidence code in
   `apps/runenwerk_editor/src/shell/self_authoring.rs`,
   `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`, and
   `apps/runenwerk_editor/src/shell/providers/self_authoring.rs` before
   editing.
2. Inspect generic readiness and fixture contracts in
   `domain/ui/ui_definition/src/production_readiness/mod.rs` and
   `domain/ui/ui_definition/src/preview_fixture/mod.rs`; add generic fields
   there only when PM-005 cannot be represented without app-local strings.
3. Ensure scenario matrix entries cover at least `editor.workbench` and
   `game.runtime`, including compatible and incompatible recipe/fixture axes,
   safe-area, input, localization, accessibility, sizing, and unsupported
   checks.
4. Ensure evidence packets include package/document/source-version provenance,
   target profile, diagnostics snapshot, fixture/binding/intent descriptors,
   artifact references, performance baseline references, and freshness status.
5. Measure frame build, canvas projection, catalog projection, diagnostics
   projection, and resize relayout through existing product code paths. Do not
   invent synthetic counters that are not consumed by the workbench.
6. Project scenario/evidence state through editor shell/app provider view
   models without making those projections source truth.
7. Add focused tests in the owning domain and app modules, then add the PM-005
   closeout, production evidence gate, and WR-131 archive evidence only after
   focused validation passes.

## Validation

Focused validation for a later implementation:

```text
cargo test -p ui_definition production_readiness
cargo test -p ui_definition preview_fixture
cargo test -p editor_shell ui_designer
cargo test -p runenwerk_editor ui_designer
cargo test -p runenwerk_editor self_authoring
```

Planning validation:

```text
task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-005 --roadmap WR-131
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

Run `./quiet_full_gate.sh` only if implementation expands beyond the bounded
scenario/evidence/performance slice.

## Acceptance Criteria

This implementation-contract action is complete when:

- this file exists with `status: active`;
- `WR-131` is `current_candidate`, links
  `PM-UI-DESIGNER-WB-V1-CLOSURE-005`, and records the implementation write
  scopes needed for the bounded PM-005 slice;
- `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-005 --roadmap WR-131`
  reports `write_implementation_contract`;
- roadmap, production, docs, planning, PUML, and whitespace checks pass;
- `task ai:goal -- --track PT-UI-DESIGNER-WB-V1-CLOSURE` is rerun before any
  implementation starts.

## Stop Conditions

Stop before implementation if:

- `WR-130` completion evidence is missing or invalid;
- the game.runtime proof needs concrete HUD runtime behavior;
- evidence packets would become source truth;
- performance baselines cannot be measured through product paths;
- a new durable game-runtime UI owner boundary is required without accepted
  design or ADR work.

## Closeout Requirements

A later PM-005 closeout must record:

- source-versioned scenario and evidence packet proof for editor.workbench and
  game.runtime target profiles;
- typed diagnostics for stale, unsupported, or mismatched evidence;
- measured frame build, canvas projection, catalog projection, diagnostics
  projection, and resize relayout baselines;
- explicit known gaps for PM-006 final closeout and downstream game HUD
  implementation.

## Perfectionist Closeout Audit

Expected PM-005 completion quality is `runtime_proven` if scenario/evidence
packets and performance baselines are measured through normal product paths.
It is not `perfectionist_verified` because final V1 product closeout and
downstream handoff remain PM-006.
