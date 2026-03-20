# Render Architecture Cutover

This reference is a short map to the current RenderFlow v2 architecture.

## Canonical Module Surface

The core render plugin architecture is:

- `api/`
- `backend/`
- `composition/`
- `graph/`
- `inspect/`
- `params/`
- `pipelines/`
- `renderer/`
- `resource/`
- `shader/`

Legacy architecture trees (`frame_graph/`, `resources/`, `domain/`, `debug/`) are removed from core render.

## Authoring Direction

- normal authoring path: `RenderFlow` v2
- common path: `with_state` + `double_buffer_storage_array` + pass builders
- graph remains explicit and inspectable through `validation_report`, `graph`, and `project_uniforms`
- advanced path remains explicit with typed handle bindings (`storage_array`, `bind_storage`, explicit uniform handles)

## Explicitly Removed

- flow contribution merge APIs
- data-driven fragment composition APIs
- string-only low-level pass read/write binding path in normal usage
