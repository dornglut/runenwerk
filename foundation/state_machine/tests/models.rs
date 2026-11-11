use state_machine::{
    InvalidTransition, Lifecycle, LifecycleError, Next, StateMachine, TransitionOutcome,
};

#[derive(Debug, PartialEq)]
enum Mode {
    Idle,
    Active { name: String, weight: f32 },
}

#[derive(Debug)]
enum Message {
    Activate(String),
    Deactivate,
    Observe,
    Invalid,
}

#[derive(Debug, PartialEq, Eq)]
struct Rejected;

type ModeRule = fn(&Mode, &Message) -> Result<Next<Mode>, Rejected>;
type ModeMachine = StateMachine<Mode, Message, Rejected, ModeRule>;

fn mode_machine(initial: Mode) -> ModeMachine {
    let transitions: ModeRule = |state, event| match (state, event) {
        (Mode::Idle, Message::Activate(name)) if !name.is_empty() => {
            Ok(Next::Enter(Mode::Active {
                name: name.clone(),
                weight: 1.5,
            }))
        }
        (Mode::Active { .. }, Message::Deactivate) => Ok(Next::Enter(Mode::Idle)),
        (_, Message::Observe) => Ok(Next::Stay),
        _ => Err(Rejected),
    };

    StateMachine::new(initial).transitions(transitions)
}

#[test]
fn data_bearing_non_copy_state_and_event_are_supported() {
    let mut machine = mode_machine(Mode::Idle);
    let event = Message::Activate("enemy-01".into());

    assert_eq!(machine.transition(&event), Ok(TransitionOutcome::Replaced));
    assert_eq!(
        machine.state(),
        &Mode::Active {
            name: "enemy-01".into(),
            weight: 1.5,
        }
    );
    assert!(matches!(event, Message::Activate(_)));
    assert_eq!(
        machine.transition(Message::Observe),
        Ok(TransitionOutcome::Stayed)
    );
    assert_eq!(
        machine.transition(Message::Deactivate),
        Ok(TransitionOutcome::Replaced)
    );
    assert_eq!(machine.state(), &Mode::Idle);
}

#[test]
fn invalid_event_does_not_mutate_state_or_consume_borrowed_event() {
    let mut machine = mode_machine(Mode::Idle);
    let event = Message::Activate(String::new());
    assert_eq!(machine.transition(&event), Err(Rejected));
    assert!(matches!(event, Message::Activate(_)));
    assert_eq!(machine.transition(Message::Invalid), Err(Rejected));
    assert_eq!(machine.state(), &Mode::Idle);
}

#[test]
fn explicit_initial_state_and_owned_extraction_do_not_need_clone_or_copy() {
    let initial = Mode::Active {
        name: "restored".to_owned(),
        weight: 0.25,
    };
    let machine = mode_machine(initial);

    assert!(matches!(machine.state(), Mode::Active { .. }));
    assert_eq!(
        machine.into_state(),
        Mode::Active {
            name: "restored".to_owned(),
            weight: 0.25,
        }
    );
}

#[test]
fn policy_can_capture_immutable_configuration_without_boxing() {
    let permitted = ["enemy-42".to_owned()];
    let mut machine = StateMachine::new(Mode::Idle).transitions(
        move |state, event: &Message| -> Result<Next<Mode>, InvalidTransition> {
            match (state, event) {
                (Mode::Idle, Message::Activate(name)) if permitted.contains(name) => {
                    Ok(Next::Enter(Mode::Active {
                        name: name.clone(),
                        weight: 1.5,
                    }))
                }
                _ => Err(InvalidTransition),
            }
        },
    );

    assert_eq!(
        machine.transition(Message::Activate("other".to_owned())),
        Err(InvalidTransition)
    );
    assert_eq!(machine.state(), &Mode::Idle);
    assert_eq!(
        machine.transition(Message::Activate("enemy-42".to_owned())),
        Ok(TransitionOutcome::Replaced)
    );
    assert!(matches!(machine.state(), Mode::Active { .. }));
}

#[test]
fn fluent_apply_preserves_earlier_commits_when_a_later_event_fails() {
    let mut machine = mode_machine(Mode::Idle);
    let result = machine
        .apply(Message::Activate("enemy-01".into()))
        .and_then(|machine| machine.apply(Message::Invalid));
    assert!(matches!(result, Err(Rejected)));
    assert!(matches!(machine.state(), Mode::Active { .. }));
}

#[test]
fn enter_equal_value_reports_replacement_not_equality() {
    let mut machine = StateMachine::new(3_u32)
        .transitions(|state, _: &()| Ok::<Next<u32>, InvalidTransition>(Next::Enter(*state)));

    assert_eq!(machine.transition(()), Ok(TransitionOutcome::Replaced));
    assert_eq!(*machine.state(), 3);
}

#[derive(Debug, PartialEq, Eq)]
enum Job {
    Queued,
    Failed,
    Stopping,
    Settled,
}

enum JobEvent {
    Fail,
    RequestStop,
    FinishSettlement,
    Retry,
}

type JobRule = fn(&Job, &JobEvent) -> Result<Next<Job>, Rejected>;
type JobTerminal = fn(&Job) -> bool;
type JobLifetime = Lifecycle<Job, JobEvent, Rejected, JobRule, JobTerminal>;

