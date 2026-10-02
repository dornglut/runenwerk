use engine::plugins::render::frame::{
    PreparedRenderProductSelectionResource, RenderDeterministicFrameContributionResource,
};
use engine::plugins::render::runtime::RenderDynamicTextureTargetRequestRegistryResource;
use runenwerk_arena::{
    ArenaPresentationSceneResource, ArenaPresentationState, build_game_app, build_headless_game_app,
};

#[test]
fn headless_game_composition_remains_presentation_free() {
    let app = build_headless_game_app();

    assert!(
        app.world().resource::<ArenaPresentationState>().is_err(),
        "headless gameplay must not install interpolation/presentation state"
    );
    assert!(
        app.world()
            .resource::<ArenaPresentationSceneResource>()
            .is_err(),
        "headless gameplay must not install renderer-local scene identity"
    );
    assert!(
        app.world()
            .resource::<PreparedRenderProductSelectionResource>()
            .is_err(),
        "headless gameplay must not activate Render product selection"
    );
    assert!(
        app.world()
            .resource::<RenderDynamicTextureTargetRequestRegistryResource>()
            .is_err(),
        "headless gameplay must not activate Render target publication"
    );
    assert!(
        app.world()
            .resource::<RenderDeterministicFrameContributionResource>()
            .is_err(),
        "headless gameplay must not activate deterministic Render frame publication"
    );
}

#[test]
fn rendered_game_composition_installs_render_and_arena_presentation() {
    let app = build_game_app(false);

    assert!(
        app.world().resource::<ArenaPresentationState>().is_ok(),
        "rendered game must install arena presentation state"
    );
    assert!(
        app.world()
            .resource::<ArenaPresentationSceneResource>()
            .is_ok(),
        "rendered game must own stable renderer-local arena identities"
    );
    assert!(
        app.world()
            .resource::<PreparedRenderProductSelectionResource>()
            .is_ok(),
        "rendered game must install Render product selection authority"
    );
    assert!(
        app.world()
            .resource::<RenderDynamicTextureTargetRequestRegistryResource>()
            .is_ok(),
        "rendered game must install Render target publication"
    );
    assert!(
        app.world()
            .resource::<RenderDeterministicFrameContributionResource>()
            .is_ok(),
        "rendered game must install deterministic Render frame publication"
    );
}
