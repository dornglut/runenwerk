//! Final consumer-side guards for the exact-SHA RunenInput cutover.

use std::fs;
use std::path::{Path, PathBuf};

const RUNEN_INPUT_REV: &str = "b2bf687e8071d19e124ea5b2c8948c49891cc1de";

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn rust_sources(root: &Path, output: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()))
    {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            rust_sources(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
}

fn sources_below(root: &Path) -> String {
    let mut paths = Vec::new();
    rust_sources(root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| read(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn runenwerk_consumes_only_the_accepted_runeninput_revision() {
    let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = engine.parent().expect("engine must be a workspace member");
    let root_manifest = read(&workspace.join("Cargo.toml"));
    let lockfile = read(&workspace.join("Cargo.lock"));

    assert!(root_manifest.contains(&format!(
        "runen-input = {{ git = \"https://github.com/dornglut/runen-input\", rev = \"{RUNEN_INPUT_REV}\" }}"
    )));
    assert!(lockfile.contains(&format!(
        "source = \"git+https://github.com/dornglut/runen-input?rev={RUNEN_INPUT_REV}#{RUNEN_INPUT_REV}\""
    )));

    for manifest in [
        engine.join("Cargo.toml"),
        workspace.join("adapters/native_tablet_input/Cargo.toml"),
        workspace.join("apps/runenwerk_draw/Cargo.toml"),
        workspace.join("apps/runenwerk_editor/Cargo.toml"),
    ] {
        assert!(
            read(&manifest).contains("runen-input.workspace = true"),
            "{} must declare its direct RunenInput dependency",
            manifest.display()
        );
    }
}

#[test]
fn predecessor_input_authority_and_forwarding_namespace_are_absent() {
    let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = engine.parent().expect("engine must be a workspace member");

    assert!(!engine.join("src/plugins/input/neutral.rs").exists());

    let input_mod = read(&engine.join("src/plugins/input/mod.rs"));
    let input_domain = read(&engine.join("src/plugins/input/domain.rs"));
    for source in [&input_mod, &input_domain] {
        assert!(!source.contains("mod neutral"));
        assert!(!source.contains("pub use runen_input"));
        assert!(!source.contains("pub use neutral"));
    }

    let input_sources = sources_below(&engine.join("src/plugins/input"));
    for retired in [
        "NeutralInputAuthority",
        "pub struct ControlId",
        "pub enum DigitalTransition",
    ] {
        assert!(
            !input_sources.contains(retired),
            "Runenwerk retained predecessor reducer authority {retired}"
        );
    }

    let direct_consumers = [
        engine.join("src/plugins/input/state.rs"),
        engine.join("src/plugins/input/actions_and_bindings.rs"),
        engine.join("src/runtime/platform.rs"),
        engine.join("src/runtime/winit_input.rs"),
        workspace.join("adapters/native_tablet_input/src/model.rs"),
        workspace.join("adapters/native_tablet_input/src/mapping.rs"),
        workspace.join("apps/runenwerk_draw/src/runtime/resources.rs"),
        workspace.join("apps/runenwerk_draw/src/runtime/systems.rs"),
        workspace.join("apps/runenwerk_editor/src/runtime/composition/input_adapter.rs"),
        workspace.join("apps/runenwerk_editor/src/runtime/composition/input_adapter/state.rs"),
    ];
    for source in direct_consumers {
        assert!(
            read(&source).contains("runen_input"),
            "{} must consume RunenInput directly",
            source.display()
        );
    }
}