fn job_lifecycle(initial: Job) -> JobLifetime {
    let transitions: JobRule = |state, event| match (state, event) {
        (Job::Queued, JobEvent::Fail) => Ok(Next::Enter(Job::Failed)),
        (Job::Queued | Job::Failed, JobEvent::RequestStop) => Ok(Next::Enter(Job::Stopping)),
        (Job::Stopping, JobEvent::FinishSettlement) => Ok(Next::Enter(Job::Settled)),
        _ => Err(Rejected),
    };

    let terminal: JobTerminal = |state| matches!(state, Job::Settled);

    StateMachine::new(initial)
        .transitions(transitions)
        .terminal_when(terminal)
}

#[test]
fn failed_lifecycle_is_not_terminal_and_can_be_settled() {
    let mut job = job_lifecycle(Job::Queued);
    job.transition(JobEvent::Fail).unwrap();
    assert!(!job.is_terminal());
    assert_eq!(
        job.transition(JobEvent::Retry),
        Err(LifecycleError::Rejected(Rejected))
    );
    job.apply(JobEvent::RequestStop)
        .unwrap()
        .apply(JobEvent::FinishSettlement)
        .unwrap();
    assert!(job.is_terminal());
    assert_eq!(
        job.transition(JobEvent::RequestStop),
        Err(LifecycleError::Terminal)
    );
    assert_eq!(job.state(), &Job::Settled);
}

#[test]
fn constructed_terminal_lifecycle_is_closed_immediately() {
    let mut job = job_lifecycle(Job::Settled);
    assert!(job.is_terminal());
    assert_eq!(
        job.transition(JobEvent::RequestStop),
        Err(LifecycleError::Terminal)
    );
    assert_eq!(job.into_state(), Job::Settled);
}

#[test]
fn terminal_guard_precedes_domain_rule_even_if_rule_would_reopen() {
    enum Phase {
        Open,
        Closed,
    }
    enum Operation {
        Close,
        Reopen,
    }

    let mut life = StateMachine::new(Phase::Open)
        .transitions(|state, event: &Operation| match (state, event) {
            (Phase::Open, Operation::Close) => Ok::<_, Rejected>(Next::Enter(Phase::Closed)),
            (Phase::Closed, Operation::Reopen) => Ok(Next::Enter(Phase::Open)),
            _ => Err(Rejected),
        })
        .terminal_when(|state| matches!(state, Phase::Closed));

    life.transition(Operation::Close).unwrap();
    assert_eq!(
        life.transition(Operation::Reopen),
        Err(LifecycleError::Terminal)
    );
}

#[test]
fn generic_lifecycle_predicate_can_capture_configuration() {
    let terminal_phase = 7_u32;
    let mut life = StateMachine::new(6_u32)
        .transitions(|state, _: &()| Ok::<_, InvalidTransition>(Next::Enter(*state + 1)))
        .terminal_when(move |state| *state >= terminal_phase);

    assert!(!life.is_terminal());
    assert_eq!(life.transition(()), Ok(TransitionOutcome::Replaced));
    assert!(life.is_terminal());
    assert_eq!(life.transition(()), Err(LifecycleError::Terminal));
}

#[test]
fn policy_panic_does_not_commit_a_partial_transition() {
    let mut machine = StateMachine::new("original".to_owned()).transitions(
        |_: &String, _: &()| -> Result<Next<String>, Rejected> {
            panic!("interrupted model transition")
        },
    );

    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = machine.transition(());
    }));

    assert!(caught.is_err());
    assert_eq!(machine.state(), "original");
}

#[test]
fn old_state_destructor_panic_leaves_new_state_committed() {
    use std::cell::Cell;
    use std::rc::Rc;

    struct Owned {
        generation: u32,
        panic_on_drop: bool,
        observed: Rc<Cell<u32>>,
    }

    impl Drop for Owned {
        fn drop(&mut self) {
            if self.panic_on_drop {
                self.panic_on_drop = false;
                self.observed.set(self.observed.get() + 1);
                panic!("old-state destructor");
            }
        }
    }

    let observed = Rc::new(Cell::new(0));
    let mut machine = StateMachine::new(Owned {
        generation: 0,
        panic_on_drop: true,
        observed: observed.clone(),
    })
    .transitions(|state, _: &()| {
        Ok::<_, Rejected>(Next::Enter(Owned {
            generation: state.generation + 1,
            panic_on_drop: false,
            observed: state.observed.clone(),
        }))
    });

    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = machine.transition(());
    }));

    assert!(caught.is_err());
    assert_eq!(machine.state().generation, 1);
    assert_eq!(observed.get(), 1);
}

#[test]
fn accepted_idempotent_event_does_not_copy_large_state() {
    struct LargeState {
        data: Vec<u8>,
    }

    let mut machine = StateMachine::new(LargeState {
        data: vec![0_u8; 8192],
    })
    .transitions(|_: &LargeState, _: &()| Ok::<_, Rejected>(Next::Stay));

    let ptr = machine.state().data.as_ptr();
    assert_eq!(machine.transition(()), Ok(TransitionOutcome::Stayed));
    assert_eq!(machine.state().data.as_ptr(), ptr);
}

#[test]
fn machine_can_clone_state_and_cloneable_policy_without_cloning_event() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Snapshot(u32);

    let mut machine = StateMachine::new(Snapshot(3))
        .transitions(|_: &Snapshot, _: &()| Ok::<_, Rejected>(Next::Stay));
    let clone = machine.clone();
    assert_eq!(machine.state(), clone.state());
    assert_eq!(machine.transition(()), Ok(TransitionOutcome::Stayed));
    assert!(format!("{machine:?}").contains("Snapshot(3)"));
}
