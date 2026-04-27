---
title: "UI Domain Surface Usage Guide"
description: "Documentation for UI Domain Surface Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# UI Domain Surface Usage Guide

## Purpose

Defines UI domain data types, template helpers, and text support models.

## Entry Points

- Module: engine/src/plugins/ui/mod.rs
- Entry: module surface only; no standalone Plugin implementation
- Local README: engine/src/plugins/ui/README.md

## Minimal Setup

```rust
use engine::plugins::ui::domain::*;

// No standalone plugin registration for this module surface.
```

## Runtime Contract

- Schedule placement: No direct schedule registration
- Ownership: UI data contracts and domain helpers.
- Non-ownership: Render scheduling and execution.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
