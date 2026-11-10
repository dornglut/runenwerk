use state_machine::{
    Lifecycle, LifecycleError, LifecycleModel, StateMachine, TransitionModel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Idle,
    Active,
}

enum Message {
    Activate(String),
    Deactivate,
    Invalid,
}

#[derive(Debug, PartialEq, Eq)]
struct Rejected;

struct Gameplay;

impl TransitionModel for Gameplay {
    type State = Mode;
    type Event = Message;
    type Error = Rejected;

    fn initial() -> Mode {
        Mode::Idle
    }

    fn next(from: Mode, event: &Message) -> Result<Mode, Rejected> {
        match (from, event) {
            (Mode::Idle, Message::Activate(name)) if !name.is_empty() => Ok(Mode::Active),
            (Mode::Active, Message::Deactivate) => Ok(Mode::Idle),
            _ => Err(Rejected),
        }
    }
}

#[test]
fn gameplay_machine_cycles_without_a_runtime_or_copyable_event() {
    let mut machine = StateMachine::<Gameplay>::new();
    assert_eq!(machine.transition(Message::Invalid), Err(Rejected));
    assert_eq!(machine.state(), Mode::Idle);
    assert!(machine
        .transition(Message::Activate("enemy-01".into()))
        .unwrap()
        .changed());
    machine.transition(Message::Deactivate).unwrap();
    assert_eq!(machine.state(), Mode::Idle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskState {
    Queued,
    Completed,
}

enum TaskEvent {
    Complete,
    Restart,
}

struct Task;

impl TransitionModel for Task {
    type State = TaskState;
    type Event = TaskEvent;
    type Error = Rejected;

    fn initial() -> TaskState {
        TaskState::Queued
    }

    fn next(from: TaskState, event: &TaskEvent) -> Result<TaskState, Rejected> {
        match (from, event) {
            (TaskState::Queued, TaskEvent::Complete) => Ok(TaskState::Completed),
            (TaskState::Completed, TaskEvent::Restart) => Ok(TaskState::Queued),
            _ => Err(Rejected),
        }
    }
}

impl LifecycleModel for Task {
    fn is_terminal(state: TaskState) -> bool {
        state == TaskState::Completed
    }
}

#[test]
fn lifecycle_rejects_reactivation_even_if_policy_would_allow_it() {
    let mut lifecycle = Lifecycle::<Task>::new();
    assert!(!lifecycle.is_terminal());
    lifecycle.transition(TaskEvent::Complete).unwrap();
    assert!(lifecycle.is_terminal());
    assert_eq!(
        lifecycle.transition(TaskEvent::Restart),
        Err(LifecycleError::Terminal)
    );
    assert_eq!(lifecycle.state(), TaskState::Completed);
}

#[test]
fn unwind_in_reducer_leaves_machine_unchanged() {
    struct Panicking;
    impl TransitionModel for Panicking {
        type State = Mode;
        type Event = ();
        type Error = Rejected;

        fn initial() -> Mode {
            Mode::Idle
        }

        fn next(_: Mode, _: &()) -> Result<Mode, Rejected> {
            panic!("transition interrupted");
        }
    }

    let mut machine = StateMachine::<Panicking>::new();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = machine.transition(());
    }));
    assert!(caught.is_err());
    assert_eq!(machine.state(), Mode::Idle);
}
