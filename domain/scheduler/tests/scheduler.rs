use std::any::TypeId;
use std::sync::{Arc, Mutex};

use scheduler::{
    AccessKey, ConflictKind, ExecutionScheduler, Node, RegisteredSystem, ScheduleLabel,
    SchedulerBuilder, SystemAccess,
};

#[derive(Copy, Clone)]
struct Startup;

impl ScheduleLabel for Startup {
    fn name() -> &'static str {
        "Startup"
    }
}

#[derive(Copy, Clone)]
struct Update;

impl ScheduleLabel for Update {
    fn name() -> &'static str {
        "Update"
    }
}

struct A;
struct B;
struct C;
struct D;

fn push_node(log: Arc<Mutex<Vec<String>>>, name: &'static str) -> Node<()> {
    Node::new(name, move |_ctx: &mut ()| {
        log.lock()
            .expect("log lock poisoned")
            .push(name.to_string());
        Ok(())
    })
}

fn noop_node(name: &'static str) -> Node<()> {
    Node::new(name, |_ctx: &mut ()| Ok(()))
}

#[test]
fn runs_nodes_in_dependency_order() {
    let log = Arc::new(Mutex::new(Vec::new()));

    let mut scheduler = SchedulerBuilder::<()>::new()
        .add_node("input", push_node(log.clone(), "input"))
        .add_node_with_edges(
            "simulation",
            push_node(log.clone(), "simulation"),
            &["input"],
        )
        .add_node_with_edges("render", push_node(log.clone(), "render"), &["simulation"])
        .build()
        .expect("scheduler should build");

    let mut ctx = ();
    scheduler
        .run(&mut ctx)
        .expect("scheduler run should succeed");

    let entries = log.lock().expect("log lock poisoned").clone();
    assert_eq!(entries, vec!["input", "simulation", "render"]);
}

#[test]
fn detects_cycles_when_running() {
    let mut scheduler = SchedulerBuilder::<()>::new()
        .add_node("a", noop_node("a"))
        .add_node("b", noop_node("b"))
        .add_edge("a", "b")
        .add_edge("b", "a")
        .build()
        .expect("scheduler should build even with cycle; cycle is validated at run time");

    let mut ctx = ();
    let err = scheduler
        .run(&mut ctx)
        .expect_err("cycle should cause scheduler run failure");
    let message = format!("{err:#}");
    assert!(
        message.contains("Cycle detected"),
        "unexpected error: {message}"
    );
}

#[test]
fn builder_fails_on_unknown_dependency() {
    let result = SchedulerBuilder::<()>::new()
        .add_node("a", noop_node("a"))
        .add_edge("a", "missing")
        .build();
    let err = match result {
        Ok(_) => panic!("build should fail when dependency refers to unknown node"),
        Err(err) => err,
    };

    let message = format!("{err:#}");
    assert!(
        message.contains("Unknown dependency target node 'missing'"),
        "unexpected error: {message}"
    );
}

#[test]
fn builder_fails_on_duplicate_node_name() {
    let result = SchedulerBuilder::<()>::new()
        .add_node("a", noop_node("a"))
        .add_node("a", noop_node("a_2"))
        .build();
    let err = match result {
        Ok(_) => panic!("build should fail on duplicate node names"),
        Err(err) => err,
    };

    let message = format!("{err:#}");
    assert!(
        message.contains("Duplicate node name 'a'"),
        "unexpected error: {message}"
    );
}

#[test]
fn scheduler_runs_systems_in_schedule_order() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut scheduler = ExecutionScheduler::<Vec<String>>::new();

    scheduler.add_system(
        RegisteredSystem::new::<Startup>("boot", SystemAccess::new(), {
            let log = log.clone();
            move |ctx: &mut Vec<String>| {
                ctx.push("boot".to_string());
                log.lock().unwrap().push("boot".to_string());
                Ok(())
            }
        })
        .unwrap(),
    );
    scheduler.add_system(
        RegisteredSystem::new::<Update>("simulate", SystemAccess::new(), {
            let log = log.clone();
            move |ctx: &mut Vec<String>| {
                ctx.push("simulate".to_string());
                log.lock().unwrap().push("simulate".to_string());
                Ok(())
            }
        })
        .unwrap(),
    );
    scheduler.add_system(
        RegisteredSystem::new::<Update>("render", SystemAccess::new(), {
            let log = log.clone();
            move |ctx: &mut Vec<String>| {
                ctx.push("render".to_string());
                log.lock().unwrap().push("render".to_string());
                Ok(())
            }
        })
        .unwrap(),
    );

    let mut ctx = Vec::new();
    scheduler.run_schedule::<Startup>(&mut ctx).unwrap();
    scheduler.run_schedule::<Update>(&mut ctx).unwrap();

    assert_eq!(ctx, vec!["boot", "simulate", "render"]);
    assert_eq!(
        log.lock().unwrap().clone(),
        vec!["boot", "simulate", "render"]
    );
}

