use std::sync::{Arc, Mutex};

use scheduler::{
    AccessKey, ExecutionScheduler, Node, RegisteredSystem, ScheduleLabel, SchedulerBuilder,
    SystemAccess,
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
