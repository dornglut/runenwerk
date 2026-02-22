# Scheduler AGENTS Guidelines

## Scope
These rules apply to code in `scheduler/`.

## Engineering Rules
- Preserve deterministic behavior in execution ordering.
- Validate graph edits strictly; never silently ignore invalid configuration.
- Keep build/setup APIs side-effect free.
- Keep context generic and runtime-safe.

## Error Handling
- Prefer typed/fallible APIs over panic for expected misuse.
- Error messages must identify problematic node/edge names or IDs.

## Testing Rules
- Every scheduler behavior change requires tests in `scheduler/tests`.
- New validation path -> add failure test.
- Ordering logic change -> add deterministic order test.

## Performance Rules
- Keep per-node run overhead low.
- Avoid unnecessary allocations in `run()`.
- Separate graph rebuild work from per-frame execution.

## Documentation Rules
- Public API behavior and invariants must be reflected in `scheduler/docs/DESIGN_GOALS.md`.
