use state_machine::{
    Lifecycle, LifecycleError, LifecycleState, Next, StateMachine, Transition, TransitionState,
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

impl TransitionState for Mode {
    type Event = Message;
    type Error = Rejected;

    fn next(&self, event: &Message) -> Result<Next<Self>, Rejected> {
        match (self, event) {
            (Self::Idle, Message::Activate(name)) if !name.is_empty() => {
                Ok(Next::Enter(Self::Active {
                    name: name.clone(),
                    weight: 1.5,
                }))
            }
            (Self::Active { .. }, Message::Deactivate) => Ok(Next::Enter(Self::Idle)),
            (_, Message::Observe) => Ok(Next::Stay),
            _ => Err(Rejected),
        }
    }
}

#[test]
fn data_bearing_non_copy_state_and_event_are_supported() {
    let mut machine = StateMachine::new(Mode::Idle);
    let event = Message::Activate("enemy-01".into());
    assert_eq!(machine.transition(&event), Ok(Transition::Changed));
    assert_eq!(
        machine.state(),
        &Mode::Active {
            name: "enemy-01".into(),
            weight: 1.5
        }
    );
    // The caller retained its non-Copy event.
    assert!(matches!(event, Message::Activate(_)));
    assert_eq!(machine.transition(Message::Observe), Ok(Transition::Unchanged));
    assert_eq!(machine.transition(Message::Deactivate), Ok(Transition::Changed));
    assert_eq!(machine.state(), &Mode::Idle);
}

#[test]
fn invalid_event_does_not_modify_state_or_consume_borrowed_data() {
    let mut machine = StateMachine::new(Mode::Idle);
    let event = Message::Activate(String::new());
    assert_eq!(machine.transition(&event), Err(Rejected));
    assert!(matches!(event, Message::Activate(_)));
    assert_eq!(machine.state(), &Mode::Idle);
    assert_eq!(machine.transition(Message::Invalid), Err(Rejected));
}

#[test]
fn validated_restoration_and_owned_extraction_do_not_need_clone_or_copy() {
    let recovered = Mode::Active {
        name: "restored".to_owned(),
        weight: 0.25,
    };
    let machine = StateMachine::new(recovered);
    assert!(matches!(machine.state(), Mode::Active { .. }));
    assert_eq!(
        machine.into_state(),
        Mode::Active {
            name: "restored".to_owned(),
            weight: 0.25
        }
    );
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

impl TransitionState for Job {
    type Event = JobEvent;
    type Error = Rejected;

    fn next(&self, event: &JobEvent) -> Result<Next<Self>, Rejected> {
        match (self, event) {
            (Self::Queued, JobEvent::Fail) => Ok(Next::Enter(Self::Failed)),
            (Self::Queued | Self::Failed, JobEvent::RequestStop) => {
                Ok(Next::Enter(Self::Stopping))
            }
            (Self::Stopping, JobEvent::FinishSettlement) => Ok(Next::Enter(Self::Settled)),
            (Self::Failed, JobEvent::Retry) => Err(Rejected),
            _ => Err(Rejected),
        }
    }
}

impl LifecycleState for Job {
    fn is_terminal(&self) -> bool {
        matches!(self, Self::Settled)
    }
}

#[test]
fn failure_is_non_runnable_but_cleanup_remains_possible() {
    let mut job = Lifecycle::new(Job::Queued);
    job.transition(JobEvent::Fail).unwrap();
    assert!(!job.is_terminal());
    assert_eq!(job.transition(JobEvent::Retry), Err(LifecycleError::Rejected(Rejected)));
    job.transition(JobEvent::RequestStop).unwrap();
    assert!(!job.is_terminal());
    job.transition(JobEvent::FinishSettlement).unwrap();
    assert!(job.is_terminal());
    assert_eq!(job.transition(JobEvent::RequestStop), Err(LifecycleError::Terminal));
    assert_eq!(job.state(), &Job::Settled);
}

#[test]
fn terminal_lifecycle_rejects_every_event_even_if_domain_would_allow_one() {
    enum Reopen {
        Open,
        Closed,
    }
    enum Operation {
        Close,
        Reopen,
    }
    impl TransitionState for Reopen {
        type Event = Operation;
        type Error = Rejected;
        fn next(&self, event: &Operation) -> Result<Next<Self>, Rejected> {
            match (self, event) {
                (Self::Open, Operation::Close) => Ok(Next::Enter(Self::Closed)),
                (Self::Closed, Operation::Reopen) => Ok(Next::Enter(Self::Open)),
                _ => Err(Rejected),
            }
        }
    }
    impl LifecycleState for Reopen {
        fn is_terminal(&self) -> bool {
            matches!(self, Self::Closed)
        }
    }
    let mut lifetime = Lifecycle::new(Reopen::Open);
    lifetime.transition(Operation::Close).unwrap();
    assert_eq!(lifetime.transition(Operation::Reopen), Err(LifecycleError::Terminal));
}

#[test]
fn panic_in_domain_rule_does_not_commit_a_partial_state() {
    struct Panicking(String);
    impl TransitionState for Panicking {
        type Event = ();
        type Error = Rejected;
        fn next(&self, _: &()) -> Result<Next<Self>, Rejected> {
            panic!("interrupted domain transition")
        }
    }
    let mut machine = StateMachine::new(Panicking("original".into()));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = machine.transition(());
    }));
    assert!(caught.is_err());
    assert_eq!(machine.state().0, "original");
}

#[test]
fn duplicate_idempotent_events_do_not_copy_state() {
    struct LargeState {
        data: Vec<u8>,
    }
    impl TransitionState for LargeState {
        type Event = ();
        type Error = Rejected;
        fn next(&self, _: &()) -> Result<Next<Self>, Rejected> {
            Ok(Next::Stay)
        }
    }
    let mut machine = StateMachine::new(LargeState {
        data: vec![0u8; 8192],
    });
    let pointer = machine.state().data.as_ptr();
    assert_eq!(machine.transition(()), Ok(Transition::Unchanged));
    assert_eq!(machine.state().data.as_ptr(), pointer);
}

#[test]
fn conditional_trait_support_matches_the_contained_state() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Model(u32);
    impl TransitionState for Model {
        type Event = ();
        type Error = Rejected;
        fn next(&self, _: &()) -> Result<Next<Self>, Rejected> {
            Ok(Next::Stay)
        }
    }
    let instance = StateMachine::new(Model(3));
    let cloned = instance.clone();
    assert_eq!(instance, cloned);
    assert!(format!("{instance:?}").contains("Model(3)"));
}
