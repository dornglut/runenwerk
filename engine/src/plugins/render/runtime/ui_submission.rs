use crate::plugins::render::*;
use crate::plugins::scene::ui::UiRenderShaderConfig;
use crate::plugins::scene::SceneResource;
use crate::runtime::WorldMut;
use crate::state::UiOverlayState;

const SCENE_OVERLAY_UI_PRODUCER_ID: UiFrameProducerId = UiFrameProducerId::new(1);
const DEBUG_METRICS_UI_PRODUCER_ID: UiFrameProducerId = UiFrameProducerId::new(2);

pub(crate) fn collect_runtime_ui_frame_submissions_system(mut world: WorldMut) {
	let Some(mut submissions) = world.remove_resource::<UiFrameSubmissionRegistryResource>() else {
		return;
	};

	let scene_submission = world
		.resource::<SceneResource>()
		.ok()
		.and_then(|scene_resource| scene_resource.manager.as_ref())
		.map(|manager| {
			let rect_shader_asset_id = manager
				.overlay_runtime
				.world
				.resource::<UiRenderShaderConfig>()
				.ok()
				.map(|config| config.rect_shader_asset_id.trim().to_string())
				.filter(|id| !id.is_empty());
			(
				manager.overlay_runtime.ui.frame.clone(),
				rect_shader_asset_id,
			)
		});

	match scene_submission {
		Some((frame, rect_shader_asset_id)) if !frame.is_empty() => {
			submissions.replace(
				UiFrameSubmission::new(SCENE_OVERLAY_UI_PRODUCER_ID)
					.with_route(UiFrameRoute::Screen)
					.with_order(UiFrameSubmissionOrder::new(0, 0))
					.with_frame(frame)
					.with_rect_shader_asset_id(rect_shader_asset_id),
			);
		}
		_ => {
			submissions.remove(&SCENE_OVERLAY_UI_PRODUCER_ID);
		}
	}

	let debug_frame = world
		.resource::<UiOverlayState>()
		.ok()
		.map(|overlay| overlay.debug_frame.clone())
		.unwrap_or_default();

	if debug_frame.is_empty() {
		submissions.remove(&DEBUG_METRICS_UI_PRODUCER_ID);
	} else {
		submissions.replace(
			UiFrameSubmission::new(DEBUG_METRICS_UI_PRODUCER_ID)
				.with_route(UiFrameRoute::Screen)
				.with_order(UiFrameSubmissionOrder::new(100, 0))
				.with_frame(debug_frame),
		);
	}

	world.insert_resource(submissions);
}