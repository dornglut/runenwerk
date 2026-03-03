# Runtime Minimal

Headless proof example for the typed engine runtime.

It demonstrates:

- `engine::App`
- `engine::App::headless()` / `run_for_frames(...)`
- `engine::Plugin`
- `Startup` and `Update` schedules
- `Query`, `ResMut`, and `Commands`
- `ecs_v2`-backed world state without `EngineData`

Run it with:

```bash
cargo run -p engine --example runtime_minimal
```
