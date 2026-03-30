use ecs::prelude::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct A(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct B(i32);

#[test]
fn same_type_double_mut_query_panics() {
    let mut world = World::new();
    world.spawn(A(1));

    let query = world.query_state::<(&mut A, &mut A), ()>();
    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        let _ = query.iter(&mut world).next();
    }));
    assert!(panic_result.is_err());
}

#[test]
fn same_type_mut_read_query_panics() {
    let mut world = World::new();
    world.spawn(A(1));

    let query = world.query_state::<(&mut A, &A), ()>();
    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        let _ = query.iter(&mut world).next();
    }));
    assert!(panic_result.is_err());
}

#[test]
fn optional_mut_query_handles_present_absent_and_repeated_iteration() {
    let mut world = World::new();
    let with_b = world.spawn((A(1), B(10)));
    let without_b = world.spawn(A(2));

    let query = world.query_state::<(&mut A, Option<&mut B>), ()>();

    for (a, maybe_b) in query.iter(&mut world) {
        a.0 += 1;
        if let Some(b) = maybe_b {
            b.0 += a.0;
        }
    }

    for (a, maybe_b) in query.iter(&mut world) {
        a.0 += 1;
        if let Some(b) = maybe_b {
            b.0 += a.0;
        }
    }

    assert_eq!(world.require::<A>(with_b).unwrap().0, 3);
    assert_eq!(world.require::<A>(without_b).unwrap().0, 4);
    assert_eq!(world.require::<B>(with_b).unwrap().0, 15);
}
