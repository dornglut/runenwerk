# State machine and lifecycle foundation

## Purpose

Typed, synchronous **state transition authoring** without an auxiliary
policy-marker type, lifecycle executor, global event bus, or callback runtime.
Authors define a state, its domain events and one transition function.

- `TransitionState::next(&self, &Event)` decides `Next::Stay`,
  `Next::Enter(next)`, or `Err(domain_error)`.
- `StateMachine<State>` stores the state, commits accepted updates atomically
  from the perspective of its own state, and reports `Transition::Changed`
  or `Unchanged`.
- `LifecycleState` states when a **lifetime has actually settled**.
  `Lifecycle<State>` uses the same machine but blocks events after finality.

The generic layer never interprets App, Window, Render, gameplay, ECS,
system schedules, resource lifetimes or external effects. Domain-owned
external work is **not** rolled back or made atomic by this library.

## Author experience: small gameplay state

```rust
use state_machine::{Next, StateMachine, TransitionState};

#[derive(Debug, PartialEq)]
enum Enemy {
    Idle,
    Chasing { target: String },
}

enum EnemyEvent {
    Detect(String),
    LoseTarget,
}

impl TransitionState for Enemy {
    type Event = EnemyEvent;
    type Error = &'static str;

    fn next(&self, event: &Self::Event) -> Result<Next<Self>, Self::Error> {
        match (self, event) {
            (Self::Idle, EnemyEvent::Detect(target)) => Ok(
                Next::Enter(Self::Chasing { target: target.clone() })
            ),
            (Self::Chasing { .. }, EnemyEvent::LoseTarget) => Ok(Next::Enter(Self::Idle)),
            _ => Err("event is not valid in current state"),
        }
    }
}

let mut enemy = StateMachine::new(Enemy::Idle);
enemy.transition(EnemyEvent::Detect("player-1".into())).unwrap();
assert!(matches!(enemy.state(), Enemy::Chasing { .. }));
```

No `Copy`, `Clone`, `Eq`, or `Default` requirement is imposed by the
transition mechanism. `Next::Stay` is explicit and does not duplicate a
non-Copy state. An event can be supplied by value or borrowed; prefer
borrowing when the original is needed for diagnostics or replay. A
`Transition::Changed` result means the model committed a newly entered
logical state; it does **not** imply GPU, window, job or other physical
completion.

## Lifecycle: failed is not terminated

```rust
use state_machine::{Lifecycle, LifecycleState, Next, TransitionState};

#[derive(Debug)]
enum Work { Running, Failed, Stopping, Settled }
enum WorkEvent { Fail, Stop, Finish }

impl TransitionState for Work {
    type Event = WorkEvent;
    type Error = &'static str;

    fn next(&self, event: &WorkEvent) -> Result<Next<Self>, Self::Error> {
        match (self, event) {
            (Self::Running, WorkEvent::Fail) => Ok(Next::Enter(Self::Failed)),
            (Self::Running | Self::Failed, WorkEvent::Stop) => Ok(Next::Enter(Self::Stopping)),
            (Self::Stopping, WorkEvent::Finish) => Ok(Next::Enter(Self::Settled)),
            _ => Err("not allowed"),
        }
    }
}
impl LifecycleState for Work {
    fn is_terminal(&self) -> bool { matches!(self, Self::Settled) }
}
let mut work = Lifecycle::new(Work::Running);
work.transition(WorkEvent::Fail).unwrap();
assert!(!work.is_terminal()); // cleanup can still happen
work.transition(WorkEvent::Stop).unwrap();
work.transition(WorkEvent::Finish).unwrap();
assert!(work.is_terminal());
```

## Integration contracts

- **App** implements `TransitionState` directly on `AppLifecycleState`;
  its facade exposes named prepare/Startup methods and a typed
  `StartupAdmission` outcome. Current App states have no settled terminal
  state, so the facade uses `StateMachine` until owner-confirmed shutdown
  semantics justify `Lifecycle`. `Starting` is recorded **before**
  executing RunenECS Startup. Returned errors mark `Failed`, while
  unwinding leaves `Starting`; neither can be retried.
- **Runenwerk Draw** uses `StateMachine<DrawingToolSessionState>` around a
  private state that holds **both** session identity and its data-bearing
  active phase. They commit together rather than updating a separate
  counter before changing the phase. Public `DrawingToolSessionPhase` stays
  a read-only observation, while transition events stay private. Input
  routing, tool intents and drawing authority remain in the App.
  End is idempotent; Begin starts a new session.
- **Future terminal App** requires actual, owner-confirmed Host and
  resource settlement under accepted ADR 0026. Current `Failed` is not
  a terminal state.

## Correctness and ergonomics

`StateMachine::new(initial)` is **unchecked**: it trusts the initial
state supplied by the owning domain. A restoring domain must validate
persisted state and all correlated data before invoking the constructor.
The mechanism supports restoration but does not implement validation
or migration. The library intentionally has no `set_state` API.
`TransitionState::next` must not mutate external state; all fallible
checks should happen before owner effects. If a domain needs to change
several correlated fields, their owner must validate and commit the whole
record rather than modifying only one generic phase field.

A panic during `next` does not update machine state. An accepted new
state is installed before the previous state is dropped. If the old state's
destructor panics, the **new** state remains committed. Do not perform
fallible side effects in destructors; a panic after commitment is not a
rejected transition and does not roll back external effects.

`Lifecycle` is intentionally **not Clone**: copying a live lifetime would
create independent transition histories. Borrow or clone the state for
read-only projections instead. `StateMachine` remains conditionally Clone
for explicit simulation snapshots and rollback use cases.

The implementation adds no automatic concurrency or event scheduling.
Synchronization and ECS storage are separate owner concerns; `Send` /
`Sync` remain derived from the stored state, not universal requirements.

## Rejected alternatives

- Separate zero-sized policy markers: increased author types without a
  benefit for the existing App owner.
- Copy/Eq-only states and before/after snapshots on every transition:
  excluded actual drawing session payloads and forced unnecessary copies.
- Unrestricted transition setter: invalidates all domain transition rules.
- Universal Lifecycle phases: not all domain objects have the same lifetime.
- Derive macro, auto-executing callbacks, hierarchy, event bus and runtime
  graph: unproven additional machinery, not needed to author these models.

## Validation

Run `cargo test -p state_machine --all-targets`, relevant
`engine` App lifecycle and `runenwerk_draw` tests, then the canonical
`cargo +stable validate` and exact-head CI. Keep the PR draft for
independent source review and acceptance under issue #1476.
