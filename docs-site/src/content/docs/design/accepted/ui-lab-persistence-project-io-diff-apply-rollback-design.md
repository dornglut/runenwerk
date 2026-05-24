---
title: UI Lab Persistence Project IO Diff Apply Rollback Design
description: Accepted design for PM-UI-LAB-005 app-owned Editor Lab project IO, diff/apply review, activation reports, failed activation preservation, and rollback.
status: accepted
owner: editor
layer: app/editor-definition
canonical: true
last_reviewed: 2026-05-24
related_adrs:
  - ../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../adr/accepted/0012-capability-workbench-clean-break.md
related_designs:
  - ../active/ui-lab-productization-design.md
  - ./ui-lab-command-catalog-and-surface-registry-design.md
  - ./ui-lab-app-hosted-editor-lab-surface-shell-design.md
  - ./ui-lab-operation-driven-visual-authoring-design.md
  - ./ui-designer-persistence-migration-diff-and-activation-design.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# UI Lab Persistence Project IO Diff Apply Rollback Design

## Status

This is the accepted design gate for `PM-UI-LAB-005`.

It authorizes roadmap intake and implementation-contract preparation for the
Editor Lab persistence, project IO, diff/apply, activation-report, failed
activation preservation, and rollback product slice. It does not authorize
product-code changes until a linked WR row exists, `task production:plan`
passes for that row, and the row names write scopes, validation, and runtime
evidence.

`PM-UI-DESIGN-009` remains the accepted generic UI persistence contract. This
design consumes that contract for Editor Lab productization, but does not move
project storage, live activation, provider sessions, or app recovery behavior
into `domain/ui/ui_definition`.

## Goal

Editor Lab V1 must let authors move from an edited draft definition to a
recoverable app-hosted project state:

```text
draft editor definition documents
  -> app-owned Editor Lab project package
  -> save/load/import/export/migration
  -> deterministic definition diff
  -> user-facing apply review
  -> activation preflight
  -> live app activation attempt
  -> activation report
  -> applied snapshot or failed activation preservation
  -> rollback or reload last applied state
```

The product outcome is not just serialized RON or a status line. The runtime
must prove that a changed lab definition can be saved, reloaded, diffed,
applied, rejected, preserved after failed activation, and rolled back through
Editor Lab flows.

## Code-Truth Reconciliation

Current code has useful partial paths, but none of them satisfy the PM005
product contract end to end:

- `apps/runenwerk_editor/src/shell/self_authoring.rs` owns
  `EditorDefinitionExportPackage`, `export_selected_to_ron`,
  `export_selected_package`, `import_versioned_ui_template_document`,
  `build_apply_preview`, `apply_selected`, and `rollback_selected`. These are
  in-memory selected-document flows, not a project document store with package
  sessions, migration reports, reload-last-applied behavior, failed activation
  preservation, or user-facing apply review.
- `domain/ui/ui_definition/src/persistence_activation/mod.rs` owns
  runtime-neutral UI persistence descriptors and validation diagnostics. It is
  reusable for generic UI definition validation, but it intentionally does not
  own concrete file IO, editor project sessions, provider behavior, or live app
  activation.
- `apps/runenwerk_editor/src/editor_app/state.rs` has
  `pending_editor_definition_activations`, but this is only a queue of
  documents. It does not retain activation reports, failed activation inputs,
  rollback records, or project package provenance.
- `apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs`
  maps an editor definition document to live activation intents. The result is
  consumed by `apps/runenwerk_editor/src/runtime/resources.rs`, which appends
  console lines and preserves prior live state through specific installer
  failures. The activation attempt is not yet represented as a durable typed
  review artifact.
- `PM-UI-LAB-004` added typed operation reports and app-owned operation
  history. PM005 may persist or summarize operation outcomes in an Editor Lab
  package, but operation history is not allowed to become the only source of
  document truth.

## Architecture Governance Result

Architecture governance accepts the PM005 direction with these boundaries:

