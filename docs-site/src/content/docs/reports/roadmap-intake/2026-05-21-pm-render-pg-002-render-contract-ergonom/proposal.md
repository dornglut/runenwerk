---
title: Roadmap Intake WR-041
description: Roadmap intake proposal for the bounded PM-RENDER-PG-002 render contract ergonomics implementation row.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-21
---

# Roadmap Intake WR-041

Idea: create a legal bounded WR row for `PM-RENDER-PG-002` after the render
contract ergonomics design moved to accepted.

Suggested title: PM-RENDER-PG-002 Render Contract Ergonomics

Initial planning state: `ready_next`

## Governance Notes

- Architecture governance kickoff was run for the bounded PM-002 scope.
- The DDD owner is the engine render plugin for request contracts, with
  `apps/runenwerk_editor` as the explicit producer owner.
- No ADR is required while PM-002 preserves the accepted Render Product Graph
  ownership boundary and adds migration-safe, return-only helpers.

## Bounded Scope

- `engine/src/plugins/render`
- `engine/tests`
- `apps/runenwerk_editor/src/runtime/viewport`
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs`
- relevant render/product-track docs, generated production docs, closeout
  evidence, and focused tests

## Validation

```text
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
cargo test -p runenwerk_editor viewport::render_jobs
cargo test -p runenwerk_editor material_preview
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-21-pm-render-pg-002-render-contract-ergonom/proposal.yaml
```
