# engine_replay

`engine_replay` provides the shared replay/checkpoint foundation for the runtime.

## Purpose

- Record periodic checkpoints
- Record per-tick journal frames
- Store replay archives
- Seek and validate replay data
- Provide a reusable base for rewind, reconnect recovery, and future rollback profiles

## Core Types

- `ReplayHeader`
- `ReplayArchive`
- `ReplayCheckpoint`
- `ReplayJournalFrame`
- `ReplayRecorder`
- `ReplayController`
- `CheckpointPolicy`
- `ReplayStoragePolicy`
- `ReplayValidationReport`

## Current Role

The engine uses this crate through `ReplayPlugin` and the scene simulation codec. The replay path is already active for the authoritative scene simulation and is part of the current dedicated-authority runtime foundation.
