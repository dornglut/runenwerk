---
title: Roadmap Intake WR-043
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-21
---

# Roadmap Intake WR-043

Idea: PM-RENDER-PG-004 bounded implementation WR for render execution graph compiler maturity. Create a new legal WR row, not WR-010. Scope: mature compiler validation/preflight over RenderFlow, PreparedRenderFrame, prepared flow invocations, target aliases, dynamic targets, history scopes, resource lifetime windows, backend capability profile diagnostics, and inspection. Must not implement render fragments, fragment assets, hot reload, last-good fragment promotion, broad product-surface producer hardening, native multi-window, material lowering, product truth, product freshness, authority, fallback legality, rebuild policy, or residency policy. Dependencies: WR-042 completed and accepted PM-004 design docs-site/src/content/docs/design/accepted/render-execution-graph-compiler-maturity-design.md. Write scopes limited to engine/src/plugins/render, engine/tests, relevant render reference/roadmap docs, production/roadmap workflow files, and the PM-004 implementation plan/closeout artifacts after they exist. Validations: cargo test -p engine render_flow; cargo test -p engine render_dynamic_targets; cargo test -p engine render_runtime_inspect; workflow docs/roadmap/production/planning/goal gates.
Suggested title: PM-RENDER-PG-004 Render Execution Graph Compiler Maturity
Initial planning state: `ready_next`

## Governance Notes

- Architecture governance review ran before design acceptance.
- DDD owner is `engine/src/plugins/render`.
- No ADR is required while the compiler contract remains engine-local and does not become a cross-domain ABI, persisted graph format, external mod contract, or source-of-truth ownership change.

## Open Questions

- None for intake application. Promotion still requires `task production:plan -- --milestone PM-RENDER-PG-004 --roadmap WR-043`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
