use crate::{UiInteraction, UiInteractionResults, UiRuntime};
use ui_input::{Modifiers, PointerEvent, PointerEventKind, UiInputEvent};
use ui_math::{UiPoint, UiRect, UiVector};
use ui_render_data::UiPrimitive;
use ui_text::{FontAtlasSource, FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas};
use ui_theme::ThemeTokens;

use crate::{
    ActiveTabDragVisualState, BODY_CONSOLE_SPLIT_WIDGET_ID, CONSOLE_HSCROLL_WIDGET_ID,
    CONSOLE_LIST_WIDGET_ID, CONSOLE_PANEL_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID, ConsoleViewModel,
    DockingInteractionVisualState, DockingPreviewDropTarget, ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID,
    ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID, EditorShellViewModel, EntityTableViewModel,
    INSPECTOR_PANEL_WIDGET_ID, InspectorFieldViewModel, InspectorTargetViewModel,
    InspectorViewModel, OUTLINER_PANEL_WIDGET_ID, OutlinerRowViewModel, OutlinerViewModel,
    PanelInstanceId, RoutedShellAction, ShellCommand, TOOLBAR_ROOT_WIDGET_ID,
    TOOLBAR_ROW_WIDGET_ID, TOOLBAR_SCROLL_WIDGET_ID, ToolbarButtonViewModel, ToolbarViewModel,
    VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID, ViewportProductChoiceViewModel,
    ViewportViewModel, WorkspaceIdentityAllocator, build_editor_shell,
    build_editor_shell_with_docking_visual_state, map_interactions_to_shell_commands,
    tab_strip_scroll_widget_id, viewport_product_button_widget_id,
};

#[test]
fn shell_view_model_builds_ui_tree_and_frame() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();

    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let atlas_source = TestAtlasSource::ascii();
    let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 1600.0, 900.0), &atlas_source);

    assert_eq!(tree.root_id().0, 1);
    assert_eq!(frame.surfaces.len(), 1);
    assert!(!frame.surfaces[0].layers[0].primitives.is_empty());
}

#[test]
fn viewport_embed_uv_rect_maps_to_canvas_screen_region() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let atlas_source = TestAtlasSource::ascii();
    let surface_bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);
    let frame = runtime.build_frame(&tree, surface_bounds, &atlas_source);

    let embed = frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .find_map(|primitive| match primitive {
            UiPrimitive::ViewportSurfaceEmbed(value) => Some(value),
            _ => None,
        })
        .expect("viewport embed primitive should exist");

    let expected_u0 = embed.rect.x / surface_bounds.width;
    let expected_v0 = embed.rect.y / surface_bounds.height;
    let expected_uw = embed.rect.width / surface_bounds.width;
    let expected_vh = embed.rect.height / surface_bounds.height;

    assert!(
        (embed.uv_rect.x - expected_u0).abs() <= 0.0001,
        "embed uv x should match normalized panel x",
    );
    assert!(
        (embed.uv_rect.y - expected_v0).abs() <= 0.0001,
        "embed uv y should match normalized panel y",
    );
    assert!(
        (embed.uv_rect.width - expected_uw).abs() <= 0.0001,
        "embed uv width should match normalized panel width",
    );
    assert!(
        (embed.uv_rect.height - expected_vh).abs() <= 0.0001,
        "embed uv height should match normalized panel height",
    );
}

#[test]
fn viewport_panel_without_viewport_identity_renders_without_embed_primitive() {
    let theme = ThemeTokens::default();
    let mut shell = sample_shell_view_model();
    shell.viewport.viewport_id = None;
    shell.viewport.product_choices.clear();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let atlas_source = TestAtlasSource::ascii();
    let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 1600.0, 900.0), &atlas_source);

    let has_embed = frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .any(|primitive| matches!(primitive, UiPrimitive::ViewportSurfaceEmbed(_)));

    assert!(
        !has_embed,
        "viewport panel must not synthesize embed primitives when viewport identity is absent",
    );
}

#[test]
fn layout_keeps_viewport_canvas_nonzero_and_inside_viewport_panel() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 1600.0, 900.0));

    let viewport_panel = layouts
        .get(&VIEWPORT_PANEL_WIDGET_ID)
        .expect("viewport panel layout should exist")
        .bounds;
    let viewport_canvas = layouts
        .get(&VIEWPORT_CANVAS_WIDGET_ID)
        .expect("viewport canvas layout should exist")
        .bounds;

    assert!(
        viewport_canvas.width > 0.0,
        "viewport canvas width must be non-zero"
    );
    assert!(
        viewport_canvas.height > 0.0,
        "viewport canvas height must be non-zero"
    );

    assert!(viewport_canvas.x >= viewport_panel.x);
    assert!(viewport_canvas.y >= viewport_panel.y);
    assert!(viewport_canvas.x + viewport_canvas.width <= viewport_panel.x + viewport_panel.width);
    assert!(viewport_canvas.y + viewport_canvas.height <= viewport_panel.y + viewport_panel.height);
}

