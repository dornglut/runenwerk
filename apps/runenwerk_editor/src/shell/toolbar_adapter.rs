use editor_core::ToolId;
use editor_shell::{ToolbarButtonViewModel, ToolbarViewModel};

pub const SELECT_TOOL_ID: ToolId = ToolId(1);
pub const TRANSLATE_TOOL_ID: ToolId = ToolId(2);

pub fn build_toolbar_view_model(
	active_tool: Option<ToolId>,
) -> ToolbarViewModel {
	ToolbarViewModel {
		buttons: vec![
			ToolbarButtonViewModel {
				id: SELECT_TOOL_ID,
				stable_name: "select",
				label: "Select".to_string(),
				is_active: active_tool == Some(SELECT_TOOL_ID),
			},
			ToolbarButtonViewModel {
				id: TRANSLATE_TOOL_ID,
				stable_name: "translate",
				label: "Translate".to_string(),
				is_active: active_tool == Some(TRANSLATE_TOOL_ID),
			},
		],
	}
}