- DDD bounded context owner: `editor`.
- App boundary owner: `apps/runenwerk_editor` owns concrete project paths,
  document-store sessions, save/load/import/export orchestration, activation
  execution, failed activation preservation, rollback, and runtime evidence.
- Domain owner: `domain/editor/editor_definition` may own runtime-neutral
  editor definition package, diff, apply-review, diagnostic, and rollback
  metadata types when those types do not know concrete paths, provider
  sessions, runtime handles, or app execution.
- Generic UI owner: `domain/ui/ui_definition` remains behavior-free and owns
  only generic UI persistence, migration, diff, activation-preflight
  descriptors, and diagnostics.
- Translation boundary: app providers translate UI actions into editor
  definition operations and project-store requests; app project IO translates
  persisted packages into domain validation inputs; runtime activation
  translates accepted editor definitions into live app changes and typed
  reports.
- Clean Architecture direction: `ui_definition` must not import editor app,
  shell provider, project IO, renderer, runtime, or filesystem concerns.
  `editor_definition` must not depend on `apps/runenwerk_editor`. App code may
  consume both domains and produce app-owned reports.
- ADR need: no new ADR is required while PM005 preserves these boundaries. A
  new ADR or accepted design update is required before concrete project IO,
  live activation, provider sessions, or runtime rollback move into a domain
  crate.
- ATAM-lite priority order: source-truth correctness and recoverability first;
  ownership boundaries second; deterministic review and diagnostics third;
  author ergonomics fourth; compatibility and migration fifth; performance and
  screenshot breadth remain PM006.
- Team Topologies label: stream-aligned editor product work with
  complicated-subsystem support from UI definition and editor definition
  owners.
- Next action: add bounded WR roadmap rows for project-store/package work and
  apply-review/activation/rollback work before implementation.

## Contracts

### EditorLabProjectPackage

`EditorLabProjectPackage` is a versioned app-facing package for Editor Lab
state. It contains:

- package id, version, kind, created/updated provenance, and source profile;
- editor definition documents as the canonical saved draft source;
- optional last-applied snapshots by document id;
- migration metadata and deterministic migration-preview references;
- operation report summaries when useful for review, never as the only source
  of document truth;
- package-level diagnostics and source-map-capable package paths.

The package must serialize deterministically enough for stable diffs and
review. Concrete file paths and filesystem errors are app-owned store details,
not domain truth.

### EditorLabDocumentStore

`EditorLabDocumentStore` is the app-owned project IO boundary. It owns:

- creating an Editor Lab package from the current self-authoring state;
- loading a package into draft documents without mutating live activation;
- saving the active package atomically where the current runtime supports it;
- importing and exporting single definitions and whole packages;
- running migration preflight against package contents;
- reloading the last saved package and the last applied snapshot;
- preserving failed load, save, migration, and activation inputs for review.

The store reports typed results instead of success-shaped console strings.
App UI may render those results, but `ui_definition` must not execute store
behavior.

### DefinitionApplyReview

`DefinitionApplyReview` is the typed user-facing review contract for a proposed
apply. It contains:

- selected document id, before draft, after draft, and last applied snapshot
  references;
- deterministic document diff rows and optional textual diff output;
- operation reports that contributed to the proposal when available;
- migration and validation diagnostics from editor and UI definition domains;
- activation preflight status and target profile;
- activation attempt report once the app executes the apply;
- rejected-apply reason when the user cancels or validation blocks activation;
- rollback metadata sufficient to restore the prior applied snapshot.

The review contract may have a runtime-neutral core in
`domain/editor/editor_definition`, but the concrete activation attempt and
filesystem/project provenance remain app-owned.

### EditorDefinitionActivationReport

`EditorDefinitionActivationReport` is app-owned evidence for a live activation
attempt. It records:

- activation id, document id, document kind, target profile, and source package;
- activation status: queued, applied, rejected, failed, no-live-activation, or
  degraded-provider;
- diagnostics from validation, domain activation mapping, registry installers,
  workspace formation, and runtime/app adapters;
- whether previous live state was preserved;
- the applied snapshot id or failed activation snapshot id;
- console summary lines only as derived display text.

