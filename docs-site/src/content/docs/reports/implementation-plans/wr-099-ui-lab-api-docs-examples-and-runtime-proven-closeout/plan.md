---
title: WR-099 UI Lab API Docs Examples And Runtime-Proven Closeout Contract
description: Promotion and implementation-readiness contract for PM-UI-LAB-007 focused public APIs, usage docs, examples, API ergonomics review, final runtime-proven closeout, and perfectionist-audit handoff.
status: active
owner: editor
layer: domain/app/docs
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
  - ../../../design/accepted/ui-lab-preview-lab-runtime-evidence-design.md
  - ../../../design/active/ui-lab-productization-design.md
  - ../../../reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-099 UI Lab API Docs Examples And Runtime-Proven Closeout Contract

## Goal

Implement `PM-UI-LAB-007` by making Editor Lab V1 discoverable and closeable as
a product: focused public workflow entry points for `ui_definition` and
`editor_definition`, usage docs, examples, public API ergonomics review, final
PT-UI-LAB runtime-proven closeout, and a separate perfectionist-audit intake.

This contract covers promotion readiness and the future bounded implementation
slice only. It must not implement product code until WR-099 is promoted by the
roadmap workflow and `task ai:goal -- --track PT-UI-LAB --scope non-deferred`
selects a legal implementation action.

## Source Of Truth

- Production milestone: `PM-UI-LAB-007` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-099` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Accepted PM007 design:
  `docs-site/src/content/docs/design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md`.
- Active productization design:
  `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`.
- Completed PM006 closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-006-preview-lab-and-runtime-evidence/closeout.md`.

Current implementation sources to inspect before product code changes:

- `domain/ui/ui_definition/src/lib.rs`
- `domain/ui/ui_definition/src/visual_layout/mod.rs`
- `domain/ui/ui_definition/src/preview_fixture/mod.rs`
- `domain/ui/ui_definition/src/persistence_activation/mod.rs`
- `domain/ui/ui_definition/src/production_readiness/mod.rs`
- `domain/editor/editor_definition/src/lib.rs`
- `domain/editor/editor_definition/src/document.rs`
- `domain/editor/editor_definition/src/operation.rs`
- `domain/editor/editor_definition/src/validate.rs`
- `apps/runenwerk_editor/src/shell/editor_lab_evidence/mod.rs`
- `docs-site/src/content/docs/domain/ui/README.md`
- `docs-site/src/content/docs/domain/editor/README.md`

## Promotion Readiness

`task production:plan -- --milestone PM-UI-LAB-007 --roadmap WR-099` reports
WR-099 as `write_promotion_contract`, with promotion preflight status
`promotable`.

Promotion is honest because:

- `PM-UI-LAB-006` is completed with runtime-proven Preview Lab evidence.
- `WR-098` is completed and archived as the direct prerequisite.
- `PM-UI-LAB-007` has an accepted API/docs/examples/runtime closeout design.
- `WR-099` has bounded write scopes and explicit non-goals for game-runtime UI
  projection and no-gap certification.
- No product code is required before promotion.

Use this evidence string for promotion:

```text
Accepted PM-UI-LAB-007 API/docs/examples/runtime closeout design plus completed PM-UI-LAB-006 runtime-proven Preview Lab evidence clear WR-099 for current-candidate implementation planning; ui_definition remains behavior-free, editor_definition remains runtime-neutral, and perfectionist certification remains a separate audit intake.
```

Suggested command after this contract validates:

```text
task roadmap:promote -- --id WR-099 --state current_candidate --evidence "Accepted PM-UI-LAB-007 API/docs/examples/runtime closeout design plus completed PM-UI-LAB-006 runtime-proven Preview Lab evidence clear WR-099 for current-candidate implementation planning; ui_definition remains behavior-free, editor_definition remains runtime-neutral, and perfectionist certification remains a separate audit intake."
```

Do not run product-code implementation before promotion and a subsequent
`task ai:goal` rerun select a legal implementation action.

## Architecture Decisions

Source-truth decisions:

- `domain/ui/ui_definition` owns behavior-free UI authoring, validation,
  normalization, visual layout operations, preview fixtures, persistence
  activation, production readiness, diagnostics, and focused public workflow
  exports.
- `domain/editor/editor_definition` owns runtime-neutral editor definition
  documents, validation, Editor Lab operations, diagnostics, and focused public
  workflow exports.
- `apps/runenwerk_editor` owns Editor Lab project IO, provider sessions, live
  activation, rollback, preview evidence execution, artifact storage, and app
  runtime closeout evidence.
- Docs and examples must use public APIs and may reference app-owned runtime
  evidence artifacts. They must not introduce hidden test-only shortcuts.

Forbidden shortcuts:

- moving project IO, activation execution, provider sessions, runtime scenario
  execution, artifact writing, screenshot capture, accessibility tooling, or
  performance runners into `domain/ui/ui_definition`;
- teaching private tests or internal-only helpers as the normal public path;
- claiming `perfectionist_verified`;
- closing PT-UI-LAB without a separate no-gap audit intake;
- using docs-only evidence for runtime claims;
- leaving normal users to discover workflow APIs through broad glob exports
  only.

## Implementation Scope

### Focused Public Entry Points

Add focused workflow modules or preludes for the normal public paths.
Suggested shape, subject to code inspection:

```text
domain/ui/ui_definition/src/
|-- prelude.rs
`-- workflow.rs

domain/editor/editor_definition/src/
|-- prelude.rs
`-- workflow.rs
```

The focused exports should cover the normal happy path first:

- author/validate/normalize UI templates;
- apply visual layout operations and inspect diffs/diagnostics;
- run preview fixture, persistence activation, and production readiness checks;
- construct editor definition documents;
- apply `EditorLabOperation` values and inspect reports/diffs;
- hand runtime-neutral documents to app-owned Editor Lab project/apply flows.

