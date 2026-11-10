# State machine foundation

`TransitionModel` describes a pure typed transition policy; `StateMachine<M>`
holds state and applies transitions. `LifecycleModel` adds terminal-state
semantics; `Lifecycle<M>` prevents further transitions after termination.

This crate does not define universal state names, call hooks, manage resources,
schedule ECS systems, dispatch events, or provide a command bus. Rejected
transitions never change state. No `Send` or `Sync` bounds are imposed.

App is the first production consumer. A separate gameplay-shaped test proves
the mechanism is not tied to App. Production Window/Job models must be
independently qualified against their owning runtimes before migration.

Accepted Runenwerk ADR 0023 preserves owner semantics and ADR 0026 describes
future App shutdown behavior. This initial refactor preserves *current*
Configuring/Prepared/Starting/Running/Failed execution, including one-shot
Startup failure and interruption protection; it does not implement terminal
Host/Render/UI settlement.

Validate via `cargo test -p state_machine --all-targets`, followed by
the repository's canonical `cargo validate` and exact-head review.
