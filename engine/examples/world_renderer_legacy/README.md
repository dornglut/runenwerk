# Legacy World Renderer Reference

This folder stores the previously in-core world-compute and chunk-mesher implementation
that was removed from `engine::plugins::render`.

It is intentionally not wired into the core render plugin.

Use it as reference for building a dedicated feature/example plugin that owns:
- world compute/compose pipelines
- chunk meshing
- render pass executors
