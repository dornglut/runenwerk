# rune — experimental Runenwerk CLI

This is a **prototype**, not an accepted stable project format or a replacement for standalone Cargo applications. See [#1460](https://github.com/dornglut/runenwerk/issues/1460) for the bounded UI proof and [#1393](https://github.com/dornglut/runenwerk/issues/1393) for the unresolved developer-tooling/product decision.

## From a Runenwerk checkout

```sh
cargo install --path apps/rune --locked --force
cd /path/to/projects
rune new hello-rune
cd hello-rune
rune run
```

`rune new` generates an `app.ron` and a `ui/main.ron` containing a static label. The UI file is an existing `ui_definition::AuthoredUiTemplate` with a `UiNodeDefinition` root, serialized without a CLI-specific schema.

Edit `app.ron` to change the native window title or edit `ui/main.ron` to change the label, then close and restart `rune run`. The initial native UI-only presentation uses a white inherited text foreground against the Engine's black clear; this is a provisional CLI presentation choice, not a global RunenUI default or an authored theme system. **Live reload is not implemented.** Project files are interpreted at startup; no project-specific Rust build is needed.

Minimal manifest shape (schema version 1, experimental):

```ron
(
    schema_version: 1,
    name: "hello-rune",
    window: (title: "Hello Rune"),
    ui: Some((source: "ui/main.ron")),
)
```

The `ui` field is optional for older window-only projects. UI paths must be relative, resolve within the project and refer to readable RON documents. The current UI adapter supports only a bounded subset of UI nodes; this proof generates a `Label` and explicitly rejects nested templates and menus. No button actions, application state, multiple windows, developer server, packaging, or hot reload are provided.

`rune run` starts an Engine process via the same installed executable's internal runtime mode. The CLI remains necessary to launch RON-only projects; these are not independent compiled Cargo applications.

## Checks

```sh
cargo test -p rune --locked
cargo clippy -p rune --all-targets --locked -- -D warnings
cargo validate
```

These commands and an actual native-window visual test remain required before accepting this prototype.
