use crate::ui::{ConsoleUiTemplate, UiNodeKind, export_console_template, initialize_console_ui};
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
    logs_width_ratio: Some(0.40),
    show_scroll_indicators: Some(true),
    show_scroll_hints: Some(true),
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
    let layout = tpl.layout.expect("layout should exist");
    assert_eq!(
        layout.panel_width_ratio,
        Some(0.72)
    );
    assert_eq!(
        layout.logs_width_ratio,
        Some(0.40)
    );
    assert_eq!(layout.show_scroll_indicators, Some(true));
    assert_eq!(layout.show_scroll_hints, Some(true));
    assert_eq!(
        tpl.root_style.expect("root style should exist").bg_color,
        Some((0.1, 0.2, 0.3, 1.0))
    );
}

#[test]
fn console_ui_component_tree_parses_from_ron() {
    let raw = r#"
(
  nodes: Some([
    (
      id: "root",
      kind: Some(Panel),
      children: [
        (id: "scrollback", kind: Some(Scrollback)),
        (id: "input", kind: Some(Input)),
        (id: "confirm_button", kind: Some(Button)),
      ],
    ),
  ]),
)
"#;

    let tpl: ConsoleUiTemplate = ron::from_str(raw).expect("component tree should parse");
    let nodes = tpl.nodes.expect("nodes should exist");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, "root");
    assert_eq!(nodes[0].kind, Some(UiNodeKind::Panel));
    assert_eq!(nodes[0].children.len(), 3);
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

#[test]
fn template_component_nodes_apply_by_stable_id() {
    let mut world = World::new();
    let mut ui = initialize_console_ui(&mut world);

    if let Some(input) = world.get_component_mut::<crate::ui::UiText>(ui.input) {
        input.content = "grotto> keep-me".to_string();
    }

    let raw = r#"
(
  nodes: Some([
    (
      id: "root",
      kind: Some(Panel),
      style: Some((bg_color: Some((0.2, 0.2, 0.2, 1.0)))),
      children: [
        (id: "input", kind: Some(Input), text: Some((content: Some("replace?"), size: Some(19.0)))),
        (id: "confirm_button", kind: Some(Button), text: Some((content: Some("Ship")))),
      ],
    ),
  ]),
)
"#;
    let tpl: ConsoleUiTemplate = ron::from_str(raw).expect("template should parse");
    crate::ui::apply_console_template(&mut world, &mut ui, tpl);

    let input = world
        .get_component::<crate::ui::UiText>(ui.input)
        .expect("input text should exist");
    assert_eq!(input.content, "grotto> keep-me");
    assert_eq!(input.size, 19.0);

    let button = world
        .get_component::<crate::ui::UiText>(ui.confirm_button)
        .expect("button text should exist");
    assert_eq!(button.content, "Ship");
}

#[test]
fn template_component_diff_skips_unchanged_nodes() {
    let mut world = World::new();
    let mut ui = initialize_console_ui(&mut world);

    let raw = r#"
(
  nodes: Some([
    (
      id: "root",
      kind: Some(Panel),
      children: [
        (id: "confirm_button", kind: Some(Button), text: Some((content: Some("Ship")))),
      ],
    ),
  ]),
)
"#;
    let tpl: ConsoleUiTemplate = ron::from_str(raw).expect("template should parse");
    crate::ui::apply_console_template(&mut world, &mut ui, tpl.clone());

    if let Some(button) = world.get_component_mut::<crate::ui::UiText>(ui.confirm_button) {
        button.content = "Runtime override".to_string();
    }

    // Same template should not re-apply unchanged node patch.
    crate::ui::apply_console_template(&mut world, &mut ui, tpl);
    let button = world
        .get_component::<crate::ui::UiText>(ui.confirm_button)
        .expect("button text should exist");
    assert_eq!(button.content, "Runtime override");
}

#[test]
fn export_console_template_contains_editor_node_tree() {
    let mut world = World::new();
    let ui = initialize_console_ui(&mut world);

    let template = export_console_template(&world, &ui);
    let nodes = template.nodes.expect("nodes should exist");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, "root");
    assert_eq!(nodes[0].children.len(), 3);
    assert!(template.layout.is_some());
    let layout = template.layout.expect("layout should exist");
    assert!(layout.logs_width_ratio.is_some());
    assert!(layout.logs_height_ratio.is_some());
    assert!(layout.show_scroll_indicators.is_some());
    assert!(layout.show_scroll_hints.is_some());
    assert!(template.confirm_button.is_some());
}