Existing broad exports may remain for compatibility, but docs and examples
must point at focused workflow/prelude exports first.

### Usage Docs

Add or update docs so normal users can find:

- UI definition authoring, validation, normalization, preview fixture,
  visual layout operation, persistence activation, diagnostics, and readiness
  workflows;
- editor definition document, validation, operation, diagnostics, and app handoff
  workflows;
- Editor Lab runtime evidence and final closeout evidence;
- ownership boundaries and what stays app-owned.

Suggested docs:

```text
docs-site/src/content/docs/domain/ui/ui-definition-usage.md
docs-site/src/content/docs/domain/editor/editor-definition-usage.md
docs-site/src/content/docs/reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/api-ergonomics-review.md
```

### Examples

Add examples that compile or run through public APIs. Suggested shape:

```text
domain/ui/ui_definition/examples/ui_definition_workflow.rs
domain/editor/editor_definition/examples/editor_definition_workflow.rs
```

Examples must avoid app-owned runtime shortcuts. If an example needs live
activation, project IO, provider sessions, or runtime evidence, it should link
to app-owned PM005/PM006 artifacts instead of moving that behavior into domain
crates.

### Final Runtime-Proven Closeout

Create the PM007 closeout only after public API, docs, examples, and ergonomics
review evidence exists. The closeout must aggregate PM001-PM006 evidence and
state remaining gaps truthfully.

Also create a separate roadmap or production intake for
`PT-UI-LAB-PERFECTION` or equivalent no-gap audit. That audit must remain
separate from this track's `runtime_proven` closeout.

## Implementation Steps

1. Re-inspect current `ui_definition` and `editor_definition` public exports,
   docs, and tests before editing.
2. Add focused workflow/prelude modules with conservative exports.
3. Add tests that prove normal imports compile and support the intended
   workflow.
4. Add public examples for UI definition and editor definition workflows.
5. Add usage docs and link them from owning docs indexes.
6. Write a public API ergonomics review artifact covering `lib.rs`, focused
   exports, docs, examples, and closeout links.
7. Create the final PM007 closeout and aggregate PM001-PM006 evidence.
8. Create a separate perfectionist-audit intake.
9. Move WR088 to completed archive state and mark PM007 plus PT-UI-LAB
   runtime-proven only after validation passes.

## Acceptance Criteria

- Normal `ui_definition` workflows have focused public imports and docs.
- Normal `editor_definition` workflows have focused public imports and docs.
- Examples compile or run through public APIs.
- Public API ergonomics review is completed and linked from closeout evidence.
- Final PT-UI-LAB closeout links PM001-PM006 and PM007 evidence and truthfully
  lists unsupported/native/perfectionist gaps.
- A separate perfectionist/no-gap audit intake exists.
- `ui_definition` remains behavior-free.
- `editor_definition` remains runtime-neutral.
- Runtime evidence execution remains app-owned.

## Validation

Minimum validation before WR088 closeout:

```text
cargo fmt
cargo test -p ui_definition
cargo test -p editor_definition
cargo test -p runenwerk_editor editor_lab
cargo test -p runenwerk_editor pm_ui_lab_006
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
task ai:goal -- --track PT-UI-LAB --scope non-deferred
```

Focused tests should prove:

- focused `ui_definition` imports support authoring, validation, visual layout,
  persistence activation, readiness, and diagnostics workflows;
- focused `editor_definition` imports support document construction,
  validation, Editor Lab operation application, and diagnostics workflows;
- examples compile or run under Cargo;
- docs and examples do not reference test-only shortcuts as the normal path.

## Closeout Requirements

- Create
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md`
  only after public API, docs, examples, ergonomics review, and final
  validation pass.
- Store the API ergonomics review under that closeout directory.
- Add closeout paths to WR088 write scopes only when the files exist.
- Move WR088 to `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
  and mark PM-UI-LAB-007 completed only after evidence supports
  `runtime_proven`.
- Mark PT-UI-LAB completed only after `task ai:goal -- --track PT-UI-LAB
  --scope non-deferred` shows no remaining non-deferred incomplete milestones.
- Preserve unsupported native screenshot/GPU visual-diff/accessibility timing
  gaps as known gaps, not hidden passes.

## Perfectionist Closeout Audit

WR088 may close PM007 and PT-UI-LAB as `runtime_proven` if public API
ergonomics, docs, examples, runtime evidence aggregation, and final closeout
evidence are complete. It must not claim `perfectionist_verified`.

The implementation must create a separate no-gap audit intake for
`PT-UI-LAB-PERFECTION` or an equivalent track. That later audit owns:

- module-structure perfectionist review;
- deeper UI ergonomics audit;
- native screenshot and visual diff platform decisions;
- accessibility and performance depth beyond current retained evidence;
- game-runtime UI projection execution;
- zero-known-gap certification.

## Non-Goals

- No game-runtime UI projection execution.
- No no-gap or `perfectionist_verified` claim.
- No moving app runtime behavior into `ui_definition` or `editor_definition`.
- No broad rewrite of all public APIs before focused normal workflow exports
  are in place.
- No native screenshot/GPU visual-diff platform implementation unless it falls
  inside a separately accepted design and WR.

## Stop Conditions

Stop implementation if:

- focused public workflow exports would require app/runtime behavior in domain
  crates;
- examples cannot compile using public APIs;
- docs and examples disagree with ownership boundaries;
- final closeout cannot link concrete runtime evidence from PM001-PM006;
- a separate perfectionist-audit intake cannot be created;
- a durable dependency-direction change is needed without a new accepted ADR or
  design.
