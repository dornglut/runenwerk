# State machine and lifecycle foundation

## The single authoring style

A state machine defines **one** closure for its domain's transition rules.
A lifecycle uses the **same** closure and adds a predicate for terminality.

```rust
use state_machine::{InvalidTransition, Next, StateMachine};

#[derive(Debug, PartialEq, Eq)]
enum EnemyState {
    Idle,
    Chasing { target: u64 },
}

enum EnemyEvent {
    Detect(u64),
    LoseTarget,
}

let mut enemy = StateMachine::new(EnemyState::Idle)
    .transitions(|state, event: &EnemyEvent| match (state, event) {
        (EnemyState::Idle, EnemyEvent::Detect(target)) =>
            Ok(Next::Enter(EnemyState::Chasing { target: *target })),
        (EnemyState::Chasing { .. }, EnemyEvent::LoseTarget) =>
            Ok(Next::Enter(EnemyState::Idle)),
        _ => Err(InvalidTransition),
    });

enemy.transition(EnemyEvent::Detect(42))?;
assert!(matches!(enemy.state(), EnemyState::Chasing { target: 42 }));
# Ok::<(), InvalidTransition>(())
```

No `impl TransitionState`, zero-sized policy marker, macro-generated
transition table, or custom builder language. The closure is **the only**
policy authority. States and events are ordinary Rust types.

## Lifecycle uses the same authoring syntax

```rust
use state_machine::{InvalidTransition, Next, StateMachine};

#[derive(Debug, PartialEq, Eq)]
enum ServiceState {
    Running,
    Failed,
    Stopping,
    Terminated,
}

enum ServiceEvent {
    Fail,
    Stop,
    Settled,
}

let mut service = StateMachine::new(ServiceState::Running)
    .transitions(|state, event: &ServiceEvent| match (state, event) {
        (ServiceState::Running, ServiceEvent::Fail) =>
            Ok(Next::Enter(ServiceState::Failed)),
        (ServiceState::Running | ServiceState::Failed, ServiceEvent::Stop) =>
            Ok(Next::Enter(ServiceState::Stopping)),
        (ServiceState::Stopping, ServiceEvent::Settled) =>
            Ok(Next::Enter(ServiceState::Terminated)),
        _ => Err(InvalidTransition),
    })
    .terminal_when(|state| matches!(state, ServiceState::Terminated));

service.transition(ServiceEvent::Fail)?;
assert!(!service.is_terminal()); // Failure is not completed termination.
service.transition(ServiceEvent::Stop)?;
service.transition(ServiceEvent::Settled)?;
assert!(service.is_terminal());
# Ok::<(), state_machine::LifecycleError<InvalidTransition>>(())
```

`terminal_when` **consumes** the configured machine and returns a
`Lifecycle` that retains its state and transition closure. Once genuinely
terminal, every event is rejected with `LifecycleError::Terminal`
**before** evaluating the transition policy. This does not start shutdown,
perform cleanup, settle work or schedule anything.

Do not mark `Failed`, `Stopping`, or `StopRequested` terminal if their
owner still has legal transitions or cleanup obligations.

## Core operations

| Operation | Meaning |
| --- | --- |
| `StateMachine::new(initial)` | Unchecked, owner-supplied initial state |
| `.transitions(|state, event| ...)` | Defines one statically typed policy |
| `.transition(event)` | Accepts a borrowed or owned event and returns an outcome |
| `.apply(event)` | Returns `&mut Self` for fluent execution |
| `.state()` | Borrow current domain state |
| `.into_state()` | Consume the machine and recover owned state |
| `.terminal_when(|state| ...)` | Convert into a terminally guarded `Lifecycle` |
| `.is_terminal()` | Observe a lifecycle's domain-defined finality |

`transition` reports `TransitionOutcome::Stayed` for an accepted
`Next::Stay`, or `TransitionOutcome::Replaced` for a committed
`Next::Enter(next)`. These names describe **commit behavior**, not
`PartialEq`: replacing a state with an equal value still reports
`Replaced`. A rejected event returns the domain error, not `Stayed`.

`apply` deliberately does **not** form a multi-event transaction:
```rust,ignore
machine
    .apply(Event::Prepare)?
    .apply(Event::Start)?;
```
If `Start` fails, the successful `Prepare` remains committed. Use
`transition` when callers need `Stayed` versus `Replaced`.

## Owned payloads, errors and configuration

- State and event payloads need not be `Copy`, `Clone`, `Eq`, or `Default`.
- A policy takes `&State` and `&Event`; it returns an owned `Next<State>`.
  Copy or clone payloads **only if the domain needs to carry them forward**.
  This contract does not promise zero-copy payload transfer.
