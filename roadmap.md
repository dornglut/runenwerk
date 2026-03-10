# Runenwerk Multiplayer Rewrite Roadmap (Revised)

## 1. Scope
This roadmap revises migration sequencing for the pinned architecture without redesigning it.

Pinned stack:
`games/*/src/net` -> `engine/src/plugins/net` -> `net/engine_net` -> `net/engine_net_quic`

Pinned ownership:
- `engine_sim`: simulation vocabulary, ticks, deterministic identity.
- `engine_net`: transport-agnostic protocol/session/replication/prediction semantics.
- `engine_net_quic`: QUIC runtime adapter only.
- `engine_net_macros`: `#[net_component]`, `#[net_entity]` metadata generation only.
- `engine/src/plugins/net`: thin ECS bridge only.
- `games/*/src/net`: gameplay schema, driver/apply/input implementations, smoothing/correction tuning.

## 2. Repository Scan Summary
Workspace crates in scope:
- `engine`
- `net/engine_sim`
- `net/engine_net`
- `net/engine_net_macros`
- `net/engine_net_quic`
- `net/engine_history` (`engine_replay`)
- `games/cavern_hunt`
- `apps/grotto_client`
- `apps/grotto_server`
- `apps/grotto_online`
- `apps/grotto_fleet_control`

Current mismatches:
- Engine plugin currently owns replication/prediction runtime logic.
- `ReplicationDriver` is monolithic.
- `games/cavern_hunt/src/net` still contains custom runtime replication path.
- Apps still install `NetworkClientPlugin` / `NetworkServerPlugin`.
- `engine_net_quic` still has transitional broad modules (`helpers.rs`, `utils.rs`).

## 3. Gap to Target
Required convergence:
- Split monolithic driver into `ReplicationDriver`, `SnapshotApplyDriver`, `InputDriver`.
- Move replication/prediction algorithm ownership to `engine_net`.
- Replace public multi-plugin API with `NetPlugin<TDriver>::client/server/host`.
- Keep host mode strict composition (`Client + Server`).
- Migrate Cavern Hunt in three safe steps:
1. Introduce new net structure.
2. Switch runtime to new structure.
3. Remove legacy runtime path.
- Keep full snapshots as correctness baseline before relying on deltas.

## 4. Migration Strategy
Migration principles:
- Strangler approach with temporary compatibility layers.
- No big-bang API break.
- App migration occurs before deleting old runtime paths.
- Plugin thinness enforced throughout.
- Transport adapter remains gameplay-agnostic.
- Full snapshot correctness gate must pass before delta-default enablement.

Temporary compatibility layers:
- Legacy-to-new driver adapter in `engine_net`.
- Legacy plugin wrappers around `NetPlugin<TDriver>`.
- Cavern legacy runtime path retained until app migration is complete.

## 5. Phased Implementation Plan

### Phase 0 - Baseline Lock and Safety Harness
Goal:
Freeze behavior and add migration safety checks before moving ownership boundaries.

Changes:
- Pin baseline tests for session/replication/plugin behavior.
- Add explicit "full snapshot works" regression tests.
- Record expected schedule ordering for net receive/fixed/flush phases.

Affected files/modules:
- `engine/tests/network_plugins.rs`
- `games/cavern_hunt/tests/live_multiplayer/helpers.rs`
- `net/engine_net/REPLICATION_PIPELINE.md`
- `engine/src/plugins/net/NETWORK_RUNTIME_FLOW.md`

Completion criteria:
- Baseline tests pass in CI and locally.
- Snapshot-first invariants are documented and test-backed.

### Phase 1 - Driver Contract Split (Compatibility First)
Goal:
Introduce target driver model without breaking existing runtime flow.

Changes:
- Add split traits: `ReplicationDriver`, `SnapshotApplyDriver`, `InputDriver`.
- Keep temporary compatibility adapter from legacy monolithic trait to split traits.
- Re-export split traits as canonical API, keep legacy trait deprecated but usable.

Affected files/modules:
- `net/engine_net/src/replication/*` (trait definitions and adapters)
- `net/engine_net/src/lib.rs`
- `net/engine_net/README.md`

Completion criteria:
- Existing call sites compile unchanged via adapter.
- New split trait API compiles and is test-covered.

### Phase 2 - Move Net Semantics to engine_net (Full Snapshot Path Only)
Goal:
Remove algorithm ownership from engine plugin; keep plugin as bridge.

Changes:
- Move replication/prediction algorithmic steps into `engine_net` runtime/replication modules.
- Refactor engine plugin systems to call `engine_net` semantic APIs only.
- Keep delta path non-default during migration; full snapshots remain authoritative baseline.

