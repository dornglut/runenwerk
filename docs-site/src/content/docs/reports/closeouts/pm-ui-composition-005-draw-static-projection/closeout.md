---
title: PM-UI-COMPOSITION-005 Runtime Closeout
status: completed
closeout_evidence:
  milestone_id: PM-UI-COMPOSITION-005
  wr_id: WR-184
  completion_quality: runtime_proven
  evidence_categories:
    - runtime_test
    - fixture
    - diagnostics
  validation_commands:
    - cargo fmt --all --check
    - cargo test -p runenwerk_draw --test app_shell
    - cargo test -p runenwerk_draw --test composition_architecture_guards
    - cargo test -p runenwerk_draw composition
    - task ui:dependencies
    - task production:render
    - task production:validate
    - task production:check
    - task roadmap:render
    - task roadmap:validate
    - task roadmap:check
    - task docs:validate
    - task planning:validate
    - cargo clippy -p runenwerk_draw --all-targets --all-features --no-deps -- -D warnings
    - ./quiet_full_gate.sh
  validation_results:
    - 'cargo:fmt (cargo fmt --all --check) -> exit 0'
    - 'cargo:test (cargo test -p runenwerk_draw --test app_shell) -> exit 0'
    - 'cargo:test (cargo test -p runenwerk_draw --test composition_architecture_guards) -> exit 0'
    - 'cargo:test (cargo test -p runenwerk_draw composition) -> exit 0'
    - 'task:ui:dependencies (task ui:dependencies) -> exit 0'
    - 'task:docs:validate (task docs:validate) -> exit 0'
    - 'task:planning:validate (task planning:validate) -> exit 0'
    - 'cargo:clippy-draw (cargo clippy -p runenwerk_draw --all-targets --all-features --no-deps -- -D warnings) -> exit 0'
    - 'quiet-full-gate (./quiet_full_gate.sh) -> exit 1: pre-existing ui_composition large_enum_variant and ui_artifacts too_many_arguments/collapsible_if Clippy errors'
  files_changed:
    - apps/runenwerk_draw/Cargo.toml
    - apps/runenwerk_draw/src/app/composition/mod.rs
    - apps/runenwerk_draw/src/app/composition/content.rs
    - apps/runenwerk_draw/src/app/composition/definition.rs
    - apps/runenwerk_draw/src/app/composition/diagnostic.rs
    - apps/runenwerk_draw/src/app/composition/extension.rs
    - apps/runenwerk_draw/src/app/composition/projection.rs
    - apps/runenwerk_draw/src/app/composition/runtime.rs
    - apps/runenwerk_draw/src/app/mod.rs
    - apps/runenwerk_draw/src/app/presentation.rs
    - apps/runenwerk_draw/src/app/state.rs
    - apps/runenwerk_draw/src/app/workspace.rs
    - apps/runenwerk_draw/src/runtime/ink.rs
    - apps/runenwerk_draw/tests/app_shell.rs
    - apps/runenwerk_draw/tests/composition_architecture_guards.rs
    - docs-site/src/content/docs/apps/runenwerk-draw/README.md
    - docs-site/src/content/docs/apps/runenwerk-draw/composition-layouts.md
    - docs-site/src/content/docs/apps/runenwerk-draw/roadmap.md
    - docs-site/src/content/docs/reports/implementation-plans/wr-184-draw-static-composition-projection/plan.md
    - docs-site/src/content/docs/reports/implementation-plans/wr-184-draw-static-composition-projection/plan.contract.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-005/runtime_test-draw-static-composition-projection.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-005/fixture-draw-definition-extension-authority.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-005/diagnostics-draw-content-liveness.yaml
  known_gaps:
    - Adaptive Draw reflow, narrow-target drawers, and Region Compass integration remain later governed checkpoints.
    - The clean-cutover branch cannot merge before the legacy cleanup and final perfectionist closeout gates pass.
    - Repository-wide Clippy remains blocked by pre-existing ui_composition and ui_artifacts warnings; WR-189 can own ui_composition cleanup, while ui_artifacts needs an owning-track fix or explicit final-gate scope amendment before merge.
  closeout_path: docs-site/src/content/docs/reports/closeouts/pm-ui-composition-005-draw-static-projection/closeout.md
  produced_at: 2026-06-20T07:37:13Z
---

# PM-UI-COMPOSITION-005 Runtime Closeout

Runenwerk Draw is now a direct non-editor runtime consumer of
`ui_composition`. Its top bar, tool rail, canvas, and support panel derive from
one ratified region graph and a deterministic Draw extension. The previous
`DrawingWorkspaceProjection`, `workspace()` API, independent narrow-width hide
rule, and `app/workspace.rs` authority are deleted.

`DrawingDocument`, drawing revision, stroke transactions, ink products, and
drawing history remain app/domain-owned. Seven content-liveness states preserve
the structural graph and use the app projection, neutral placeholder, then
explicitly permitted hide order. Invalid target projection rejects atomically
and retains the last-good frame state.

The locked verification writer produced resolver-backed runtime, fixture, and
diagnostics evidence after all declared validation commands passed. This
closeout proves only the static Draw checkpoint. Adaptive Draw behavior and the
selected Region Compass runtime remain later work, and the branch is not
mergeable before cleanup and final closeout. The required phase drift check also
ran `quiet_full_gate.sh`; Draw's package-scoped no-dependency Clippy gate passed,
but the workspace gate stopped on existing `ui_composition` and `ui_artifacts`
Clippy findings outside WR-184. Those findings remain explicit final-gate
blockers rather than being hidden or fixed through an unauthorized scope
expansion.
