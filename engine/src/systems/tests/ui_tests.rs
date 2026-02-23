use crate::systems::ui::EditorNodeRect;
use crate::systems::ui::WrappedEditorRow;
use crate::systems::ui::adaptive_footer_metrics;
use crate::systems::ui::build_scrollback_view_text;
use crate::systems::ui::build_visible_multiline_input;
use crate::systems::ui::estimate_text_width;
use crate::systems::ui::move_cursor_vertical;
use crate::systems::ui::pick_editor_node_at;
use crate::systems::ui::point_in_rect;
use crate::systems::ui::scrollback_line_style;
use crate::systems::ui::snap_to_grid;
use crate::systems::ui::visible_line_capacity;
use crate::systems::ui::wrap_editor_rows;
use crate::ui::EditorBuffer;
use crate::ui::UiEditorNode;
use crate::ui::UiTextMetrics;
use crate::ui::UiTransform;
use std::collections::HashMap;

fn mono_metrics() -> UiTextMetrics {
    UiTextMetrics {
        glyphs: HashMap::new(),
        base_size: 10.0,
        fallback_advance: 10.0,
    }
}

#[test]
fn point_in_rect_bounds_check_works() {
    let rect = UiTransform {
        x: 10.0,
        y: 10.0,
        w: 20.0,
        h: 20.0,
    };
    assert!(point_in_rect((10.0, 10.0), &rect));
    assert!(point_in_rect((30.0, 30.0), &rect));
    assert!(!point_in_rect((31.0, 30.0), &rect));
    assert!(!point_in_rect((9.0, 11.0), &rect));
}

#[test]
fn estimate_text_width_scales_with_size_and_length() {
    let w_small = estimate_text_width("Confirm", 12.0);
    let w_large = estimate_text_width("Confirm", 24.0);
    let w_long = estimate_text_width("ConfirmConfirm", 12.0);

    assert!(w_large > w_small);
    assert!(w_long > w_small);
}

#[test]
fn wrap_editor_rows_keeps_explicit_empty_lines() {
    let rows = wrap_editor_rows("ab\n\ncd", &mono_metrics(), 10.0, 220.0);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0],
        WrappedEditorRow {
            start_char: 0,
            end_char: 2
        }
    );
    assert_eq!(
        rows[1],
        WrappedEditorRow {
            start_char: 3,
            end_char: 3
        }
    );
    assert_eq!(
        rows[2],
        WrappedEditorRow {
            start_char: 4,
            end_char: 6
        }
    );
}

#[test]
fn visible_multiline_input_wraps_with_prompt_budget_on_first_row() {
    let mut editor = EditorBuffer {
        text: "abcdefghijklmno".to_string(),
        cursor_chars: 15,
        ..EditorBuffer::default()
    };
    let layout = build_visible_multiline_input(&mut editor, &mono_metrics(), 10.0, 120.0, 80.0);
    let lines: Vec<&str> = layout.content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "grotto> abcd");
    assert_eq!(lines[1], "efghijklmno");
}

#[test]
fn move_cursor_vertical_preserves_visual_column_between_wrapped_rows() {
    let metrics = mono_metrics();
    let mut editor = EditorBuffer {
        text: "abcdefghijklmnopqrstuvwxyz12".to_string(),
        cursor_chars: 9,
        ..EditorBuffer::default()
    };

    assert!(move_cursor_vertical(
        &mut editor,
        &metrics,
        10.0,
        120.0,
        true
    ));
    assert_eq!(editor.cursor_chars, 21);
    assert!(move_cursor_vertical(
        &mut editor,
        &metrics,
        10.0,
        120.0,
        false
    ));
    assert_eq!(editor.cursor_chars, 9);
}

#[test]
fn multiline_input_viewport_tracks_caret_row() {
    let mut editor = EditorBuffer {
        text: "abcdefghijklmnopqrstuvwxyz1234567890ABCD".to_string(),
        cursor_chars: 40,
        ..EditorBuffer::default()
    };
    let layout = build_visible_multiline_input(&mut editor, &mono_metrics(), 10.0, 120.0, 25.0);
    let lines: Vec<&str> = layout.content.lines().collect();

    assert_eq!(layout.visible_rows, 2);
    assert_eq!(editor.viewport_row, 2);
    assert_eq!(lines.len(), 2);
    assert!(!lines[0].starts_with("grotto> "));
}

#[test]
fn visible_line_capacity_has_minimum_one() {
    assert_eq!(visible_line_capacity(1.0, 24.0), 1);
    assert!(visible_line_capacity(400.0, 16.0) > 1);
}

#[test]
fn build_scrollback_view_text_takes_visible_window_from_bottom_offset() {
    let lines = vec![
        "l1".to_string(),
        "l2".to_string(),
        "l3".to_string(),
        "l4".to_string(),
    ];
    let view = build_scrollback_view_text(&lines, 1, 2, 200.0, 14.0);
    assert_eq!(view, "l2\nl3");
}

#[test]
fn scrollback_line_style_applies_category_colors() {
    let default = [0.5, 0.5, 0.5, 1.0];
    let (combat_color, combat_text) = scrollback_line_style("[combat] hit for 12", default);
    assert_eq!(combat_text, "hit for 12");
    assert_ne!(combat_color, default);

    let (normal_color, normal_text) = scrollback_line_style("grotto> hello", default);
    assert_eq!(normal_text, "grotto> hello");
    assert_eq!(normal_color, default);
}

#[test]
fn adaptive_footer_metrics_keep_controls_inside_footer_width() {
    let footer_w = 90.0;
    let (_, _, button_w, input_w) =
        adaptive_footer_metrics(footer_w, 120.0, 10.0, 1.0, 40.0, 28.0, 100.0, 8.0);
    assert!(button_w > 0.0);
    assert!(input_w > 0.0);
    assert!(button_w + input_w <= footer_w);
}

#[test]
fn pick_editor_node_at_prefers_topmost_overlap() {
    let nodes = vec![
        EditorNodeRect {
            node: UiEditorNode::Root,
            z: 0,
            rect: UiTransform {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
            },
        },
        EditorNodeRect {
            node: UiEditorNode::Input,
            z: 2,
            rect: UiTransform {
                x: 10.0,
                y: 10.0,
                w: 60.0,
                h: 30.0,
            },
        },
    ];

    assert_eq!(
        pick_editor_node_at((20.0, 20.0), &nodes),
        Some(UiEditorNode::Input)
    );
    assert_eq!(
        pick_editor_node_at((5.0, 5.0), &nodes),
        Some(UiEditorNode::Root)
    );
    assert_eq!(pick_editor_node_at((200.0, 200.0), &nodes), None);
}

#[test]
fn snap_to_grid_rounds_to_nearest_step() {
    assert_eq!(snap_to_grid(12.0, 10.0), 10.0);
    assert_eq!(snap_to_grid(15.0, 10.0), 20.0);
    assert_eq!(snap_to_grid(24.9, 10.0), 20.0);
}
