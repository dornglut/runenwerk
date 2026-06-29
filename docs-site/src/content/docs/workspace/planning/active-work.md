---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-29
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-implementation-scope.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-012A`

Title: UI Component Platform Executable Interaction Story

State: implementation scope accepted / active implementation

Lifecycle state: `active-implementation`

Owner: `ui_story` owns the executable interaction workflow profile and proof-envelope nodes only. `ui_runtime` owns interaction story session execution mechanics, replay/live application, reports, visual proof, parity reports, and proof-frame projection. `ui_input` owns normalized input facts and any minimal event-to-sample helpers. `ui_static_mount` owns static `UiFrame` validation. `runenwerk_editor` owns the narrow proof-host adapter from `UiInputEvent` to runtime session evidence. `ui_controls` remains the owner of reusable interaction descriptors and catalog/inspection declarations. Product/editor/app mutation, overlay behavior, and full text editing remain out of scope.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/documentation-structure.md`, `docs-site/src/content/docs/workspace/authority-model.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/planning/README.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-implementation-scope.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, and `docs-site/src/content/docs/design/active/ui-component-platform-story-proof-envelope-design.md`.

Write scope: Implement only the files named in `ui-component-platform-executable-interaction-story-implementation-scope.md`: `domain/ui/ui_story/src/workflow/builtin.rs`, `domain/ui/ui_story/src/workflow/mod.rs`, `domain/ui/ui_story/src/lib.rs`, `domain/ui/ui_runtime/Cargo.toml`, `domain/ui/ui_runtime/src/input/generic_interaction.rs`, `domain/ui/ui_runtime/src/input/generic_interaction_fixture.rs`, `domain/ui/ui_runtime/src/input/generic_interaction_visual_frame.rs`, `domain/ui/ui_runtime/src/input/interaction_story_session.rs`, `domain/ui/ui_runtime/src/input/mod.rs`, `domain/ui/ui_runtime/tests/interaction_replay_report.rs`, `domain/ui/ui_runtime/tests/executable_interaction_story.rs`, `domain/ui/ui_input/src/facts.rs`, `domain/ui/ui_input/src/event.rs`, `domain/ui/ui_input/src/lib.rs`, `domain/ui/ui_input/tests/input_normalized_facts.rs`, `domain/ui/ui_static_mount/src/lib.rs`, `domain/ui/ui_static_mount/tests/phase12_generic_interaction_static_mount.rs`, `domain/ui/ui_static_mount/tests/phase12_executable_interaction_story_static_mount.rs`, `apps/runenwerk_editor/Cargo.toml`, `apps/runenwerk_editor/src/editor_features/mod.rs`, `apps/runenwerk_editor/src/editor_features/executable_interaction_story_proof.rs`, and `apps/runenwerk_editor/tests/phase12_executable_interaction_story_proof_host.rs`.

Validation expectation: Run and record focused validation from the implementation scope: `cargo fmt --all --check`, `cargo check -p ui_story`, `cargo check -p ui_controls`, `cargo check -p ui_input`, `cargo check -p ui_runtime`, `cargo check -p ui_static_mount`, `cargo check -p runenwerk_editor`, `cargo test -p ui_story executable_interaction_workflow`, `cargo test -p ui_controls control_interaction`, `cargo test -p ui_input input`, `cargo test -p ui_runtime interaction`, `cargo test -p ui_runtime executable_interaction_story`, `cargo test -p ui_runtime --test interaction_replay_report`, `cargo test -p ui_static_mount phase12_executable_interaction_story`, `cargo test -p runenwerk_editor phase12_executable_interaction_story_proof_host`, `python3 tools/docs/validate_docs.py`, and `git diff --check`. If exact test names differ, record the actual mapping in the PR body and closeout.

Known blockers: None for starting the scoped implementation. Product-facing editor window/gallery exposure is not guaranteed by this slice unless it can be added without editor shell surface registry changes. If shell surface registry changes become necessary, stop and record a scope revision first.

Next action: Implement the scoped Tier 5 executable generic interaction story. Start with the shared runtime session path so batch replay and live apply use the same internal fact application code after `NormalizedInputSample`. Then add the `ui_story` workflow profile, static mount coverage, and narrow editor proof-host adapter.

Evidence: User accepted the Tier 5 design on 2026-06-29. The implementation scope records exact owner files/crates, host adapter location, runtime session API scope, validation envelope, evidence artifacts, manual validation expectation, and stop conditions. PR #43 already provides the lower-tier assets this implementation must reuse: package-backed interaction descriptors, catalog/inspection projection, normalized input facts, descriptor-driven replay/report, `InteractionVisualProof`, `InteractionProofRenderFrame`, and `UiStaticMountReport::from_frame`.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, and stop conditions are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Write scope:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
