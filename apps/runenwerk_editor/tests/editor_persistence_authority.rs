const EDITOR_CORE_COMMAND: &str = include_str!("../../../domain/editor/editor_core/src/command.rs");
const EDITOR_CORE_DOCUMENT: &str =
    include_str!("../../../domain/editor/editor_core/src/document.rs");
const EDITOR_CORE_SESSION: &str = include_str!("../../../domain/editor/editor_core/src/session.rs");
const EDITOR_SCENE_RUNTIME: &str =
    include_str!("../../../domain/editor/editor_scene/src/bridge/scene_runtime.rs");
const EDITOR_SHELL_COMMAND: &str =
    include_str!("../../../domain/editor/editor_shell/src/commands/shell_command.rs");
const RUNENWERK_SHELL_DISPATCH: &str = include_str!("../src/shell/dispatch_shell_command.rs");
const SCENE_PERSISTENCE_CONTEXT: &str = include_str!("../src/persistence/scene_context.rs");

#[test]
fn generic_editor_dirty_and_save_authority_stays_retired() {
    for (owner, source, forbidden) in [
        (
            "editor_core command",
            EDITOR_CORE_COMMAND,
            "mark_document_dirty",
        ),
        ("editor_core document", EDITOR_CORE_DOCUMENT, "is_dirty"),
        ("editor_core document", EDITOR_CORE_DOCUMENT, "with_dirty"),
        (
            "editor_core session",
            EDITOR_CORE_SESSION,
            "DirtyDocumentClosePolicy",
        ),
        (
            "editor_core session",
            EDITOR_CORE_SESSION,
            "set_document_dirty",
        ),
        (
            "editor_core session",
            EDITOR_CORE_SESSION,
            "mark_document_dirty",
        ),
        (
            "editor_core session",
            EDITOR_CORE_SESSION,
            "mark_document_saved",
        ),
        ("editor_scene bridge", EDITOR_SCENE_RUNTIME, "EditorSession"),
        (
            "editor_scene bridge",
            EDITOR_SCENE_RUNTIME,
            "mark_document_dirty",
        ),
        (
            "editor_shell command",
            EDITOR_SHELL_COMMAND,
            "SaveDocumentTab",
        ),
        (
            "editor_shell command",
            EDITOR_SHELL_COMMAND,
            "CloseDocumentTab",
        ),
        (
            "Runenwerk shell dispatcher",
            RUNENWERK_SHELL_DISPATCH,
            "DirtyDocumentClosePolicy",
        ),
        (
            "Runenwerk shell dispatcher",
            RUNENWERK_SHELL_DISPATCH,
            "SaveDocumentTab",
        ),
        (
            "Runenwerk shell dispatcher",
            RUNENWERK_SHELL_DISPATCH,
            "CloseDocumentTab",
        ),
    ] {
        assert!(
            !source.contains(forbidden),
            "{owner} must not regain generic persistence authority via {forbidden}"
        );
    }
}

#[test]
fn scene_persistence_context_remains_scene_specific_content_derived_and_app_owned() {
    assert!(SCENE_PERSISTENCE_CONTEXT.contains("ScenePersistenceContext"));
    assert!(SCENE_PERSISTENCE_CONTEXT.contains("SceneFileV2"));
    assert!(SCENE_PERSISTENCE_CONTEXT.contains("Unbound"));
    assert!(SCENE_PERSISTENCE_CONTEXT.contains("Persisted"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("dirty: bool"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("revision:"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("AssetCatalog"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("MaterialLab"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("EditorDefinition"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("CompositionLayout"));
    assert!(!SCENE_PERSISTENCE_CONTEXT.contains("DocumentKind"));
}
