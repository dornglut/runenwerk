# Game of Life SDF Example

Feature-owned render example that uses the engine `RenderPlugin` with:

- a custom compute executor to advance Conway's Game of Life on GPU
- a custom compose executor to render cells via signed-distance-field shading

## Entry

- `engine/examples/game_of_life_sdf/main.rs`

## Shader

- `assets/shaders/game_of_life_sdf.wgsl`

## Run

```bash
cargo run -p engine --example game_of_life_sdf
```

## Controls

- `Space`: pause / resume simulation
- `Enter`: single-step one generation
- `PageUp`: increase simulation rate
- `PageDown`: decrease simulation rate