#[test]
fn layout_ensures_major_panels_do_not_overlap() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 1600.0, 900.0));

    let toolbar = layout_bounds(&layouts, TOOLBAR_ROOT_WIDGET_ID);
    let outliner = layout_bounds(&layouts, OUTLINER_PANEL_WIDGET_ID);
    let viewport = layout_bounds(&layouts, VIEWPORT_PANEL_WIDGET_ID);
    let inspector = layout_bounds(&layouts, INSPECTOR_PANEL_WIDGET_ID);
    let console = layout_bounds(&layouts, CONSOLE_PANEL_WIDGET_ID);

    assert!(!intersects(toolbar, outliner));
    assert!(!intersects(toolbar, viewport));
    assert!(!intersects(toolbar, inspector));
    assert!(!intersects(outliner, viewport));
    assert!(!intersects(outliner, inspector));
    assert!(!intersects(viewport, inspector));
    assert!(!intersects(outliner, console));
    assert!(!intersects(viewport, console));
    assert!(!intersects(inspector, console));
}

#[test]
fn tab_strips_use_horizontal_scroll_containers_in_narrow_bounds() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let build = build_editor_shell(&shell, &theme, &sample_workspace_state());

    let fixed = &build.projection_artifacts.workspace.fixed_layout;
    let tab_stacks = [
        (
            fixed.outliner.tab_stack_id,
            fixed.outliner.tab_strip_widget_id,
        ),
        (
            fixed.viewport.tab_stack_id,
            fixed.viewport.tab_strip_widget_id,
        ),
        (
            fixed.inspector.tab_stack_id,
            fixed.inspector.tab_strip_widget_id,
        ),
        (
            fixed.console.tab_stack_id,
            fixed.console.tab_strip_widget_id,
        ),
    ];

    for (tab_stack_id, strip_row_id) in tab_stacks {
        let strip_scroll_id = tab_strip_scroll_widget_id(tab_stack_id);
        let strip_scroll = build
            .tree
            .walk()
            .find(|node| node.id == strip_scroll_id)
            .expect("tab strip horizontal scroll node should exist");
        let strip_row = build
            .tree
            .walk()
            .find(|node| node.id == strip_row_id)
            .expect("tab strip row node should exist");
        assert!(
            matches!(&strip_scroll.kind, crate::UiNodeKind::Scroll(scroll) if scroll.axis == ui_math::Axis::Horizontal),
            "tab strip should be wrapped by a horizontal scroll container",
        );
        assert!(
            strip_scroll
                .children
                .iter()
                .any(|child| child.id == strip_row_id),
            "tab strip row should be direct content of the horizontal scroll container",
        );
        assert!(matches!(strip_row.kind, crate::UiNodeKind::Stack(_)));
    }
}

#[test]
fn tab_strips_hide_drop_zone_widgets_without_active_drag() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let build = build_editor_shell(&shell, &theme, &sample_workspace_state());

    let drop_zone_ids = build
        .projection_artifacts
        .workspace
        .fixed_layout
        .outliner
        .drop_slots
        .iter()
        .map(|slot| slot.widget_id)
        .collect::<Vec<_>>();

    for drop_zone_id in drop_zone_ids {
        assert!(
            !build.tree.walk().any(|node| node.id == drop_zone_id),
            "tab drop-zone widget should be hidden while no active tab drag is present",
        );
    }
}

#[test]
fn tab_strips_show_drop_zone_widgets_during_active_drag_preview() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let workspace = sample_workspace_state();
    let baseline = build_editor_shell(&shell, &theme, &workspace);
    let outliner_slot = &baseline
        .projection_artifacts
        .workspace
        .fixed_layout
        .outliner;
    let panel_instance_id = outliner_slot
        .active_panel
        .expect("outliner should have an active panel")
        .panel_instance_id;
    let docking = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id,
            source_tab_stack_id: outliner_slot.tab_stack_id,
            preview_target: Some(DockingPreviewDropTarget::TabStack {
                tab_stack_id: outliner_slot.tab_stack_id,
                insert_index: 0,
            }),
        }),
        active_split_border_widget: None,
    };
    let with_drag =
        build_editor_shell_with_docking_visual_state(&shell, &theme, &workspace, Some(&docking));

    for drop_zone in &outliner_slot.drop_slots {
        assert!(
            with_drag
                .tree
                .walk()
                .any(|node| node.id == drop_zone.widget_id),
            "tab drop-zone widgets should be mounted during active drag preview",
        );
    }
}

