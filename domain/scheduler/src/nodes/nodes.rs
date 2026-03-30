use crate::Node;

macro_rules! define_dummy_nodes {
    ($($name:ident),* $(,)?) => {
        $(
            pub fn $name<C>() -> Node<C> {
                Node::new(stringify!($name), move |_ctx: &mut C| {
                    println!("Running {}", stringify!($name));
                    Ok(())
                })
            }
        )*
    };
}

// Usage:
define_dummy_nodes!(
    gpucompute_node,
    simulation_node,
    physics_node,
    input_node,
    aiupdate_node,
    chunk_window_node,
    lodupdate_node,
    render_node,
    network_node,
    audio_node,
);