Activation reports are observations. They do not replace the document package
or become source truth.

### EditorLabRollbackRecord

`EditorLabRollbackRecord` is app-owned recovery metadata. It records:

- rollback id and source apply-review id;
- document id and prior applied snapshot;
- reason and initiating command;
- preflight diagnostics;
- rollback status and diagnostics;
- resulting active/applied document state.

Rollback must fail closed when no prior applied snapshot exists.

## Runtime Workflow

PM005 implementation rows must converge on these runtime workflows:

1. Save current Editor Lab drafts into an `EditorLabProjectPackage`.
2. Reload the saved package into the lab without changing live editor state.
3. Import/export selected definitions and whole packages with typed parse,
   schema, migration, and validation diagnostics.
4. Build a `DefinitionApplyReview` from draft versus last applied state.
5. Reject a review without mutating applied state.
6. Apply an accepted review by queueing and executing app-owned live activation.
7. Preserve the failed activation input, diagnostics, and prior live state when
   activation fails.
8. Roll back an applied definition to the previous applied snapshot.
9. Reload last applied state after app/session reconstruction where the current
   runtime supports it.

## Implementation Row Candidates

Roadmap intake should split PM005 into bounded rows unless review decides a
single row is safer:

| Candidate | Primary scope | Runtime evidence |
|---|---|---|
| Editor Lab project package and document store | `apps/runenwerk_editor/src/shell` or a new app-owned Editor Lab project IO module, plus editor-definition package DTOs if runtime-neutral | Save, reload, import, export, migration preflight, parse failure, invalid package preservation, and deterministic package diff artifacts. |
| Definition apply review, activation report, and rollback | `apps/runenwerk_editor/src/shell`, `apps/runenwerk_editor/src/editor_app`, `apps/runenwerk_editor/src/runtime`, and runtime-neutral review DTOs if needed | Review, reject, apply, activation success, activation failure preservation, previous-state preservation, rollback, and reload-last-applied artifacts. |

Both rows must keep write scopes disjoint from PM006 screenshot/accessibility
evidence breadth and PM007 public API/docs/examples closeout.

## Required Fitness Functions

PM005 implementation rows must add focused validation for:

- deterministic package serialization and reload round-trip;
- import/export of a selected definition and full package;
- unsupported package version and invalid schema diagnostics;
- migration dry-run diagnostics before activation;
- deterministic diff rows for draft versus applied documents;
- rejected apply preserving draft and applied state;
- successful apply producing an activation report and applied snapshot;
- failed activation preserving the failed input and previous active state;
- rollback requiring a prior applied snapshot;
- rollback restoring the previous applied snapshot and producing diagnostics;
- reload-last-applied behavior where the runtime supports it;
- command/catalog and provider UI paths exposing typed diagnostics rather than
  generic unavailable messages.

## Non-Goals

PM005 does not:

- implement PM006 screenshot matrices, accessibility checks, performance
  evidence, or broad degraded-provider scenario coverage;
- implement PM007 public API ergonomics review, usage guides, examples, or
  final runtime-proven track closeout;
- move project file IO, activation execution, provider sessions, renderer
  resources, or app runtime rollback into `domain/ui/ui_definition`;
- claim game-runtime UI projection execution;
- claim `perfectionist_verified` or no-gap completion;
- treat console lines, retained previews, or descriptors alone as runtime
  evidence.

## Acceptance Bar

PM005 can move from design to implementation planning when:

- this accepted design exists and is linked from the production milestone;
- the active UI Lab productization design references this PM005 design;
- roadmap intake creates bounded WR rows or one explicitly bounded full-slice
  WR with write scopes, dependencies, validation, and closeout evidence;
- `task docs:validate`, `task roadmap:render`, `task roadmap:validate`,
  `task roadmap:check`, `task production:render`,
  `task production:validate`, `task production:check`, and
  `task planning:validate` pass after metadata edits.

PM005 can be closed only when closeout evidence proves save, reload, import,
export, migration, diff, apply, reject, failed activation preservation, and
rollback in the app-hosted Editor Lab.
