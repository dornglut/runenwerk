use ui_runtime_view::ButtonRuntimeHostData;
use ui_story::UiStoryManifestV2;

const SELECTED_BUTTON_STORY_ID: &str = "ui.gallery.button.selected";
const SELECTED_BUTTON_ACTIVE_ENDPOINT: &str = "ui_gallery.button.selected.active";

pub(super) fn host_data_for_story(story: &UiStoryManifestV2) -> ButtonRuntimeHostData {
    let host_data = ButtonRuntimeHostData::new();
    if story.story_id.as_str() == SELECTED_BUTTON_STORY_ID {
        host_data.with_bool(SELECTED_BUTTON_ACTIVE_ENDPOINT, true)
    } else {
        host_data
    }
}
