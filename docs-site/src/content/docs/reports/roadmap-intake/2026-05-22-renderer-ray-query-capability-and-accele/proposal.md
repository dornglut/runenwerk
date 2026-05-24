---
title: Roadmap Intake WR-073
description: Roadmap intake proposal generated from a new idea.
status: completed
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-23
---

# Roadmap Intake WR-073

Idea: Renderer Ray Query Capability And Acceleration Resources
Suggested title: Renderer Ray Query Capability And Acceleration Resources
Planning state: `completed`

## Governance Notes

- Architecture governance review confirms `engine/src/plugins/render` owns
  ray-query capability and derived acceleration-resource inspection.
- No ADR is required for renderer-owned DTOs and fail-closed diagnostics.
- Stop for ADR if implementation introduces a durable cross-domain
  acceleration-resource ABI, moves source truth or fallback authority into
  renderer code, or requires RT hardware for baseline correctness.

## Readiness

- Source design:
  `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`.
- Dependencies: completed `PM-RENDER-RT-001` doctrine closeout and completed
  `WR-061` working-set/residency evidence.
- Implementation contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-073-renderer-ray-query-capability-and-acceleration-resources/plan.md`.
- Bounded-contract closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-073-renderer-ray-query-capability-and-acceleration-resources/closeout.md`.

## Scope

WR-073 added renderer-owned ray-query capability diagnostics, derived
acceleration-resource inspection, source-lineage evidence, unsupported-state
diagnostics, public API docs, tests, and closeout metadata. It does not expose
mutable backend handles, require RT hardware for baseline correctness, or move
producer truth into renderer inspection.

## Validation

```text
cargo fmt
cargo test -p engine render_ray_query
cargo test -p engine render_runtime_inspect
cargo test -p engine render_resource_model
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```

## Completion

WR-073 is complete at `bounded_contract` quality. Hybrid runtime proof remains
WR-074 scope, ray-query production evidence remains WR-075 scope, and final
no-gap verification remains `PT-RENDER-PERFECTION` scope.
