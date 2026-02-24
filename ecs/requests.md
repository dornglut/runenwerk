# ECS Requests

All previously tracked ECS feature requests are complete as of 2026-02-24.

Implemented:

- secondary component indexes (default + named indexes)
- mutable query builder with `with/without` filters (`query_mut_components`)
- component/resource change ticks
- fine-grained component/resource change records
- entity lifecycle events (`EntitySpawnedEvent`, `EntityDespawnedEvent`)
- event drain helpers (`drain_events_map`, `drain_events_filter`)

Open requests: none.
