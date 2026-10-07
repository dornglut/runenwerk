use super::*;
use crate::plugins::render::{
    RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor, RenderTextureSampleMode,
    RenderTextureTargetFormat,
};

fn continuity(raw: u64, surface: RenderSurfaceId) -> Continuity {
    Continuity {
        producer: RenderFrameProducerId::try_from_raw(raw).unwrap(),
        surface,
        target: RenderDynamicTextureTargetKey::new("retained-session-test", format!("{raw}")),
    }
}

fn register(targets: &mut RenderDynamicTextureTargetRequestRegistryResource, key: &Continuity) {
    targets
        .replace_surface_contribution(
            key.producer,
            key.surface,
            [RenderDynamicTextureTargetDescriptor::color_sampled(
                key.target.clone(),
                2,
                2,
                RenderTextureTargetFormat::R32Float,
                RenderTextureSampleMode::NonFilterableFloat,
                RenderDynamicTextureRetention::RetainWhileRequested,
            )],
        )
        .unwrap();
}

#[test]
fn frame_gaps_and_staged_replacement_preserve_exact_registration_until_withdrawal() {
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    let key = continuity(11, RenderSurfaceId::primary());
    register(&mut targets, &key);
    let mut sessions = RetainedRenderSessions::default();
    let registration = sessions
        .select_session(&key, &targets)
        .unwrap()
        .registration
        .clone();
    // No semantic contribution is published in any of these frames.
    for _ in 0..64 {
        sessions.synchronize(&targets);
    }
    assert_eq!(sessions.sessions.len(), 1);
    let mut staged = targets.clone();
    register(&mut staged, &key);
    targets = staged;
    assert!(Weak::ptr_eq(
        &registration,
        &sessions
            .select_session(&key, &targets)
            .unwrap()
            .registration
    ));
    targets.remove_contribution(key.producer);
    sessions.synchronize(&targets);
    assert!(sessions.sessions.is_empty());
    assert!(sessions.select_session(&key, &targets).is_err());
}

#[test]
fn withdrawal_and_reregistration_between_frames_retires_old_continuity() {
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    let key = continuity(12, RenderSurfaceId::primary());
    register(&mut targets, &key);
    let mut sessions = RetainedRenderSessions::default();
    let previous = sessions
        .select_session(&key, &targets)
        .unwrap()
        .registration
        .clone();
    // Keep an old staged snapshot alive: it must not keep the authoritative mapping alive.
    let old_snapshot = targets.clone();
    targets.remove_contribution(key.producer);
    register(&mut targets, &key);
    sessions.synchronize(&targets);
    assert!(sessions.sessions.is_empty());
    let current = sessions
        .select_session(&key, &targets)
        .unwrap()
        .registration
        .clone();
    assert!(!Weak::ptr_eq(&previous, &current));
    assert!(
        old_snapshot
            .continuity_for_target(key.producer, key.surface, &key.target)
            .is_some()
    );
}

#[test]
fn producer_churn_is_bounded_by_live_registrations_and_surface_teardown() {
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    let mut sessions = RetainedRenderSessions::default();
    let secondary = RenderSurfaceId::try_from_raw(2).unwrap();
    let stable = continuity(10, secondary);
    register(&mut targets, &stable);
    sessions.select_session(&stable, &targets).unwrap();
    for raw in 100..356 {
        let transient = continuity(raw, RenderSurfaceId::primary());
        register(&mut targets, &transient);
        sessions.select_session(&transient, &targets).unwrap();
        assert_eq!(sessions.sessions.len(), 2);
        targets.remove_contribution(transient.producer);
        sessions.synchronize(&targets);
        assert_eq!(sessions.sessions.len(), 1);
    }
    sessions.retire_surface(RenderSurfaceId::primary());
    assert_eq!(sessions.sessions.len(), 1);
    sessions.retire_surface(secondary);
    assert!(sessions.sessions.is_empty());
}

#[test]
fn scope_replacement_target_withdrawal_and_registry_clear_retire_only_owned_sessions() {
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    let mut sessions = RetainedRenderSessions::default();
    let primary = continuity(13, RenderSurfaceId::primary());
    let secondary = continuity(14, RenderSurfaceId::try_from_raw(2).unwrap());
    for key in [&primary, &secondary] {
        register(&mut targets, key);
        sessions.select_session(key, &targets).unwrap();
    }
    let mut moved = primary.clone();
    moved.surface = secondary.surface;
    register(&mut targets, &moved);
    sessions.synchronize(&targets);
    assert_eq!(sessions.sessions.len(), 1);
    assert!(sessions.sessions.contains_key(&secondary));
    sessions.select_session(&moved, &targets).unwrap();
    targets
        .replace_surface_contribution(moved.producer, moved.surface, [])
        .unwrap();
    sessions.synchronize(&targets);
    assert_eq!(sessions.sessions.len(), 1);
    targets.clear();
    sessions.synchronize(&targets);
    assert!(sessions.sessions.is_empty());
}

#[test]
fn target_withdrawal_between_frames_renews_only_that_target_registration() {
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    let key = continuity(15, RenderSurfaceId::primary());
    register(&mut targets, &key);
    let mut sessions = RetainedRenderSessions::default();
    let previous = sessions
        .select_session(&key, &targets)
        .unwrap()
        .registration
        .clone();
    let mut replacement = key.clone();
    replacement.target = RenderDynamicTextureTargetKey::new("retained-session-test", "replacement");
    register(&mut targets, &replacement);
    register(&mut targets, &key);
    sessions.synchronize(&targets);
    assert!(sessions.sessions.is_empty());
    let current = sessions
        .select_session(&key, &targets)
        .unwrap()
        .registration
        .clone();
    assert!(!Weak::ptr_eq(&previous, &current));
}

#[test]
fn authoritative_surface_retirement_drops_sessions_even_before_backend_detachment() {
    use crate::plugins::render::host::RenderSurfaceRegistryResource;
    use crate::runtime::NativeWindowId;
    let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
    let mut sessions = RetainedRenderSessions::default();
    let mut surfaces = RenderSurfaceRegistryResource::default();
    let primary = continuity(16, RenderSurfaceId::primary());
    let native = NativeWindowId::try_from_raw(2).unwrap();
    let secondary_surface = surfaces.reserve_surface_for_native_window(native, (2, 2));
    let secondary = continuity(17, secondary_surface);
    surfaces
        .confirm_surface_attachment(primary.surface, NativeWindowId::primary(), (2, 2))
        .unwrap();
    surfaces
        .confirm_surface_attachment(secondary.surface, native, (2, 2))
        .unwrap();
    for key in [&primary, &secondary] {
        register(&mut targets, key);
        sessions.select_session(key, &targets).unwrap();
    }
    surfaces.retire_surface_for_native_window(native);
    sessions.retire_unattached_surfaces(&surfaces);
    assert_eq!(sessions.sessions.len(), 1);
    assert!(sessions.sessions.contains_key(&primary));
}

mod execution;