#[test]
fn split_handles_render_for_resizable_shell_splits() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let workspace = sample_workspace_state();
    let build = build_editor_shell(&shell, &theme, &workspace);

    for widget_id in [
        crate::LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        crate::CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        crate::BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID,
    ] {
        assert!(
            build.tree.walk().any(|node| node.id == widget_id),
            "split handle widget should be mounted for resizable split",
        );
    }
}

#[test]
fn active_split_border_highlights_matching_split_handle() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let workspace = sample_workspace_state();
    let docking = DockingInteractionVisualState {
        active_tab_drag: None,
        active_split_border_widget: Some(crate::LEFT_RIGHT_SPLIT_WIDGET_ID),
    };
    let build =
        build_editor_shell_with_docking_visual_state(&shell, &theme, &workspace, Some(&docking));

    let left_handle = build
        .tree
        .walk()
        .find(|node| node.id == crate::LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID)
        .expect("left-right split handle should exist");
    let center_handle = build
        .tree
        .walk()
        .find(|node| node.id == crate::CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID)
        .expect("center-right split handle should exist");

    let crate::UiNodeKind::Button(left_button) = &left_handle.kind else {
        panic!("left split handle should render as button");
    };
    let crate::UiNodeKind::Button(center_button) = &center_handle.kind else {
        panic!("center split handle should render as button");
    };
    assert!(
        left_button.selected,
        "active split handle should be selected"
    );
    assert!(
        !center_button.selected,
        "non-active split handle should remain unselected",
    );
}

#[test]
fn outliner_selected_row_uses_visual_selection_without_prefix_marker() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let outliner_row = tree
        .walk()
        .find(|node| node.id == crate::outliner_row_widget_id(0))
        .expect("first outliner row should exist");

    let crate::UiNodeKind::Button(button) = &outliner_row.kind else {
        panic!("outliner row should be rendered as a button");
    };
    assert!(
        button.selected,
        "selected outliner row should use button selected state",
    );
    assert!(
        !button.label.starts_with('•'),
        "outliner label should not use bullet-prefix selection marker",
    );
    assert!(
        !button.label.starts_with('*'),
        "outliner label should not use star-prefix selection marker",
    );
}

#[test]
fn toolbar_uses_horizontal_scroll_container_for_overflow() {
    let theme = ThemeTokens::default();
    let shell = sample_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let toolbar_scroll = tree
        .walk()
        .find(|node| node.id == TOOLBAR_SCROLL_WIDGET_ID)
        .expect("toolbar horizontal scroll node should exist");
    let toolbar_row = tree
        .walk()
        .find(|node| node.id == TOOLBAR_ROW_WIDGET_ID)
        .expect("toolbar row node should exist");

    assert!(
        matches!(&toolbar_scroll.kind, crate::UiNodeKind::Scroll(scroll) if scroll.axis == ui_math::Axis::Horizontal),
        "toolbar should use a horizontal scroll container",
    );
    assert!(
        toolbar_scroll
            .children
            .iter()
            .any(|child| child.id == TOOLBAR_ROW_WIDGET_ID),
        "toolbar row should be direct content of the horizontal scroll container",
    );
    assert!(matches!(toolbar_row.kind, crate::UiNodeKind::Stack(_)));
}

#[test]
fn entity_table_uses_horizontal_scroll_for_wide_content() {
    let theme = ThemeTokens::default();
    let view_model = sample_shell_view_model().entity_table;
    let panel = crate::build_entity_table_panel(&view_model, &theme, PanelInstanceId::new(1), None);
    let tree = crate::UiTree::new(panel);

    let header_scroll = tree
        .walk()
        .find(|node| node.id == ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID)
        .expect("entity table header horizontal scroll should exist");
    let table_scroll = tree
        .walk()
        .find(|node| node.id == ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID)
        .expect("entity table body horizontal scroll should exist");

    assert!(
        matches!(&header_scroll.kind, crate::UiNodeKind::Scroll(scroll) if scroll.axis == ui_math::Axis::Horizontal),
        "entity table header should use horizontal scroll",
    );
    assert!(
        matches!(&table_scroll.kind, crate::UiNodeKind::Scroll(scroll) if scroll.axis == ui_math::Axis::Horizontal),
        "entity table rows should use horizontal scroll",
    );
}

