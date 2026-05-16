---
title: WR-006 Draw DRF4-DRF5 Closeout
description: Retrospective completion evidence for Draw GPU ink proof and GPU promotion/fallback.
status: completed
owner: apps/runenwerk_draw
layer: apps
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/runenwerk-draw-pen-first-radial-tablet-ux-design.md
related_roadmaps:
  - ../../../apps/runenwerk-draw/roadmap.md
  - ../../../apps/runenwerk-draw/render-foundation-roadmap.md
  - ../../../workspace/roadmap-items.yaml
---

# WR-006 Draw DRF4-DRF5 Closeout

## Status

Complete as of 2026-05-15. This retrospective closeout preserves the existing
roadmap evidence in a path-backed artifact so completed roadmap rows cannot rely
on prose-only status.

## Completion Evidence

- `apps/runenwerk_draw/src/runtime/gpu_ink.rs` owns the Draw GPU ink proof path.
- `apps/runenwerk_draw/src/runtime/ink_jobs.rs` and adjacent runtime systems keep
  CPU tile formation as canonical input while GPU promotion/fallback consumes
  render-flow and product-surface contracts.
- `apps/runenwerk_draw/tests/app_shell.rs` covers the app shell flow used by the
  Draw roadmap validation.

## Boundaries

- Future Draw work should preserve CPU truth and public render-flow/product
  surface boundaries.
- This closeout does not reopen product-job ownership, procgen bake/rollback, or
  editor viewport work.

## Validation

Roadmap validation for the completed row records:

- `cargo test -p runenwerk_draw`
- `cargo test -p engine`

