---
title: "Time Plugin"
description: "Documentation for Time Plugin."
---

# Time Plugin

## Purpose

Updates frame timing state each engine tick.

## Usage

- Plugin: `TimePlugin`
- Typed schedule: `PreUpdate`
- Typed set: `CoreSet::Time`

The plugin advances `Time` once per frame on the runtime.

## Ownership Boundaries

- Owns frame delta/time progression.
- Does not own downstream consumers of timing state.

## Extension Points

- Add fixed-step utilities/resources tied to frame tick lifecycle.
- Add additional timing diagnostics/state as needed.

## Guides

- Usage: [../../../docs/reference/plugins/time/usage-guide.md](../../reference/plugins/time/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/time/advanced-guide.md](../../reference/plugins/time/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/time/architecture.md](../../reference/plugins/time/architecture.md)