#[test]
fn console_scroll_stays_within_console_panel_on_tight_vertical_bounds() {
    let theme = ThemeTokens::default();
    let shell = scrollable_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 420.0, 120.0));

    let console_panel = layout_bounds(&layouts, CONSOLE_PANEL_WIDGET_ID);
    let console_scroll = layout_bounds(&layouts, CONSOLE_SCROLL_WIDGET_ID);

    assert!(
        console_scroll.x >= console_panel.x && console_scroll.y >= console_panel.y,
        "console scroll origin should remain inside panel bounds even in tight vertical sizes",
    );
    if console_panel.width <= f32::EPSILON || console_panel.height <= f32::EPSILON {
        assert!(
            console_scroll.width <= f32::EPSILON || console_scroll.height <= f32::EPSILON,
            "zero-sized console panel should collapse scroll viewport instead of producing visible overflow",
        );
        return;
    }
    assert!(
        console_scroll.x + console_scroll.width <= console_panel.x + console_panel.width + 0.001
            && console_scroll.y + console_scroll.height
                <= console_panel.y + console_panel.height + 0.001,
        "console scroll area should remain contained within panel bounds in tight vertical sizes",
    );
}

#[test]
fn console_scroll_offset_clamps_and_reserves_scrollbar_gutter() {
    let theme = ThemeTokens::default();
    let shell = scrollable_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let mut runtime = UiRuntime::new();
    let bounds = UiRect::new(0.0, 0.0, 1280.0, 720.0);

    let initial_layouts = runtime.compute_layout(&tree, bounds);
    let scroll_bounds = layout_bounds(&initial_layouts, CONSOLE_SCROLL_WIDGET_ID);
    let scroll_content_bounds = initial_layouts
        .get(&CONSOLE_SCROLL_WIDGET_ID)
        .expect("console scroll layout should exist")
        .content_bounds;
    assert!(
        scroll_content_bounds.width < scroll_bounds.width,
        "scroll container should reserve visible scrollbar gutter"
    );

    let pointer = UiPoint::new(
        scroll_content_bounds.x + scroll_content_bounds.width * 0.5,
        scroll_content_bounds.y + 12.0,
    );

    for _ in 0..64 {
        let layouts = runtime.compute_layout(&tree, bounds);
        runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: pointer,
                delta: UiVector::new(0.0, -8.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
            }),
        );
    }

    let max_offset = max_scroll_offset(
        &runtime.compute_layout(&tree, bounds),
        CONSOLE_SCROLL_WIDGET_ID,
        CONSOLE_LIST_WIDGET_ID,
    );
    let offset = runtime.state().scroll_offset(CONSOLE_SCROLL_WIDGET_ID);
    assert!(offset > 0.0, "scrolling should advance console offset");
    assert!(
        offset <= max_offset + 0.001,
        "scroll offset should clamp to content range"
    );

    for _ in 0..64 {
        let layouts = runtime.compute_layout(&tree, bounds);
        runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: pointer,
                delta: UiVector::new(0.0, 8.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
            }),
        );
    }
    assert!(
        runtime.state().scroll_offset(CONSOLE_SCROLL_WIDGET_ID) <= 0.001,
        "scrolling upward should clamp back to zero"
    );
}

#[test]
fn console_uses_nested_horizontal_scroll_for_wide_lines() {
    let theme = ThemeTokens::default();
    let shell = scrollable_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let vscroll = tree
        .walk()
        .find(|node| node.id == CONSOLE_SCROLL_WIDGET_ID)
        .expect("console vertical scroll node should exist");
    let hscroll = tree
        .walk()
        .find(|node| node.id == CONSOLE_HSCROLL_WIDGET_ID)
        .expect("console horizontal scroll node should exist");

    assert!(
        matches!(&vscroll.kind, crate::UiNodeKind::Scroll(scroll) if scroll.axis == ui_math::Axis::Vertical),
        "console primary scroll should remain vertical",
    );
    assert!(
        matches!(&hscroll.kind, crate::UiNodeKind::Scroll(scroll) if scroll.axis == ui_math::Axis::Horizontal),
        "console should include nested horizontal scroll for long lines",
    );
    assert_eq!(
        hscroll.children.first().map(|child| child.id),
        Some(CONSOLE_SCROLL_WIDGET_ID),
        "console vertical scroll should be direct content of the horizontal scroll container",
    );
}

#[test]
fn floating_host_hides_drop_zone_when_no_drag_is_active() {
    let theme = ThemeTokens::default();
    let shell = scrollable_shell_view_model();
    let workspace = workspace_with_console_moved_to_floating_host();
    let build = build_editor_shell(&shell, &theme, &workspace);

    assert!(
        build
            .tree
            .walk()
            .filter_map(|node| match &node.kind {
                crate::UiNodeKind::Button(button) => Some(button.label.as_str()),
                _ => None,
            })
            .all(|label| label != "Float Drop Zone" && label != "Drop Here To Float"),
        "floating drop zone should only render during active drag preview",
    );
}

