---
title: "Grid Plugin"
description: "Documentation for Grid Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Grid Plugin

## Purpose

Prepares world/grid render parameters from gameplay configuration.

## Usage

- Plugin: `GridPlugin`
- Typed schedule: `Update`
- Typed set: `CoreSet::Scene`

The plugin writes grid/chunk parameters from gameplay state into `GridRuntimeConfig` each frame.

## Ownership Boundaries

- Owns grid-specific render parameter extraction from scene gameplay config.
- Owns `GridRuntimeConfig` resource publication for render consumers.
- Does not own world simulation or render pass execution.

## Extension Points

- Add additional grid/world streaming parameters in `grid_prepare_system`.
- Extend gameplay config mapping as new grid controls are introduced.

## Guides

- Usage: [../../../docs/reference/plugins/grid/usage-guide.md](../../reference/plugins/grid/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/grid/advanced-guide.md](../../reference/plugins/grid/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/grid/architecture.md](../../reference/plugins/grid/architecture.md)


