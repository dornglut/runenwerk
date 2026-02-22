use crate::ui::{ConsoleUiTemplate, initialize_console_ui};
use ecs::World;

#[test]
fn console_ui_template_parses_from_ron() {
    let raw = r#"
(
  max_lines: Some(250),
  root_style: Some((
    bg_color: Some((0.1, 0.2, 0.3, 1.0)),
  )),
  layout: Some((
    panel_width_ratio: Some(0.72),
    outer_margin: Some(20.0),
  )),
  confirm_button: Some((
    text: Some((
      content: Some("Run"),
    )),
  )),
)
"#;

    let tpl: ConsoleUiTemplate = ron::from_str(raw).expect("template should parse");
    assert_eq!(tpl.max_lines, Some(250));
    assert_eq!(
        tpl.layout
            .expect("layout should exist")
            .panel_width_ratio,
        Some(0.72)
    );
    assert_eq!(
        tpl.root_style
            .expect("root style should exist")
            .bg_color,
        Some((0.1, 0.2, 0.3, 1.0))
    );
}

#[test]
fn template_apply_preserves_runtime_input_content() {
    let mut world = World::new();
    let mut ui = initialize_console_ui(&mut world);

    if let Some(input) = world.get_component_mut::<crate::ui::UiText>(ui.input) {
        input.content = "grotto> user typing".to_string();
    }

    let raw = r#"
(
  input_text: Some((
    content: Some("should-not-overwrite"),
    size: Some(18.0),
  )),
  confirm_button: Some((
    text: Some((
      content: Some("Execute"),
    )),
  )),
)
"#;
    let tpl: ConsoleUiTemplate = ron::from_str(raw).expect("template should parse");
    crate::ui::apply_console_template(&mut world, &mut ui, tpl);

    let input = world
        .get_component::<crate::ui::UiText>(ui.input)
        .expect("input text should exist");
    assert_eq!(input.content, "grotto> user typing");
    assert_eq!(input.size, 18.0);

    let button = world
        .get_component::<crate::ui::UiText>(ui.confirm_button)
        .expect("button text should exist");
    assert_eq!(button.content, "Execute");
}
