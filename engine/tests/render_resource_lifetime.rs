use engine::plugins::render::resource::{
    ResourceLifetime, build_transient_alias_assignments, build_transient_alias_slots,
    build_transient_windows, find_aliasable_transients,
};
use engine::plugins::render::{RenderFlow, RenderResourceDescriptor};

#[test]
fn descriptors_expose_imported_persistent_and_transient_lifetimes() {
    let imported = RenderResourceDescriptor::imported_texture("surface.color");
    assert_eq!(imported.lifetime(), ResourceLifetime::Imported);

    let persistent = RenderResourceDescriptor::color_target("main.color");
    assert_eq!(persistent.lifetime(), ResourceLifetime::Persistent);

    let transient = RenderResourceDescriptor::color_target_with_lifetime(
        "post.temp",
        ResourceLifetime::Transient,
    );
    assert_eq!(transient.lifetime(), ResourceLifetime::Transient);
}

#[test]
fn transient_windows_and_alias_candidates_are_computed() {
    let flow = RenderFlow::new("post.flow")
        .import_texture("surface.color")
        .transient_color_target("post.temp.a")
        .transient_color_target("post.temp.b")
        .fullscreen_pass("post.a")
        .reads("surface.color")
        .writes("post.temp.a")
        .finish()
        .fullscreen_pass("post.consume_a")
        .reads("post.temp.a")
        .writes("surface.color")
        .depends_on("post.a")
        .finish()
        .fullscreen_pass("post.b")
        .reads("surface.color")
        .writes("post.temp.b")
        .depends_on("post.consume_a")
        .finish();

    flow.validate()
        .expect("flow with transient resources should validate");

    let windows = build_transient_windows(flow.graph());
    assert_eq!(windows.len(), 2);
    assert!(windows.iter().any(|window| {
        window.resource_id.as_str() == "post.temp.a"
            && window.first_pass_index == 0
            && window.last_pass_index == 1
    }));
    assert!(windows.iter().any(|window| {
        window.resource_id.as_str() == "post.temp.b"
            && window.first_pass_index == 2
            && window.last_pass_index == 2
    }));

    let aliases = find_aliasable_transients(&windows);
    assert!(aliases.iter().any(|pair| {
        let left = pair.left.as_str();
        let right = pair.right.as_str();
        (left == "post.temp.a" && right == "post.temp.b")
            || (left == "post.temp.b" && right == "post.temp.a")
    }));

    let assignments = build_transient_alias_assignments(&windows);
    assert_eq!(assignments.len(), 2);
    let slots = build_transient_alias_slots(&assignments);
    assert_eq!(slots.len(), 1);
    assert!(
        slots[0]
            .resources
            .iter()
            .any(|id| id.as_str() == "post.temp.a")
    );
    assert!(
        slots[0]
            .resources
            .iter()
            .any(|id| id.as_str() == "post.temp.b")
    );
}