Affected files/modules:
- `net/engine_net/src/runtime/client.rs`
- `net/engine_net/src/runtime/server.rs`
- `net/engine_net/src/replication/*`
- `engine/src/plugins/net/prediction.rs`
- `engine/src/plugins/net/runtime_io.rs`

Completion criteria:
- Engine plugin contains orchestration/bridging only, not replication logic.
- Server-client sync works with full snapshots only.

### Phase 3 - Introduce NetPlugin<TDriver> (Keep Legacy Wrappers)
Goal:
Ship target plugin API while preserving runtime stability.

Changes:
- Add `NetRole` and `NetPlugin<TDriver>::client/server/host`.
- Add role-aware startup wiring and runtime bridge modules.
- Keep `NetworkClientPlugin`, `NetworkServerPlugin`, `ReplicationPlugin`, `PredictionPlugin` as wrappers delegating to `NetPlugin<TDriver>`.

Affected files/modules:
- `engine/src/plugins/net/mod.rs`
- `engine/src/plugins/net/plugin.rs`
- `engine/src/plugins/net/config.rs` (new)
- `engine/src/plugins/net/startup.rs` (new)
- `engine/src/plugins/net/runtime_bridge.rs` (new)
- `engine/src/plugins/net/resources.rs`
- `engine/src/plugins/net/schedules.rs`
- `engine/src/plugins/net/diagnostics.rs`

Completion criteria:
- New plugin API is production-ready.
- Legacy plugin API still functions through wrappers.
- Host constructor composes client+server semantics only.

### Phase 4 - QUIC Adapter Alignment and Boundary Hardening
Goal:
Align `engine_net_quic` to updated contracts while keeping transport-only ownership.

Changes:
- Update QUIC runtime handle integration to new contract surface.
- Replace broad runtime utility buckets with explicit responsibility modules.
- Keep lane/profile mapping delegated to `engine_net` semantics.

Affected files/modules:
- `net/engine_net_quic/src/runtime/handles.rs`
- `net/engine_net_quic/src/runtime/routing.rs`
- `net/engine_net_quic/src/client/runtime.rs`
- `net/engine_net_quic/src/server/runtime.rs`
- `net/engine_net_quic/src/runtime/helpers.rs` (split)
- `net/engine_net_quic/src/runtime/utils.rs` (split)
- `net/engine_net_quic/src/transport/lanes.rs`

Completion criteria:
- No gameplay policy logic in QUIC adapter.
- Adapter passes existing runtime tests with new contracts.

### Phase 5 - Cavern Hunt Migration Step 1: Introduce New Net Structure
Goal:
Create target game-net layout without switching runtime path yet.

Changes:
- Introduce `driver.rs`, `apply.rs`, `input.rs`, `components.rs`, `config.rs`, `smoothing.rs` as target shape.
- Keep existing cavern runtime path active.
- Bridge existing schema/components into new files progressively.

Affected files/modules:
- `games/cavern_hunt/src/net/mod.rs`
- `games/cavern_hunt/src/net/driver.rs` (new)
- `games/cavern_hunt/src/net/apply.rs`
- `games/cavern_hunt/src/net/input.rs` (new)
- `games/cavern_hunt/src/net/components.rs` (new)
- `games/cavern_hunt/src/net/config.rs`
- `games/cavern_hunt/src/net/smoothing.rs`
- `games/cavern_hunt/src/net/replication_intent/components.rs` (migration source)

Completion criteria:
- New structure compiles and is testable.
- Legacy cavern runtime path still drives production behavior.

### Phase 6 - Cavern Hunt Migration Step 2: Switch Runtime to New Structure
Goal:
Make Cavern Hunt runtime use split drivers and `NetPlugin<TDriver>` path.

Changes:
- Wire `CavernReplicationDriver` + `SnapshotApplyDriver` + `InputDriver`.
- Switch game plugin wiring to net plugin integration.
- Keep legacy files present but unused for rollback safety during this phase.

Affected files/modules:
- `games/cavern_hunt/src/app/internal/plugin_wiring.rs`
- `games/cavern_hunt/src/net/driver.rs`
- `games/cavern_hunt/src/net/apply.rs`
- `games/cavern_hunt/src/net/input.rs`
- `games/cavern_hunt/src/net/runtime.rs` (decommissioned usage)
- `engine/src/plugins/net/plugin.rs`

Completion criteria:
- Cavern multiplayer runs through new structure.
- Full snapshot synchronization passes live tests.

