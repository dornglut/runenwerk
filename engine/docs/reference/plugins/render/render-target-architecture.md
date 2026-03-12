# Render Architecture Cutover

This reference is a short map to the canonical render architecture definition.

## Source of Truth

- Hard-cut roadmap:
  - `engine/src/plugins/render/docs/roadmap.md`

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

- Normal authoring path: `RenderFlow` and `RenderFlowContribution`.
- Normal runtime path: builtin compiled execution (not custom executor registration).
- Missing builtin pass implementations must fail loudly.
- Input bindings stay in app/input APIs (`App::add_input_bindings`), not in render flow declarations.
