use std::sync::{Arc, Mutex};

use scheduler::{Node, SchedulerBuilder};

fn push_node(log: Arc<Mutex<Vec<String>>>, name: &'static str) -> Node<()> {
    Node::new(name, move |_ctx: &mut ()| {
        log.lock().expect("log lock poisoned").push(name.to_string());
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
        .add_node_with_edges("simulation", push_node(log.clone(), "simulation"), &["input"])
        .add_node_with_edges("render", push_node(log.clone(), "render"), &["simulation"])
        .build()
        .expect("scheduler should build");

    let mut ctx = ();
    scheduler.run(&mut ctx).expect("scheduler run should succeed");

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
    assert!(message.contains("Cycle detected"), "unexpected error: {message}");
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