#[test]
fn scheduler_records_conflicts_but_stays_serial() {
    let mut scheduler = ExecutionScheduler::<Vec<String>>::new();
    let conflicting_access = SystemAccess::new().with_write(AccessKey::resource::<u32>("counter"));

    scheduler.add_system(
        RegisteredSystem::new::<Update>(
            "first",
            conflicting_access.clone(),
            |ctx: &mut Vec<String>| {
                ctx.push("first".to_string());
                Ok(())
            },
        )
        .unwrap(),
    );
    scheduler.add_system(
        RegisteredSystem::new::<Update>("second", conflicting_access, |ctx: &mut Vec<String>| {
            ctx.push("second".to_string());
            Ok(())
        })
        .unwrap(),
    );

    let plan = scheduler.plan_for::<Update>().unwrap().clone();
    assert_eq!(plan.conflicts.len(), 1);
    assert_eq!(plan.conflicts[0].first_system, "first");
    assert_eq!(plan.conflicts[0].second_system, "second");

    let mut ctx = Vec::new();
    scheduler.run_schedule::<Update>(&mut ctx).unwrap();
    assert_eq!(ctx, vec!["first", "second"]);
}

#[test]
fn same_system_mixed_intents_on_queue_are_rejected() {
    let work_queue_key = AccessKey::work_queue::<u32>("queue");

    let result = RegisteredSystem::new::<Update>(
        "invalid_mixed_intent",
        SystemAccess::new()
            .with_read(work_queue_key)
            .with_drain(work_queue_key),
        |_ctx: &mut Vec<String>| Ok(()),
    );
    let err = match result {
        Ok(_) => panic!("mixed read+drain intent should fail validation"),
        Err(err) => err,
    };

    let message = format!("{err:#}");
    assert!(message.contains("conflicting access"), "{message}");
    assert!(
        message.contains("read/drain conflict on work queue"),
        "{message}"
    );
}

#[test]
fn multiple_resource_conflicts_are_reported_in_registration_order() {
    let first = AccessKey::resource::<A>("same_name");
    let second = AccessKey::resource::<B>("same_name");
    let left = SystemAccess::new().with_write(first).with_write(second);
    let right = SystemAccess::new().with_read(first).with_read(second);

    for _ in 0..16 {
        let conflicts = left.conflicts_with(&right);
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].kind, ConflictKind::ReadWrite);
        assert_eq!(conflicts[0].key.type_id(), Some(TypeId::of::<A>()));
        assert_eq!(conflicts[1].kind, ConflictKind::ReadWrite);
        assert_eq!(conflicts[1].key.type_id(), Some(TypeId::of::<B>()));
    }
}

#[test]
fn same_name_conflicts_do_not_collapse_or_swap() {
    let first = AccessKey::broadcast_stream::<A>("same_stream");
    let second = AccessKey::broadcast_stream::<B>("same_stream");
    let left = SystemAccess::new().with_write(first).with_write(second);
    let right = SystemAccess::new().with_write(first).with_write(second);

    let conflicts = left.conflicts_with(&right);
    assert_eq!(conflicts.len(), 2);
    assert_eq!(conflicts[0].key.type_id(), Some(TypeId::of::<A>()));
    assert_eq!(conflicts[1].key.type_id(), Some(TypeId::of::<B>()));
    assert!(
        conflicts
            .iter()
            .all(|conflict| conflict.diagnostic_message().contains("broadcast stream"))
    );
}

#[test]
fn work_queue_conflicts_sort_by_domain_mode_and_registration_order() {
    let queue_a = AccessKey::work_queue::<A>("queue_a");
    let queue_b = AccessKey::work_queue::<B>("queue_b");
    let left = SystemAccess::new()
        .with_read(queue_b)
        .with_write(queue_a)
        .with_drain(queue_b)
        .with_drain(queue_a);
    let right = SystemAccess::new()
        .with_drain(queue_b)
        .with_drain(queue_a)
        .with_write(queue_b);

    let conflicts = left.conflicts_with(&right);
    let summary: Vec<_> = conflicts
        .iter()
        .map(|conflict| {
            (
                conflict.kind,
                conflict.key.type_id(),
                conflict.diagnostic_message(),
            )
        })
        .collect();

    assert_eq!(
        summary,
        vec![
            (
                ConflictKind::ReadWrite,
                Some(TypeId::of::<B>()),
                "read/write conflict on work queue 'queue_b'".to_string(),
            ),
            (
                ConflictKind::ReadDrain,
                Some(TypeId::of::<B>()),
                "read/drain conflict on work queue 'queue_b'".to_string(),
            ),
            (
                ConflictKind::WriteDrain,
                Some(TypeId::of::<B>()),
                "write/drain conflict on work queue 'queue_b'".to_string(),
            ),
            (
                ConflictKind::WriteDrain,
                Some(TypeId::of::<A>()),
                "write/drain conflict on work queue 'queue_a'".to_string(),
            ),
            (
                ConflictKind::DrainDrain,
                Some(TypeId::of::<B>()),
                "drain/drain conflict on work queue 'queue_b'".to_string(),
            ),
            (
                ConflictKind::DrainDrain,
                Some(TypeId::of::<A>()),
                "drain/drain conflict on work queue 'queue_a'".to_string(),
            ),
        ]
    );
}

