# engine_history

`engine_history` provides replay, checkpoint, and validation primitives for deterministic simulation history.

## Purpose

- Record per-tick command journal frames
- Record periodic world checkpoints
- Build replay archives
- Seek replay state by tick
- Validate replay outcomes with world hashes

## Module Layout

- `src/model.rs`:
  - `ReplayHeader`
  - `ReplayCheckpointMeta`
  - `ReplayCheckpoint<S>`
  - `ReplayJournalFrame<C>`
- `src/policy.rs`:
  - `CheckpointPolicy`
  - `ReplayStoragePolicy`
- `src/archive/mod.rs`:
  - `ReplayArchive<S, C>`
  - compressed encode/decode helpers
- `src/recorder/mod.rs`:
  - `ReplayRecorder<S, C>`
- `src/controller/mod.rs`:
  - `ReplayController<S, C>`
- `src/validation/mod.rs`:
  - `ReplayMismatch`
  - `ReplayValidationReport`

## Integration Role

`engine_history` is the history substrate used by runtime systems that need:

- reconnect recovery
- replay export/import
- deterministic divergence checks
- future rollback-oriented flows

It depends on `engine_sim` for shared simulation identity/hash vocabulary (`SimulationTick`, `SimulationHash`, `SimulationSessionId`, `SimulationSeed`).
