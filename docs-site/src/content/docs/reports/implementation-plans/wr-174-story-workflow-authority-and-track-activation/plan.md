---
title: Story Workflow Authority And Track Activation Implementation Plan
status: active
type: implementation-plan
wr: WR-174
milestone: PM-UI-STORY-001
---

# Story Workflow Authority And Track Activation

Quality doctrine: runenwerk-quality-doctrine-v1.

This is the bounded design-first implementation contract for `WR-174` /
`PM-UI-STORY-001`. It activates `PT-UI-STORY-PLATFORM` as planning and
sequencing authority for the UI story proof substrate only. It does not
authorize product code, crate creation, runtime rendering, gallery migration,
static mount, product mount eligibility, reusable component maturity,
Designer/Workbench product authoring, screen-space game HUD behavior, or
world-space/entity-attached UI.

Executable authority lives in `plan.contract.yaml`. This prose file records
the human-readable decisions that make the sidecar safe to execute.

## Goal

Establish the story-first production authority, sequencing rules, deferred
standalone-rendering decision, future WR expansion policy, bounded ownership
split, and no-runtime-code stop conditions for `PT-UI-STORY-PLATFORM`.

The completed slice must prove that the track is a story proof-substrate track,
not a general UI feature bucket. The track can coordinate future `UiStory`
contracts only after each future milestone receives its own owning WR and
accepted plan.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-story-platform.yaml`
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`
- `docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md`
- `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md`
- `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`
- `docs-site/src/content/docs/domain/ui/roadmap.md`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-program-arch-014-architecture-behavior-truth-and-extension-readiness-closure/closeout.md`

`UiStoryRunReport` is recorded as the future proof envelope. Until the later
story runner/report milestones are implemented, the current slice may only
record governance authority and evidence that the downstream work remains
blocked behind explicit WRs.

## Readiness

`WR-174` is intentionally `blocked_deferred` at B4 until this design-first
governance proof writes evidence and closeout. The row is linked to
`PM-UI-STORY-001`, and the manifest now owns `WR-174`.

The linked production dependency `PM-UI-PROGRAM-ARCH-014` is represented by the
completed closeout path in the manifest design dependencies. If production
validation reports a dependency-name mismatch, repair only the exact
production/manifest metadata that names that dependency; do not infer runtime
authority from this plan.

## Owners

- Workspace governance owns production sequencing, roadmap metadata, manifest
  authority, generated planning docs, and closeout evidence for this slice.
- Domain UI owns the future story proof-substrate contracts named by later
  milestones.
- Renderer, ECS, editor, game, Designer/Workbench, component platform, and
  world-space UI owners remain consumers or downstream owners only. They do not
  own story semantics in this slice.

## Implementation Scope

Allowed work is limited to:

- verify the active story-driven workflow and UI platform roadmap still encode
  the bounded ownership split;
- keep `PT-UI-STORY-PLATFORM` planning-and-sequencing only for
  `PM-UI-STORY-001`;
- preserve `WR-173` / standalone runtime rendering as deferred superseded work;
- keep `PM-UI-STORY-002` through `PM-UI-STORY-005` as future milestones with
  their own WR candidates;
- write governance evidence for `PM-UI-STORY-001`;
- close `PM-UI-STORY-001` as `bounded_contract` only after validation passes.

No Rust, app, engine, renderer, ECS, editor, game, or product behavior files are
in scope.

## Non-Goals

- no `domain/ui/ui_story` crate creation;
- no `UiStoryManifest`, `UiStoryRegistry`, `UiStoryRunner`,
  `UiStoryRunReport`, or `UiStoryMountEligibility` implementation;
- no gallery or CLI migration;
- no runtime rendering proof;
- no static mount or product mount eligibility;
- no reusable component maturity;
- no Designer/Workbench product authoring UX;
- no screen-space game HUD behavior;
- no world-space, entity-attached, projected attachment, or gameplay/world
  binding UI;
- no renderer-, ECS-, editor-, or game-owned UI semantics.

## Steps

1. Run the accepted sidecar through the Track Execution Harness.
2. Produce `artifact-governance.yaml` evidence for the active design docs and
   domain UI roadmap subjects.
3. Run the sidecar validation commands.
4. Rerun `task ai:goal -- --track PT-UI-STORY-PLATFORM --scope non-deferred`.
5. Close only `PM-UI-STORY-001` when evidence exists and validation remains
   clean.
6. Stop before expanding `PM-UI-STORY-002`.

## Validation

- `task production:render`
- `task production:validate`
- `task production:check`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task planning:validate`

Workflow-tooling changes made to support this contract must also pass:

- `uv run pytest tools/workflow/tests/test_execution_contracts.py -k planning_expansion`
- `task truth:certify -- --track PT-TRACK-EXECUTION-HARNESS --claim track-execution-harness-authority`

## Acceptance Criteria

- `WR-174` remains the owning WR for `PM-UI-STORY-001`.
- The manifest and production track agree that the slice is
  docs-governance-only.
- The governance evidence file exists and references the story workflow design,
  UI platform roadmap, runtime rendering roadmap, and domain UI roadmap.
- No product code or runtime behavior changes are made.
- Downstream story milestones remain incomplete and explicitly blocked behind
  future WR expansion or accepted plans.
- All listed production, roadmap, docs, and planning validation commands pass.

## Stop Conditions

- stop after planning metadata, manifest, and docs authority updates;
- stop before runtime code;
- stop before crate creation;
- stop before gallery migration;
- stop before product mount eligibility;
- stop before reusable component maturity;
- stop before Designer/Workbench product authoring;
- stop before game HUD behavior;
- stop before world-space or entity-attached UI;
- stop if validation fails;
- stop if generated docs require hand edits;
- stop if any source file changes enough that `task ai:goal` must be rerun.

## Closeout Requirements

Close `PM-UI-STORY-001` as `bounded_contract`, not
`perfectionist_verified`. The closeout must name:

- `WR-174`;
- the governance evidence file;
- every validation command and result;
- the exact downstream known gaps: `PM-UI-STORY-002` through
  `PM-UI-STORY-005`, component maturity, Designer/Workbench product work,
  screen-space game HUD behavior, and world-space/entity-attached UI;
- the next legal action after closeout: expand or link `PM-UI-STORY-002`
  before any `domain/ui/ui_story` crate creation.

## Perfectionist Closeout Audit

This slice cannot honestly claim `perfectionist_verified`. The track-level
perfectionist claim remains blocked until every story proof-substrate milestone
is complete, current truth evidence exists, no bypass path around
`UiStoryRunReport` remains, and the final no-gap audit in `PM-UI-STORY-005`
passes.
