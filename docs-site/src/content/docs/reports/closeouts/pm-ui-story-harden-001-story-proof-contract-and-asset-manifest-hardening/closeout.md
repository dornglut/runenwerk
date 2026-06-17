---
title: PM-UI-STORY-HARDEN-001 Story Proof Contract And Asset Manifest Hardening Closeout
status: completed
closeout_evidence:
  milestone_id: PM-UI-STORY-HARDEN-001
  wr_id: WR-178
  completion_quality: runtime_proven
  evidence_categories:
    - runtime_test
    - manifest
    - diagnostics
    - gallery_no_bypass
    - governance
  validation_commands:
    - cargo fmt --check
    - cargo check --workspace
    - cargo test -p ui_story story_manifest
    - cargo test -p ui_story expected_failure
    - cargo test -p ui_story proof_contract
    - cargo test -p ui_story
    - cargo test -p runenwerk_editor story
    - task production:render
    - task production:validate
    - task production:check
    - task roadmap:render
    - task roadmap:validate
    - task roadmap:check
    - task docs:validate
    - task planning:validate
    - task ai:goal -- --track PT-UI-STORY-PLATFORM --scope non-deferred
  validation_results:
    - 'cargo:fmt (cargo fmt --check) -> exit 0'
    - 'cargo:check (cargo check --workspace) -> exit 0'
    - 'cargo:test (cargo test -p ui_story story_manifest) -> exit 0'
    - 'cargo:test (cargo test -p ui_story expected_failure) -> exit 0'
    - 'cargo:test (cargo test -p ui_story proof_contract) -> exit 0'
    - 'cargo:test (cargo test -p ui_story) -> exit 0'
    - 'cargo:test (cargo test -p runenwerk_editor story) -> exit 0'
    - 'task:production:render (task production:render) -> exit 0'
    - 'task:roadmap:render (task roadmap:render) -> exit 0'
    - 'task:roadmap:validate (task roadmap:validate) -> exit 0'
    - 'task:roadmap:check (task roadmap:check) -> exit 0'
    - 'task:truth:certify (PT-UI-PROGRAM-ARCHITECTURE ui-program-architecture-implementation) -> exit 0'
    - 'task:truth:certify (PT-UI-PROGRAM-ARCHITECTURE retained-ui-compatibility) -> exit 0'
    - 'task:truth:certify (PT-UI-PROGRAM-ARCHITECTURE ui-program-perfectionist-conformance) -> exit 0'
    - 'task:production:complete-track-contracts (PT-UI-PROGRAM, PT-TRACK-EXECUTION-HARNESS, PT-UI-PROGRAM-ARCHITECTURE, PT-UI-STORY-PLATFORM) -> exit 0'
    - 'task:production:validate (task production:validate) -> exit 0'
    - 'task:production:check (task production:check) -> exit 0'
    - 'task:docs:validate (task docs:validate) -> exit 0'
    - 'task:planning:validate (task planning:validate) -> exit 0'
    - 'task:ai:goal (task ai:goal -- --track PT-UI-STORY-PLATFORM --scope non-deferred) -> exit 0'
  files_changed:
    - assets/ui_gallery/stories/controls/button/basic.story.ron
    - assets/ui_gallery/stories/controls/button/selected.story.ron
    - assets/ui_gallery/stories/controls/button/missing_source.failure.story.ron
    - apps/runenwerk_editor/src/bin/runenwerk_ui_gallery.rs
    - apps/runenwerk_editor/src/runtime/ui_gallery.rs
    - domain/ui/ui_story/Cargo.toml
    - domain/ui/ui_story/src/gallery.rs
    - domain/ui/ui_story/src/lib.rs
    - domain/ui/ui_story/src/manifest.rs
    - domain/ui/ui_story/src/proof.rs
    - domain/ui/ui_story/src/registry.rs
    - domain/ui/ui_story/src/report.rs
    - domain/ui/ui_story/src/runner.rs
    - docs-site/src/content/docs/reports/implementation-plans/wr-178-story-proof-contract-and-asset-manifest-hardening/plan.md
    - docs-site/src/content/docs/reports/closeouts/pm-ui-story-harden-001-story-proof-contract-and-asset-manifest-hardening/closeout.md
    - docs-site/src/content/docs/workspace/production-tracks.yaml
    - docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-story-platform.yaml
    - docs-site/src/content/docs/workspace/roadmap-items.yaml
    - docs-site/src/content/docs/workspace/production-track-index.md
    - docs-site/src/content/docs/workspace/production-milestone-register.md
    - docs-site/src/content/docs/workspace/diagrams/production-track-roadmap.puml
    - docs-site/src/content/docs/workspace/diagrams/production-track-full-roadmap.puml
    - docs-site/src/content/docs/workspace/roadmap-decision-register.md
    - docs-site/src/content/docs/workspace/design-implementation-triage.md
    - docs-site/src/content/docs/workspace/diagrams/value-weighted-dependency-roadmap.puml
  known_gaps:
    - ui_definition splitting, ui_surface semantic cleanup, render primitive generic registry redesign, physical folder moves, reusable component maturity, Designer/Workbench product authoring, game HUD behavior, and world-space/entity-attached UI remain out of WR-178 scope unless a future validation proves a direct story-proof bypass.
  closeout_path: docs-site/src/content/docs/reports/closeouts/pm-ui-story-harden-001-story-proof-contract-and-asset-manifest-hardening/closeout.md
  produced_at: 2026-06-17T00:00:00Z
---

# PM-UI-STORY-HARDEN-001 Story Proof Contract And Asset Manifest Hardening Closeout

WR-178 completed the prerequisite story proof-substrate hardening before
`PM-UI-STORY-005`.

The implementation keeps `UiStoryStageKind` as the compatibility report view
while adding machine-readable proof producers, proof keys, proof requirements,
proof evidence, and exact diagnostic expectations. Checked-in gallery metadata
now lives in `.story.ron` manifests under `assets/ui_gallery/stories` and points
at the existing source assets under `assets/ui_gallery/button`.

Expected-failure story reports now pass only when every declared diagnostic is
present with matching stage, proof key, code, and severity, and when no
unexpected `Error` diagnostic is present. Expected-failure stories remain mount
ineligible even when their declared failure produces a passing story verdict.

The editor gallery adapter consumes registry manifests and derives preview
publication from `UiStoryRunReport` plus `UiStoryMountEligibility`. A regression
test verifies that a failed story report carrying mounted-frame data still
publishes no gallery frame and contributes no button output.
