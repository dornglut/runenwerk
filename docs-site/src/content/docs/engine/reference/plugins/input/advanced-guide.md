---
title: "Input Plugin Advanced Guide"
description: "Advanced Runenwerk integration guidance for standalone RunenInput."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
---

# Input Plugin Advanced Guide

## Extension pattern

- Add reusable device observation/state semantics in `dornglut/runen-input`.
- Add Runenwerk action ids, binding presets, rebinding, and product projection in the
  product action/binding owner.
- Keep winit/native acquisition at explicit outer adapters.
- Do not re-export RunenInput values through Engine as a compatibility namespace.
- Preserve one reducer: Runenwerk integration delegates confirmed state to
  `runen_input::InputState`.

## Integration notes

Use `InputState` for Runenwerk ECS/frame integration and `ActionState` for product
actions. Import reusable semantic values such as `PhysicalKeyIdentity`,
`InputContext`, or `TabletObservation` directly from `runen_input`.

When projecting frame edges, compare confirmed state before and after
`InputState::admit`; do not reconstruct the predecessor's private digital-admission
types.

Preserve repeat/synthetic reconciliation rules, source/device aggregation, measurement
uncertainty, contact identity/cancellation, and predicted-vs-confirmed tablet semantics.

## Validation focus

- exact accepted RunenInput revision remains pinned;
- predecessor `neutral.rs` and forwarding namespaces remain absent;
- focused Engine input tests preserve frame/product behavior;
- native-tablet, Draw, and Editor consumers compile against direct RunenInput imports;
- repository-owned `cargo validate` and exact-head hosted CI remain green.
