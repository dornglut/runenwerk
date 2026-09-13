---
title: "Time Plugin"
description: "Documentation for Time Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Time Plugin

## Purpose

Installs and advances frame timing state for runtime consumers.

## Usage

- Plugin: `TimePlugin`
- Owned resource: `Time`
- Typed schedule: `PreUpdate`
- Typed set: `CoreSet::Time`

Selecting `TimePlugin` initializes `Time` when it is absent and advances it once per frame.
An explicitly pre-inserted `Time` value is preserved during plugin installation.
Bare `App` construction does not imply timing state; applications that need frame timing
select `TimePlugin` directly or through a plugin stack such as `default_plugins()`.

## Ownership Boundaries

- Owns default `Time` installation and frame delta/time progression.
- Does not own fixed-step catchup semantics or downstream consumers of timing state.

## Extension Points

- Add timing diagnostics/state that remain semantically owned by frame timing.
- Keep fixed-step cadence and simulation identity in their respective owners.

## Guides

- Usage: [../../../docs/reference/plugins/time/usage-guide.md](../../reference/plugins/time/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/time/advanced-guide.md](../../reference/plugins/time/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/time/architecture.md](../../reference/plugins/time/architecture.md)

