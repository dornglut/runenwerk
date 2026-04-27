---
title: "Numerics"
description: "Documentation for Numerics."
status: active
owner: sdf
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# Numerics

Numerical policy for `domain/sdf`.

## Centralized Epsilons

Defaults live in `src/epsilon.rs`:

- sampling epsilon
- normal/gradient epsilon
- ray hit epsilon
- projection convergence epsilon
- classify epsilon

Use these defaults (or explicit overrides) instead of hardcoding unrelated
thresholds in individual modules.

## Finite Differences

Gradient estimates use central finite differences with axis offsets.

Tradeoffs:

- robust and simple baseline
- sensitive to epsilon selection
- can degrade near non-smooth composition seams

## Determinism

CPU query behavior is deterministic for fixed inputs and tolerances.
Floating-point behavior still depends on hardware/target specifics at the
expected `f32` precision level.

## Conservative Bounds

Bounds should err on the side of inclusion. If a robust conservative bound is
not available for an operation, return `FieldBounds::Unbounded`.