#[test]
fn empty_console_slot_collapses_bottom_split_and_expands_body() {
    let theme = ThemeTokens::default();
    let shell = scrollable_shell_view_model();
    let workspace = workspace_with_console_moved_to_floating_host();
    let build = build_editor_shell(&shell, &theme, &workspace);

    assert!(
        build
            .tree
            .walk()
            .all(|node| node.id != BODY_CONSOLE_SPLIT_WIDGET_ID),
        "body/console split should collapse when console tab stack is empty",
    );
}

#[test]
fn tiny_bounds_frame_build_does_not_panic() {
    let theme = ThemeTokens::default();
    let shell = scrollable_shell_view_model();
    let tree = build_editor_shell(&shell, &theme, &sample_workspace_state()).tree;
    let runtime = UiRuntime::new();
    let atlas_source = TestAtlasSource::ascii();

    let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 96.0, 64.0), &atlas_source);

    assert_eq!(frame.surfaces.len(), 1);
    assert!(
        !frame.surfaces[0].layers[0].primitives.is_empty(),
        "tiny bounds should still produce a valid frame",
    );
}

struct TestAtlasSource {
    atlas: MsdfFontAtlas,
}

impl TestAtlasSource {
    fn ascii() -> Self {
        let mut glyphs = std::collections::HashMap::new();
        for byte in 32_u8..=126_u8 {
            glyphs.insert(
                byte as char,
                GlyphMetrics {
                    advance: 10.0,
                    plane_left: 0.0,
                    plane_top: 8.0,
                    plane_right: 8.0,
                    plane_bottom: -2.0,
                    atlas_left: 0.0,
                    atlas_top: 0.0,
                    atlas_right: 0.1,
                    atlas_bottom: 0.1,
                },
            );
        }

        Self {
            atlas: MsdfFontAtlas {
                font_id: FontId(1),
                texture_width: 256,
                texture_height: 256,
                metrics: FontFaceMetrics {
                    ascender: 9.0,
                    descender: -3.0,
                    line_height: 12.0,
                    base_size: 12.0,
                },
                glyphs,
            },
        }
    }
}

impl FontAtlasSource for TestAtlasSource {
    fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas> {
        if font_id == self.atlas.font_id {
            Some(&self.atlas)
        } else {
            None
        }
    }
}

#[test]
fn toolbar_activation_maps_to_shell_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(
            crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
        )],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(2),
                stable_name: "translate",
                label: "Translate".to_string(),
                is_active: false,
                enabled: true,
            }],
        },
        outliner: OutlinerViewModel::default(),
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel::default(),
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    assert_eq!(commands, vec![ShellCommand::ActivateTranslateTool]);
}

#[test]
fn outliner_row_activation_maps_to_select_entity_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(crate::outliner_row_widget_id(0))],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel {
            rows: vec![OutlinerRowViewModel {
                entity: editor_core::EntityId(42),
                display_name: "Root".to_string(),
                depth: 0,
                is_selected: false,
            }],
        },
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel::default(),
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    let expected_target = composition
        .projection_artifacts
        .widget_structural_context_by_id
        .get(&crate::outliner_row_widget_id(0))
        .copied()
        .expect("outliner row structural context should exist");
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        ShellCommand::SelectOutlinerEntity {
            entity: editor_core::EntityId(42),
            target,
            projection_epoch: 0,
        } if target.panel_instance_id == expected_target.panel_instance_id
            && target.active_tool_surface == expected_target.active_tool_surface
            && target.tab_stack_id == expected_target.tab_stack_id
    ));

    assert_eq!(
        composition.projection_artifacts.projection_epoch, 0,
        "direct build artifacts are unbound until cached in shell state",
    );
}

#[test]
fn viewport_product_activation_maps_to_select_product_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(viewport_product_button_widget_id(
            0,
        ))],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel {
            viewport_id: Some(editor_viewport::ViewportId(1)),
            selected_primary_product_id: Some(editor_viewport::ExpressionProductId(1)),
            product_choices: vec![ViewportProductChoiceViewModel {
                viewport_id: editor_viewport::ViewportId(1),
                product_id: editor_viewport::ExpressionProductId(2),
                label: "PickingIds2D".to_string(),
                selected: false,
                enabled: true,
            }],
            details_visible: false,
            selected_entity: None,
            hovered_entity: None,
            drag_in_progress: false,
            preview_active: false,
        },
        inspector: InspectorViewModel::default(),
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        ShellCommand::SelectViewportProduct {
            viewport_id: editor_viewport::ViewportId(1),
            product_id: editor_viewport::ExpressionProductId(2),
            projection_epoch: 0,
            ..
        }
    ));
}

