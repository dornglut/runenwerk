---
title: PM-UI-COMPOSITION-006 Runtime Closeout
status: completed
closeout_evidence:
  milestone_id: PM-UI-COMPOSITION-006
  wr_id: WR-185
  completion_quality: runtime_proven
  evidence_categories:
    - runtime_test
    - fixture
    - visual
    - diagnostics
  validation_commands:
    - cargo fmt --all --check
    - cargo test -p ui_adaptive_composition
    - cargo test -p ui_input semantic
    - cargo test -p ui_testing adaptive_composition
    - cargo test -p ui_composition
    - cargo bench -p ui_adaptive_composition --bench adaptive_composition
    - cargo clippy -p ui_adaptive_composition -p ui_input -p ui_testing --all-targets --all-features --no-deps -- -D warnings
    - cargo clippy -p ui_composition --all-targets --all-features --no-deps -- -D warnings -A clippy::large_enum_variant
    - task ui:dependencies
    - task docs:validate
    - task planning:validate
    - ./quiet_full_gate.sh
  validation_results:
    - 'cargo:fmt (cargo fmt --all --check) -> exit 0'
    - 'cargo:test (cargo test -p ui_adaptive_composition) -> exit 0: 8 tests'
    - 'cargo:test (cargo test -p ui_input semantic) -> exit 0: 2 tests'
    - 'cargo:test (cargo test -p ui_testing adaptive_composition) -> exit 0: 1 test'
    - 'cargo:test (cargo test -p ui_composition) -> exit 0: 37 tests'
    - 'cargo:bench (cargo bench -p ui_adaptive_composition --bench adaptive_composition) -> exit 0: all eight separate p95 budgets passed; drag-frame full graph clones = 0'
    - 'cargo:clippy-adaptive-input-testing -> exit 0'
    - 'cargo:clippy-composition-with-known-large-enum-exemption -> exit 0'
    - 'task:ui:dependencies (task ui:dependencies) -> exit 0'
    - 'task:docs:validate (task docs:validate) -> exit 0'
    - 'task:planning:validate (task planning:validate) -> exit 0: 288 workflow tests plus roadmap, production, prompt, and docs checks'
    - 'execution:harness (single ActionContract PM-UI-COMPOSITION-006) -> exit 0 with four resolver-backed evidence records'
    - 'quiet-full-gate (./quiet_full_gate.sh) -> exit 1: existing ui_composition large_enum_variant, ui_artifacts too_many_arguments/collapsible_if, and ui_story clone_on_copy findings'
  files_changed:
    - Cargo.toml
    - Cargo.lock
    - domain/ui/ui-crate-ownership.toml
    - domain/ui/ui_adaptive_composition/Cargo.toml
    - domain/ui/ui_adaptive_composition/src/lib.rs
    - domain/ui/ui_adaptive_composition/src/accessibility/mod.rs
    - domain/ui/ui_adaptive_composition/src/diagnostic/mod.rs
    - domain/ui/ui_adaptive_composition/src/fixture/mod.rs
    - domain/ui/ui_adaptive_composition/src/interaction/hit_index.rs
    - domain/ui/ui_adaptive_composition/src/interaction/session.rs
    - domain/ui/ui_adaptive_composition/src/projection/model.rs
    - domain/ui/ui_adaptive_composition/src/projection/policy.rs
    - domain/ui/ui_adaptive_composition/src/promotion/mod.rs
    - domain/ui/ui_adaptive_composition/src/proposal/model.rs
    - domain/ui/ui_adaptive_composition/benches/adaptive_composition.rs
    - domain/ui/ui_adaptive_composition/benchmark-artifacts/reference-desktop-2026-06-20.txt
    - domain/ui/ui_composition/src/transaction/apply.rs
    - domain/ui/ui_composition/src/history/journal.rs
    - domain/ui/ui_composition/tests/transaction_atomicity.rs
    - domain/ui/ui_input/src/lib.rs
    - domain/ui/ui_input/src/semantic.rs
    - domain/ui/ui_testing/src/lib.rs
    - domain/ui/ui_testing/src/composition_fixture.rs
    - domain/ui/ui_testing/src/adaptive_composition_fixture.rs
    - docs-site/src/content/docs/domain/ui/ui-adaptive-composition-usage.md
    - docs-site/src/content/docs/reports/implementation-plans/wr-185-adaptive-composition-headless-proposals-and-semantic-input/plan.md
    - docs-site/src/content/docs/reports/implementation-plans/wr-185-adaptive-composition-headless-proposals-and-semantic-input/plan.contract.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-006/runtime_test-adaptive-proposals-and-semantic-input.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-006/fixture-neutral-adaptive-conformance.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-006/visual-accessibility-headless-metadata.yaml
    - docs-site/src/content/docs/reports/execution-evidence/pt-ui-composition-cutover/pm-ui-composition-006/diagnostics-performance-and-clone-probes.yaml
  known_gaps:
    - Region Compass runtime chrome, editor transaction materialization, and native-window coordination remain governed by WR-186.
    - Draw adaptive runtime, legacy-authority deletion, and independent perfectionist verification remain later checkpoints.
    - The clean-cutover branch cannot merge before cleanup and final closeout pass.
    - Repository-wide Clippy remains blocked by explicitly recorded ui_composition, ui_artifacts, and ui_story findings; they require owning scope before the final merge gate.
  closeout_path: docs-site/src/content/docs/reports/closeouts/pm-ui-composition-006-adaptive-headless-proposals/closeout.md
  produced_at: 2026-06-20T13:17:37Z
---

# PM-UI-COMPOSITION-006 Runtime Closeout

`ui_adaptive_composition` now owns transient adaptive projection, deterministic
edit classification, revision-bound docking and resize intent, preview state,
promotion deltas, interaction sessions, accessibility metadata, and headless
fixture policy. Pointer, keyboard, touch, and controller sources converge on the
same semantic action vocabulary in `ui_input`.

The proposal boundary was corrected before closeout. Left, right, top, bottom,
and center Region Compass zones remain typed intent through the adaptive layer;
`AdaptiveProposal` contains no `CompositionCommand` and exposes
`requires_host_transaction()` for structural intent. The editor host must
materialize the ordered, topology-aware transaction after app policy, identity
allocation, extension-state planning, and current-revision checks. This prevents
edge zones from falsely behaving as center moves and preserves the accepted
proposal-only ownership contract.

The benchmark uses a 2,048-region, 1,024-mounted-unit, 16-target large fixture
and a 64-command transaction path. Separate p95 measurements pass for region hit
testing, proposal generation, preview projection, drag-frame updates,
transaction validation, committed mutation, serialization, and validation plus
deserialization. Drag-frame updates report zero full graph clones. The core
transaction path therefore performs one atomic candidate clone and validates
resize-only batches without cloning the full graph per command or pointer move.

The locked verification writer regenerated four resolver-backed evidence
records after every declared command passed. Browser, terminal, dashboard,
mobile, and game fixtures remain product-free contracts rather than product
implementations. The workspace-wide full gate still reports known findings in
`ui_composition`, `ui_artifacts`, and `ui_story`; these are explicit final-gate
blockers and do not weaken this bounded checkpoint's runtime proof.