- A simple machine may reject with the provided `InvalidTransition`.
  Richer domains return their own typed `Error`; error types are inferred
  from the closure's result.
- Closures can capture immutable configuration. The policy is stored by
  value and dispatched statically; the foundation itself does not allocate
  or require a `dyn Fn` trait object.
- `Fn` prevents directly mutable captured variables but does **not**
  prohibit interior mutability or external side effects. Authors must
  keep policy evaluation free of owner effects that require rollback.
- `StateMachine::new(initial)` does **not** validate persisted state or
  correlated fields. Domains validate restoration before construction.

## Storing a reusable machine as a struct field

A closure has an unnameable concrete type. When no runtime configuration
needs capturing, a local function-pointer type is a straightforward way
to name the **same closure-authored policy** in a private struct field:

```rust
use state_machine::{InvalidTransition, Next, StateMachine};

#[derive(Clone, Copy)]
enum Phase {
    Idle,
    Running,
}
enum Event {
    Start,
}
type Rules = fn(&Phase, &Event) -> Result<Next<Phase>, InvalidTransition>;
type Machine = StateMachine<Phase, Event, InvalidTransition, Rules>;

struct Session {
    machine: Machine,
}

impl Session {
    fn new() -> Self {
        let rules: Rules = |state, event| match (state, event) {
            (Phase::Idle, Event::Start) => Ok(Next::Enter(Phase::Running)),
            _ => Err(InvalidTransition),
        };
        Self {
            machine: StateMachine::new(Phase::Idle).transitions(rules),
        }
    }
}

let mut session = Session::new();
session.machine.transition(Event::Start)?;
# Ok::<(), InvalidTransition>(())
```

A function pointer does not capture configuration. If a machine must hold a
capturing closure, store the resulting concrete generic machine inside a
generic domain owner. The authoring expression stays the same; do not
silently allocate or erase the policy merely to simplify a field type.
In ECS, the owning component's `Send`, `Sync` and `'static` constraints
must be satisfied by the captured policy and state as required by that ECS.

## Failure, ownership and cleanup guarantees

A rejected event or panic **inside the closure** does not change the
machine's state. When `Next::Enter` succeeds, the replacement is stored
before the previous state is dropped. A panic from the previous state's
destructor therefore leaves the **replacement** committed, even though
the caller observes a panic. Avoid fallible side effects in destructors.
No external effect is made atomic or rolled back by the machine.

`StateMachine` supports conditional `Clone` only when its state and
policy can be cloned (useful for deliberate simulation snapshots);
`Lifecycle` is intentionally not `Clone` because it represents one
authoritative lifetime. Neither type provides unrestricted mutable
state access or a `set_state` bypass. Errors, events and terminal
predicates are domain-owned—not global engine policy.

## Production integrations

- **Engine App**: `AppLifecycle` wraps
  `StateMachine::new(Configuring).transitions(|state, event| ...)`.
  One policy defines idempotent Prepare, one-shot Startup admission,
  returned-error `Failed` and panic-left-`Starting` semantics.
  The App exposes named domain operations. Current `Failed` is
  **non-runnable, not physically terminated**.
- **Runenwerk Draw**: `DrawingToolSession` wraps a private state containing
  both its session ID counter and its data-bearing phase, plus the same
  `.transitions(|state, event| ...)` authoring syntax. It commits
  correlated identity and phase together; input dispatch and drawing
  effects remain app-owned.

Neither integration gives the foundation authority over ECS, App Startup
execution, Host identity, UI/window creation, rendering, GPU progress,
job/resource settlement, or lifecycle callbacks.

## Design choices and rejected alternatives

- Mandatory `TransitionState`/`LifecycleState` implementations: too
  much repeated associated-type and implementation boilerplate.
- A second authoring macro/builder DSL: would fracture the single mental
  model and obscure data-bearing guarded rules.
- Boxed policy as the only construction route: unnecessary allocation and
  dynamic dispatch for ordinary models.
- `FnMut` policy: directly permits hidden mutable transition history.
  Use a domain state field and `Fn` rules instead.
- Generic automatic entry/exit callbacks or event scheduling: would
  silently transfer domain execution and resource ownership.
- A universal terminal state or failed-is-terminal shortcut: conflicts
  with accepted App/Host lifetime distinctions.

## Validation

Compile-test the simple closure API, capturing policy, non-Copy states
and events, typed errors, value replacement, idempotence, rejection,
destructor/reducer panics, terminal finality, stored function-pointer
policies and both production integrations. The canonical
`cargo +stable validate` and exact-head native/GPU/documentation CI
must pass before critical review and protected acceptance of PR #1477.