#[test]
fn inspector_field_activation_maps_to_shell_edit_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(crate::inspector_field_widget_id(
            3,
        ))],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel {
            target: InspectorTargetViewModel::Entity {
                display_name: "Entity".to_string(),
            },
            fields: vec![
                InspectorFieldViewModel {
                    label: "a".to_string(),
                    path_key: None,
                    value_summary: "1".to_string(),
                    is_focused: false,
                    editable: false,
                },
                InspectorFieldViewModel {
                    label: "b".to_string(),
                    path_key: None,
                    value_summary: "2".to_string(),
                    is_focused: false,
                    editable: false,
                },
                InspectorFieldViewModel {
                    label: "c".to_string(),
                    path_key: None,
                    value_summary: "3".to_string(),
                    is_focused: false,
                    editable: false,
                },
                InspectorFieldViewModel {
                    label: "d".to_string(),
                    path_key: None,
                    value_summary: "4".to_string(),
                    is_focused: false,
                    editable: false,
                },
            ],
        },
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        ShellCommand::ActivateInspectorField {
            index: 3,
            projection_epoch: 0,
            ..
        }
    ));
}

#[test]
fn inspector_editable_field_text_input_maps_to_append_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::TextInput {
            target: crate::inspector_field_widget_id(0),
            event: ui_input::TextInputEvent {
                text: "1".to_string(),
            },
        }],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel {
            target: InspectorTargetViewModel::Component {
                entity_display_name: "Entity".to_string(),
                component_display_name: "LocalTransform".to_string(),
            },
            fields: vec![InspectorFieldViewModel {
                label: "translation.x".to_string(),
                path_key: Some("translation.x".to_string()),
                value_summary: "0".to_string(),
                is_focused: true,
                editable: true,
            }],
        },
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        ShellCommand::AppendInspectorFieldText {
            index: 0,
            text,
            projection_epoch: 0,
            ..
        } if text == "1"
    ));
}

#[test]
fn inspector_editable_field_enter_maps_to_commit_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::KeyboardInput {
            target: crate::inspector_field_widget_id(0),
            event: ui_input::KeyboardEvent {
                key: ui_input::Key::Enter,
                state: ui_input::KeyState::Pressed,
                modifiers: ui_input::Modifiers::default(),
            },
        }],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel {
            target: InspectorTargetViewModel::Component {
                entity_display_name: "Entity".to_string(),
                component_display_name: "LocalTransform".to_string(),
            },
            fields: vec![InspectorFieldViewModel {
                label: "translation.x".to_string(),
                path_key: Some("translation.x".to_string()),
                value_summary: "0".to_string(),
                is_focused: true,
                editable: true,
            }],
        },
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        ShellCommand::CommitInspectorFieldText {
            index: 0,
            projection_epoch: 0,
            ..
        }
    ));
}

#[test]
fn inspector_focus_label_activation_maps_to_focus_command() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(
            crate::inspector_field_focus_widget_id(0),
        )],
    };
    let view_model = EditorShellViewModel {
        toolbar: ToolbarViewModel::default(),
        outliner: OutlinerViewModel::default(),
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel::default(),
        inspector: InspectorViewModel {
            target: InspectorTargetViewModel::Component {
                entity_display_name: "Entity".to_string(),
                component_display_name: "LocalTransform".to_string(),
            },
            fields: vec![InspectorFieldViewModel {
                label: "translation.x".to_string(),
                path_key: Some("translation.x".to_string()),
                value_summary: "0".to_string(),
                is_focused: false,
                editable: true,
            }],
        },
        console: ConsoleViewModel::default(),
    };

    let composition = build_editor_shell(
        &view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );
    let commands =
        map_interactions_to_shell_commands(&interactions, &composition.projection_artifacts);

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        ShellCommand::FocusInspectorField {
            index: 0,
            projection_epoch: 0,
            ..
        }
    ));
}

