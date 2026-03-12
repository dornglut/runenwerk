# ECS Crate

`ecs` is the ECS runtime foundation in the `foundation` domain.

This README is intentionally concise and points to the canonical docs tree.

## Start Here

- Docs entrypoint: [`foundation/ecs/docs/index.md`](./docs/index.md)
- Reference docs:
  - [`foundation/ecs/docs/reference/usage-guide.md`](./docs/reference/usage-guide.md)
  - [`foundation/ecs/docs/reference/architecture.md`](./docs/reference/architecture.md)
- Roadmaps:
  - [`foundation/ecs/docs/roadmaps/phase6a-archetype-storage-plan.md`](./docs/roadmaps/phase6a-archetype-storage-plan.md)
  - [`foundation/ecs/docs/roadmaps/phase6-archetype-full-switch-plan.md`](./docs/roadmaps/phase6-archetype-full-switch-plan.md)
  - [`foundation/ecs/docs/roadmaps/phase6-closeout-roadmap.md`](./docs/roadmaps/phase6-closeout-roadmap.md)
- Phase 6 benchmark docs:
  - [`foundation/ecs/docs/benchmarks/phase6/benchmark-suite.md`](./docs/benchmarks/phase6/benchmark-suite.md)
  - [`foundation/ecs/docs/benchmarks/phase6/progress-report.md`](./docs/benchmarks/phase6/progress-report.md)
  - [`foundation/ecs/docs/benchmarks/phase6/final-decision-report.md`](./docs/benchmarks/phase6/final-decision-report.md)

## Runtime Surface (At a Glance)

- `World`: entity lifecycle, resources, events, indexes, command queue creation
- `QueryState<Q, F = ()>`: reusable detached query state from `World::query_state`
- `Runtime`: schedule registration and execution
- System params: `Query`, `Res`, `ResMut`, `Commands`, `EventReader`, `EventWriter`
- Query filters: `With<T>`, `Without<T>`, `Changed<T>`, `Added<T>`

## Validation Commands

```powershell
cargo test -p ecs
```

Phase 6 closeout benchmark/profile commands are documented in:
[`foundation/ecs/docs/benchmarks/phase6/benchmark-suite.md`](./docs/benchmarks/phase6/benchmark-suite.md)