### Phase 7 - App and Integration Migration
Goal:
Move all app entrypoints and integration tests to new plugin API before deletions.

Changes:
- Migrate client app to `NetPlugin::<CavernReplicationDriver>::client()`.
- Migrate server app to `NetPlugin::<CavernReplicationDriver>::server()`.
- Update host/integration tests to `NetPlugin::<...>::host()` where needed.
- Remove reliance on legacy plugin names in app wiring.

Affected files/modules:
- `apps/grotto_client/src/main.rs`
- `apps/grotto_server/src/main.rs`
- `engine/tests/network_plugins.rs`
- `games/cavern_hunt/tests/live_multiplayer.rs`
- `games/cavern_hunt/tests/live_multiplayer/helpers.rs`

Completion criteria:
- Production apps run on new API.
- No mandatory runtime dependency on legacy plugin symbols.

### Phase 8 - Delta Re-enable and Interest Validation Gate
Goal:
Re-enable delta as default only after full snapshot parity is proven.

Changes:
- Enable delta pipeline as default in `engine_net`.
- Validate baseline fallback to full snapshot on mismatch.
- Verify interest policy behavior in replication pipeline order.

Affected files/modules:
- `net/engine_net/src/replication/timeline.rs`
- `net/engine_net/src/runtime/server.rs`
- `net/engine_net/src/runtime/client.rs`
- `net/engine_net/src/replication/interest.rs`
- `net/engine_net/src/protocol/snapshot.rs`

Completion criteria:
- Delta tests pass.
- Resync fallback tests pass.
- Full snapshot fallback remains correct under packet loss/baseline loss.

### Phase 9 - Cavern Legacy Removal and Final Cleanup
Goal:
Remove compatibility layers and obsolete runtime paths after migration is fully live.

Changes:
- Remove legacy plugin wrappers.
- Remove legacy monolithic driver adapter.
- Remove Cavern custom legacy runtime replication path.
- Update docs to final architecture.

Affected files/modules:
- `engine/src/plugins/net/plugin.rs` (remove wrappers)
- `net/engine_net/src/replication/*` (remove legacy adapter)
- `games/cavern_hunt/src/net/runtime.rs` (remove)
- `games/cavern_hunt/src/net/emit.rs` (remove or narrow to new role if still needed)
- `games/cavern_hunt/src/net/capture.rs` (remove or narrow)
- `engine/src/plugins/net/README.md`
- `net/engine_net/README.md`
- `net/engine_net_quic/README.md`
- `docs/roadmaps/ROADMAP.md`

Completion criteria:
- Only target APIs remain.
- No legacy runtime/plugin paths referenced by apps/tests.

## 6. Verification Gates
Run at every phase boundary:
- `cargo test -p engine_net`
- `cargo test -p engine_net_quic`
- `cargo test -p engine --test network_plugins`
- `cargo test -p cavern_hunt --tests`
- `cargo test -p cavern_hunt --test live_multiplayer`
- `cargo check -p grotto_client`
- `cargo check -p grotto_server`

Additional gates:
- Gate A (after Phase 2): full snapshot-only mode passes all multiplayer tests.
- Gate B (after Phase 6): Cavern Hunt runs fully on new driver/apply/input path.
- Gate C (after Phase 7): apps run on `NetPlugin<TDriver>` without legacy plugin usage.
- Gate D (after Phase 8): delta + resync fallback pass under simulated baseline loss.
- Gate E (before Phase 9 merge): no regressions in session lifecycle, host composition, and diagnostics exposure.

## 7. Risks and Mitigations
- Risk: large refactor causes prolonged breakage.
  - Mitigation: compatibility adapters and wrappers kept until apps are migrated.
- Risk: plugin regains semantic ownership.
  - Mitigation: phase gates require algorithm code to live in `engine_net`, plugin remains bridge-only.
- Risk: QUIC adapter leaks gameplay behavior.
  - Mitigation: enforce adapter-only scope and route semantics through `engine_net`.
- Risk: Cavern migration blocks runtime progress.
  - Mitigation: two-step game migration plus delayed deletion phase.
- Risk: delta path regresses correctness.
  - Mitigation: full snapshot parity gate before delta-default enablement.

## 8. Definition of Done
- `NetPlugin<TDriver>::client/server/host` is the only public net plugin model.
- `engine_net` owns replication/prediction/session semantics.
- `engine/src/plugins/net` is thin ECS bridge only.
- `engine_net_quic` is transport/runtime adapter only.
- Game networking follows split driver model and declarative metadata.
- Cavern Hunt legacy runtime path is removed.
- Full snapshot correctness and delta fallback behavior are both validated.