#[test]
fn inspector_vector_fields_group_horizontally_from_path_keys() {
    let theme = ThemeTokens::default();
    let panel = crate::build_inspector_panel(
        &InspectorViewModel {
            target: InspectorTargetViewModel::Component {
                entity_display_name: "Entity".to_string(),
                component_display_name: "Transform".to_string(),
            },
            fields: vec![
                InspectorFieldViewModel {
                    label: "x".to_string(),
                    path_key: Some("translation.x".to_string()),
                    value_summary: "0.0".to_string(),
                    is_focused: false,
                    editable: true,
                },
                InspectorFieldViewModel {
                    label: "y".to_string(),
                    path_key: Some("translation.y".to_string()),
                    value_summary: "0.0".to_string(),
                    is_focused: false,
                    editable: true,
                },
                InspectorFieldViewModel {
                    label: "z".to_string(),
                    path_key: Some("translation.z".to_string()),
                    value_summary: "0.0".to_string(),
                    is_focused: false,
                    editable: true,
                },
            ],
        },
        &theme,
        PanelInstanceId::new(1),
        None,
    );
    let tree = crate::UiTree::new(panel);

    assert!(
        tree.walk().any(|node| matches!(
            &node.kind,
            crate::UiNodeKind::Label(label) if label.text == "Position:"
        )),
        "vector group header should render canonical Position label",
    );
    assert!(
        tree.walk().all(|node| !matches!(
            &node.kind,
            crate::UiNodeKind::Label(label) if label.text == "translation.x:"
        )),
        "ungrouped per-axis labels should not be rendered for grouped vectors",
    );
}

#[test]
fn unchanged_workspace_rebuild_keeps_structural_projection_routing_stable() {
    let theme = ThemeTokens::default();
    let workspace = sample_workspace_state();
    let view_model = sample_shell_view_model();

    let first = build_editor_shell(&view_model, &theme, &workspace);
    let second = build_editor_shell(&view_model, &theme, &workspace);

    assert_eq!(
        first.projection_artifacts.workspace.widget_context_by_id,
        second.projection_artifacts.workspace.widget_context_by_id
    );
}

#[test]
fn widget_rebuild_keeps_structural_routing_context_for_same_widget() {
    let theme = ThemeTokens::default();
    let workspace = sample_workspace_state();
    let mut first_view_model = sample_shell_view_model();
    first_view_model.outliner.rows = vec![
        OutlinerRowViewModel {
            entity: editor_core::EntityId(11),
            display_name: "A".to_string(),
            depth: 0,
            is_selected: false,
        },
        OutlinerRowViewModel {
            entity: editor_core::EntityId(12),
            display_name: "B".to_string(),
            depth: 0,
            is_selected: false,
        },
    ];
    let mut second_view_model = first_view_model.clone();
    second_view_model.outliner.rows.reverse();

    let first = build_editor_shell(&first_view_model, &theme, &workspace);
    let second = build_editor_shell(&second_view_model, &theme, &workspace);

    let first_panel_context = first
        .projection_artifacts
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner panel structural context should exist");
    let second_panel_context = second
        .projection_artifacts
        .workspace
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
        .expect("outliner panel structural context should exist");
    assert_eq!(first_panel_context, second_panel_context);

    let first_row_action = first
        .projection_artifacts
        .widget_actions_by_id
        .get(&crate::outliner_row_widget_id(0))
        .cloned()
        .expect("row action should exist");
    let second_row_action = second
        .projection_artifacts
        .widget_actions_by_id
        .get(&crate::outliner_row_widget_id(0))
        .cloned()
        .expect("row action should exist");

    match (first_row_action, second_row_action) {
        (
            RoutedShellAction::SelectOutlinerEntity {
                entity: first_entity,
                context: first_context,
            },
            RoutedShellAction::SelectOutlinerEntity {
                entity: second_entity,
                context: second_context,
            },
        ) => {
            assert_ne!(first_entity, second_entity);
            assert_eq!(first_context, second_context);
        }
        _ => panic!("outliner row action should preserve structural context across rebuild"),
    }
}

#[test]
fn routing_uses_projection_artifact_not_index_or_view_model_decoding() {
    let interactions = UiInteractionResults {
        items: vec![UiInteraction::Activated(crate::outliner_row_widget_id(0))],
    };
    let mut first_view_model = sample_shell_view_model();
    first_view_model.outliner.rows = vec![OutlinerRowViewModel {
        entity: editor_core::EntityId(700),
        display_name: "Primary".to_string(),
        depth: 0,
        is_selected: true,
    }];
    let first_build = build_editor_shell(
        &first_view_model,
        &ThemeTokens::default(),
        &sample_workspace_state(),
    );

    let mut unrelated_view_model = first_view_model.clone();
    unrelated_view_model.outliner.rows[0].entity = editor_core::EntityId(701);
    let _ = unrelated_view_model;

    let commands =
        map_interactions_to_shell_commands(&interactions, &first_build.projection_artifacts);
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        ShellCommand::SelectOutlinerEntity {
            entity: editor_core::EntityId(700),
            projection_epoch: 0,
            ..
        }
    ));
}

