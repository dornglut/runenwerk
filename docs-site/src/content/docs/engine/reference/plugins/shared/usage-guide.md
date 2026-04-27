---
title: "Shared Plugin Utilities Usage Guide"
description: "Documentation for Shared Plugin Utilities Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Shared Plugin Utilities Usage Guide

## Purpose

Provides cross-plugin helper utilities such as reload and watch formatting helpers.

## Entry Points

- Module: engine/src/plugins/shared/mod.rs
- Entry: module surface only; no standalone Plugin implementation
- Local README: engine/src/plugins/shared/README.md

## Minimal Setup

```rust
use engine::plugins::shared::{watch_status_line, ReloadStatusPayload};

// No standalone plugin registration for this module surface.
```

## Runtime Contract

- Schedule placement: No direct schedule registration
- Ownership: Reusable plugin utility helpers.
- Non-ownership: Feature-specific domain logic.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