#[test]
fn tick_buffer_and_broadcast_conflict_wording_and_order_are_stable() {
    let broadcast = AccessKey::broadcast_stream::<C>("damage");
    let tick_buffer = AccessKey::tick_buffer::<D>("input");
    let left = SystemAccess::new()
        .with_write(tick_buffer)
        .with_write(broadcast);
    let right = SystemAccess::new()
        .with_drain(tick_buffer)
        .with_read(broadcast);

    let conflicts = left.conflicts_with(&right);
    let messages: Vec<_> = conflicts
        .iter()
        .map(|conflict| conflict.diagnostic_message())
        .collect();

    assert_eq!(
        messages,
        vec![
            "read/write conflict on broadcast stream 'damage'",
            "write/drain conflict on tick buffer 'input'",
        ]
    );
}

#[test]
fn validate_internal_reports_deterministic_conflict_when_multiple_exist() {
    let resource = AccessKey::resource::<A>("resource");
    let queue = AccessKey::work_queue::<B>("queue");
    let tick_buffer = AccessKey::tick_buffer::<C>("input");
    let access = SystemAccess::new()
        .with_read(tick_buffer)
        .with_read(resource)
        .with_write(resource)
        .with_drain(queue)
        .with_write(queue)
        .with_drain(tick_buffer);

    for _ in 0..16 {
        let conflict = access
            .validate_internal()
            .expect_err("multiple internal conflicts should be rejected");
        assert_eq!(conflict.kind, ConflictKind::ReadWrite);
        assert_eq!(conflict.key.type_id(), Some(TypeId::of::<A>()));
        assert_eq!(
            conflict.diagnostic_message(),
            "read/write conflict on resource 'resource'"
        );
    }
}

#[test]
fn conflict_matrix_covers_broadcast_queue_and_tick_buffer_domains() {
    fn kind(left: SystemAccess, right: SystemAccess) -> Option<scheduler::ConflictKind> {
        left.conflicts_with(&right)
            .first()
            .map(|conflict| conflict.kind)
    }

    let broadcast = AccessKey::broadcast_stream::<u32>("broadcast");
    assert_eq!(
        kind(
            SystemAccess::new().with_read(broadcast),
            SystemAccess::new().with_read(broadcast),
        ),
        None
    );
    assert_eq!(
        kind(
            SystemAccess::new().with_read(broadcast),
            SystemAccess::new().with_write(broadcast),
        ),
        Some(scheduler::ConflictKind::ReadWrite)
    );
    assert_eq!(
        kind(
            SystemAccess::new().with_write(broadcast),
            SystemAccess::new().with_write(broadcast),
        ),
        Some(scheduler::ConflictKind::WriteWrite)
    );

    let queue = AccessKey::work_queue::<u32>("queue");
    assert_eq!(
        kind(
            SystemAccess::new().with_read(queue),
            SystemAccess::new().with_drain(queue),
        ),
        Some(scheduler::ConflictKind::ReadDrain)
    );
    assert_eq!(
        kind(
            SystemAccess::new().with_write(queue),
            SystemAccess::new().with_drain(queue),
        ),
        Some(scheduler::ConflictKind::WriteDrain)
    );
    assert_eq!(
        kind(
            SystemAccess::new().with_drain(queue),
            SystemAccess::new().with_drain(queue),
        ),
        Some(scheduler::ConflictKind::DrainDrain)
    );

    let input = AccessKey::tick_buffer::<u32>("tick_buffer");
    assert_eq!(
        kind(
            SystemAccess::new().with_read(input),
            SystemAccess::new().with_write(input),
        ),
        Some(scheduler::ConflictKind::ReadWrite)
    );
    assert_eq!(
        kind(
            SystemAccess::new().with_read(input),
            SystemAccess::new().with_drain(input),
        ),
        Some(scheduler::ConflictKind::ReadDrain)
    );
    assert_eq!(
        kind(
            SystemAccess::new().with_write(input),
            SystemAccess::new().with_drain(input),
        ),
        Some(scheduler::ConflictKind::WriteDrain)
    );
}

#[test]
fn scheduler_assigns_monotonic_ids_and_surfaces_them_in_plans() {
    let mut scheduler = ExecutionScheduler::<Vec<String>>::new();

    scheduler.add_system(
        RegisteredSystem::new::<Update>("a", SystemAccess::new(), |_ctx: &mut Vec<String>| Ok(()))
            .unwrap(),
    );
    scheduler.add_system(
        RegisteredSystem::new::<Update>("b", SystemAccess::new(), |_ctx: &mut Vec<String>| Ok(()))
            .unwrap(),
    );

    let ids: Vec<u64> = scheduler
        .systems()
        .iter()
        .map(|system| system.id().as_raw())
        .collect();
    assert_eq!(ids, vec![0, 1]);

    let plan = scheduler.plan_for::<Update>().unwrap();
    let stage_ids: Vec<u64> = plan
        .stages
        .iter()
        .flat_map(|stage| stage.system_ids.iter().map(|id| id.as_raw()))
        .collect();
    assert_eq!(stage_ids, vec![0, 1]);
}
