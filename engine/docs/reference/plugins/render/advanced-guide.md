# Render Plugin Advanced Guide

## Advanced Surfaces

- typed explicit resource handles (`storage_array` + `bind_storage`)
- ergonomic ping-pong binding (`double_buffer_storage_array` + `bind_ping_pong_storage`)
- state-projected uniforms and dispatch
- transient/persistent/imported resource lifetime modeling
- inspection surfaces under `engine::plugins::render::inspect`

## Validation and Safety

Use:

- `RenderFlow::validate()` for chainable validation
- `RenderFlow::validation_report()` for inspectable contract checks

Validation catches:

- duplicate and unknown IDs
- pass-shape errors
- dependency cycles
- invalid resource usage for pass bindings

## Contract Inspection

Use:

- `flow.graph()` for pass/resource declarations
- `flow.project_uniforms(...)` for frame-level uniform projection checks
- `dump_flow_graph(...)`, `inspect_resources(...)`, `inspect_texture_resources(...)`, and `summarize_pass_timings(...)` for runtime diagnostics

These APIs keep the graph explicit and testable while keeping common-path declaration compact.
