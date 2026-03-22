# Procedural Sky + SDF Terrain Example

Windowed public `RenderFlow` sample that raymarches a noise-shaped SDF terrain and a fully procedural sky in one fullscreen pass.

## Controls

- `Tab`: cycle debug view mode (`lit` -> `height` -> `normals` -> `steps`)

## Structure

- `main.rs`
  - entry point
- `rendering/graph.rs`
  - flow declaration (`with_state`, `with_surface_color`, fullscreen compose pass)
- `rendering/state.rs`
  - ECS-owned camera/time/view state and projected uniform payload
- `runtime/app.rs`
  - app/plugin wiring and per-frame animation + view switching
- shader:
  - `assets/shaders/procedural_sky_sdf_terrain_compose.wgsl`

## Run

```bash
cargo run -p engine --example procedural_sky_sdf_terrain
```
