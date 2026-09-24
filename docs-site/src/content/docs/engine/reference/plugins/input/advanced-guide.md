---
title: "Input Plugin Advanced Guide"
description: "Documentation for Input Plugin Advanced Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Input Plugin Advanced Guide

## Extension Pattern

- Extend physical/device semantics in the owning input state/neutral modules only under accepted input-boundary authority.
- Extend Runenwerk product action vocabulary, binding presets, or rebinding in the product action/binding owner rather than in `InputState`.
- Keep composition changes in plugin build surfaces and avoid moving ownership across domains.
- Keep schedule assumptions aligned with action projection in `PreUpdate` / `CoreSet::Input` and frame cleanup in `FrameEnd` / `CoreSet::FrameEnd`.

## Integration Notes

- Reuse `InputState`, `ActionState`, and existing neutral input identities before introducing new abstractions.
- Bind product actions to `PhysicalKeyIdentity`; backend adapters may translate winit evidence at explicit outer boundaries, but binding semantic types must stay backend-neutral.
- Prefer typed schedule ordering (`CoreSet` and schedule markers) when adding consumers around input projection.
- Preserve press-time modifier evidence, repeat/synthetic reconciliation rules, and aggregate multi-device held semantics documented in the local README and accepted physical-input design.

## Validation Focus

- Verify startup/resource installation and `PreUpdate` action projection in headless tests.
- Verify ordinary first press, repeat suppression, synthetic reconciliation, multi-device aggregation, and press-time modifier matching.
- Verify rebinding while held recomputes `action_down` without fabricating `action_pressed`.
- Verify cross-plugin consumers read product actions from `ActionState` while non-action pointer/text/touch behavior remains on `InputState`.
