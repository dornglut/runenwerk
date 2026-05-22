---
title: Roadmap Intake WR-062
description: Roadmap intake proposal generated from a new idea.
status: completed
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-062

Idea: Renderer Scale GPU Driven Culling LOD And Indirect Submission
Suggested title: Renderer Scale GPU Driven Culling LOD And Indirect Submission
Initial planning state: `blocked_deferred`
Current planning state: `completed`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- None. WR-062 completed with bounded renderer visibility, LOD, compaction,
  and indirect-submission inspection evidence.

## Resolved Decisions

- Promotion evidence is the accepted scale doctrine at
  `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`,
  the GPU evidence doctrine at
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`,
  completed WR-061 working-set residency budget evidence at
  `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`,
  completed WR-056 GPU timing evidence at
  `docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md`,
  and the WR-062 implementation contract at
  `docs-site/src/content/docs/reports/implementation-plans/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/plan.md`.
- WR-062 depends on completed WR-061 residency/working-set evidence and
  completed WR-056 GPU timing/capability evidence.
- Implementation is bounded to renderer execution/inspection modules, focused
  engine tests, renderer reference docs, roadmap metadata, production metadata,
  and WR-062 closeout evidence.
- No ADR is required unless implementation moves product semantic LOD,
  streaming, fallback, freshness, visibility truth, or cross-domain ABI
  ownership into renderer code.
- Promotion preflight reported `promotable`; the promotion command is recorded
  in
  `docs-site/src/content/docs/reports/implementation-plans/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/plan.md`.
- WR-062 completed at `bounded_contract` quality and now provides
  `PM-RENDER-SCALE-003` evidence through
  `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`.
- Runtime scale proof, hardware profiles, benchmarks, and production readiness
  remain WR-063 scope.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
