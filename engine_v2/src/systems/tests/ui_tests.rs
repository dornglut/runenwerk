use crate::systems::ui::build_scrollback_view_text;
use crate::systems::ui::estimate_text_width;
use crate::systems::ui::point_in_rect;
use crate::systems::ui::visible_input_text;
use crate::systems::ui::visible_line_capacity;
use crate::ui::UiTransform;

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
fn visible_input_text_keeps_prompt_and_tail() {
    let full = "grotto> this is a very long command";
    let clipped = visible_input_text(full, 14.0, 120.0);
    assert!(clipped.starts_with("grotto> "));
    assert!(clipped.len() < full.len());
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
    let view = build_scrollback_view_text(&lines, 1, 2);
    assert_eq!(view, "l2\nl3");
}
