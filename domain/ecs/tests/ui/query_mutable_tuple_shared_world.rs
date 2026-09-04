#[derive(ecs::Component)]
struct A;

#[derive(ecs::Component)]
struct B;

fn main() {
    let world = ecs::World::new();
    let state = world.query_state::<(&mut A, &B), ()>();
    let _items = state.iter(&world);
}