fn sample_shell_view_model() -> EditorShellViewModel {
    EditorShellViewModel {
        toolbar: ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(1),
                    stable_name: "select",
                    label: "Select".to_string(),
                    is_active: true,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2),
                    stable_name: "translate",
                    label: "Translate".to_string(),
                    is_active: false,
                    enabled: true,
                },
            ],
        },
        outliner: OutlinerViewModel {
            rows: vec![OutlinerRowViewModel {
                entity: editor_core::EntityId(1),
                display_name: "Player".to_string(),
                depth: 0,
                is_selected: true,
            }],
        },
        entity_table: EntityTableViewModel::default(),
        viewport: ViewportViewModel {
            viewport_id: Some(editor_viewport::ViewportId(1)),
            selected_primary_product_id: Some(editor_viewport::ExpressionProductId(1)),
            product_choices: vec![
                ViewportProductChoiceViewModel {
                    viewport_id: editor_viewport::ViewportId(1),
                    product_id: editor_viewport::ExpressionProductId(1),
                    label: "SceneColor2D".to_string(),
                    selected: true,
                    enabled: true,
                },
                ViewportProductChoiceViewModel {
                    viewport_id: editor_viewport::ViewportId(1),
                    product_id: editor_viewport::ExpressionProductId(2),
                    label: "PickingIds2D".to_string(),
                    selected: false,
                    enabled: true,
                },
            ],
            details_visible: false,
            selected_entity: Some(editor_core::EntityId(1)),
            hovered_entity: None,
            drag_in_progress: false,
            preview_active: false,
        },
        inspector: InspectorViewModel {
            target: InspectorTargetViewModel::Component {
                entity_display_name: "Player".to_string(),
                component_display_name: "LocalTransform".to_string(),
            },
            fields: vec![InspectorFieldViewModel {
                label: "translation.x".to_string(),
                path_key: Some("translation.x".to_string()),
                value_summary: "1.0".to_string(),
                is_focused: false,
                editable: true,
            }],
        },
        console: ConsoleViewModel {
            lines: vec!["boot".to_string()],
        },
    }
}

fn sample_workspace_state() -> crate::WorkspaceState {
    let mut allocator = WorkspaceIdentityAllocator::new();
    let workspace_id = allocator.allocate_workspace_id();
    crate::WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
}

fn scrollable_shell_view_model() -> EditorShellViewModel {
    let mut shell = sample_shell_view_model();
    shell.console = ConsoleViewModel {
        lines: (0..120).map(|index| format!("line-{index:03}")).collect(),
    };
    shell
}

fn workspace_with_console_moved_to_floating_host() -> crate::WorkspaceState {
    let workspace = sample_workspace_state();
    let console_panel_id = workspace
        .panels_by_id
        .values()
        .find(|panel| panel.panel_kind == crate::PanelKind::Console)
        .map(|panel| panel.id)
        .expect("console panel should exist in default workspace");
    let console_stack_id = workspace
        .tab_stacks_by_id
        .values()
        .find(|stack| stack.ordered_panels.contains(&console_panel_id))
        .map(|stack| stack.id)
        .expect("console stack should exist in default workspace");

    let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
    let floating_host_id = allocator.allocate_panel_host_id();
    let floating_stack_id = allocator.allocate_tab_stack_id();

    crate::reduce_workspace(
        &workspace,
        crate::WorkspaceMutation::MovePanelToNewFloatingHost {
            panel_id: console_panel_id,
            source_tab_stack_id: console_stack_id,
            floating_host_id,
            floating_tab_stack_id: floating_stack_id,
            bounds: crate::FloatingHostBounds::new(160.0, 96.0, 520.0, 320.0),
        },
    )
    .expect("moving console panel into floating host should succeed")
}

fn layout_bounds(layouts: &crate::ComputedLayoutMap, id: crate::WidgetId) -> UiRect {
    layouts
        .get(&id)
        .expect("layout should contain widget")
        .bounds
}

fn intersects(a: UiRect, b: UiRect) -> bool {
    a.intersect(b).is_some()
}

fn max_scroll_offset(
    layouts: &crate::ComputedLayoutMap,
    scroll_widget_id: crate::WidgetId,
    list_widget_id: crate::WidgetId,
) -> f32 {
    let scroll = layouts
        .get(&scroll_widget_id)
        .expect("scroll layout should exist");
    let list = layouts
        .get(&list_widget_id)
        .expect("list layout should exist");
    (list.measured_size.height - scroll.content_bounds.height).max(0.0)
}
