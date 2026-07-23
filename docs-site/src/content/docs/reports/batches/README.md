---
title: Batch Reports
description: Index of preserved parallel roadmap batch manifests, prompts, closeouts, and historical proposal artifacts.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-20
---

# Batch Reports

This folder preserves generated parallel-roadmap batch evidence. Batch artifacts
are historical records, not active roadmap truth. The active roadmap source is
[`Roadmap`](../../workspace/planning/roadmap.md), and
the generated roadmap view is
[`../../workspace/roadmap-decision-register.md`](../../workspace/roadmap-decision-register.md).

## Current Proposal

No active parallel roadmap batch proposal is indexed here. `WR-018` and
`WR-020` are completed closeout evidence, and `WR-026` editor adapters remain
not-started downstream work.

## Historical Proposal Artifacts

- Rejected WR-029 parallel-batch proposal: historical context only. WR-029
  Phase 1-3 evidence lives in
  [`../implementation-plans/wr-029-model-mesh-material-binding/plan.md`](../implementation-plans/wr-029-model-mesh-material-binding/plan.md),
  and Phase 4 remains open for model/mesh GPU pixel proof.
- Rejected WR-018 continuation proposal: historical context only. WR-018
  completion evidence lives in
  [`../closeouts/wr-018-rendered-world-v1/closeout.md`](../closeouts/wr-018-rendered-world-v1/closeout.md).

## Completed Batch Evidence

- [`2026-05-14-l0-substrate-pilot`](2026-05-14-l0-substrate-pilot/batch.md):
  integrated L0 ECS/runtime and render contract support.
- [`2026-05-14-wr-001-product-job-draw-bridge`](2026-05-14-wr-001-product-job-draw-bridge/batch.md):
  integrated WR-001 product-job and Draw bridge stabilization.
- [`2026-05-15-wr-025-interaction-v2-menu-stack-scroll-ownership`](2026-05-15-wr-025-interaction-v2-menu-stack-scroll-ownership/batch.md):
  integrated WR-025 menu-stack and scroll-ownership slice.
- [`2026-05-15-next-current-candidate-roadmap-batch-wr-025-wr-018`](2026-05-15-next-current-candidate-roadmap-batch-wr-025-wr-018/batch.md):
  integrated WR-025 and WR-018 candidate batch.
- [`2026-05-15-next-current-candidate-roadmap-batch-wr-`](2026-05-15-next-current-candidate-roadmap-batch-wr-/batch.md):
  older integrated current-candidate batch for WR-025, WR-018, and WR-007.
  Kept for evidence continuity even though the slug was produced before batch
  id suffix hardening.
- [`2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025`](2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025/batch.md):
  integrated WR-025 continuation slice.
- [`2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-2`](2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-2/batch.md):
  integrated WR-025 continuation slice.
- [`2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-3`](2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-3/batch.md):
  integrated WR-025 continuation slice.
- [`2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-4`](2026-05-15-continue-roadmap-batch-after-2026-05-15-wr-025-4/batch.md):
  integrated WR-025 doctrine-repair closeout slice.

## Preservation Rules

- Keep batch manifests, worker prompts, and rendered `batch.md` reports when
  they contain unique validation, integration, or tooling-hardening evidence.
- Proposed batches may remain only while they represent the current intended
  continuation and all prompt paths are repository-relative.
- Completed batches must record `integration_status = "merged"` and
  `closeout_status = "completed"` in `batch.toml`.
- Historical malformed slugs may remain when indexed here, but new generated
  batches should use suffix-preserving batch ids.
