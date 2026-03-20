use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn hard_cutoff_removes_legacy_render_symbols_and_fallbacks() {
    let forbidden_symbols = [
        "RenderFrameResourceBindings",
        "storage_state(",
        "packet.surface_size.0 + workgroup[0] - 1",
        "packet.surface_size.1 + workgroup[1] - 1",
    ];

    let files = [
        "src/plugins/render/mod.rs",
        "src/plugins/render/plugin.rs",
        "src/plugins/render/api/passes.rs",
        "src/plugins/render/composition/integration.rs",
        "src/plugins/render/renderer/submit.rs",
        "src/plugins/render/renderer/render_flow.rs",
    ];

    for file in files {
        let source = read(file);
        for symbol in forbidden_symbols {
            assert!(
                !source.contains(symbol),
                "legacy cutoff guard hit in '{file}': found forbidden symbol '{symbol}'"
            );
        }
    }

    let render_flow = read("src/plugins/render/renderer/render_flow.rs");
    assert!(
        render_flow.contains("must declare explicit dispatch_workgroups"),
        "compute runtime path should enforce explicit dispatch contract"
    );
}
