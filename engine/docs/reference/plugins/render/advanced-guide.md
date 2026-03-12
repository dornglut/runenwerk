# Render Plugin Advanced Guide

## Advanced Surfaces

- `RenderFlowContribution` multi-plugin composition
- data-driven fragment compilation (`RenderFlowFragmentSpec`)
- fragment hot reload tracking (`RenderFlowFragmentHotReloadState`)
- transient/persistent/imported resource lifetime modeling
- copy/present/graphics pass kinds
- inspection surfaces under `engine::plugins::render::inspect`

## Validation and Safety

Use `RenderFlow::validate()` before flow registration to catch:

- pass/resource ID collisions
- pass-shape errors (`copy_pass`, `present_pass`)
- incompatible resource usage
- dependency cycles and unknown references

## Internal/Advanced Plumbing

- executor and graph registry plumbing remains internal/advanced infrastructure
- the preferred user surface is `RenderFlow` + ECS projection + contributions

## Inspection

Use:

- `dump_flow_graph(...)`
- `inspect_resources(...)`
- `inspect_texture_resources(...)`
- `summarize_pass_timings(...)`

for debugging mixed plugin-owned flows.
