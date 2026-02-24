# Grid Plugin

## Purpose

Prepares world/grid render parameters from gameplay configuration.

## Usage

- Plugin: `GridPlugin`
- Scheduler node: `grid_prepare`
- Runs after: `world_scene_update`

The plugin writes grid/chunk parameters into the world render resource each frame.

## Ownership Boundaries

- Owns grid-specific render parameter extraction from scene gameplay config.
- Does not own world simulation or render pass execution.

## Extension Points

- Add additional grid/world streaming parameters in `grid_prepare_system`.
- Extend gameplay config mapping as new grid controls are introduced.
