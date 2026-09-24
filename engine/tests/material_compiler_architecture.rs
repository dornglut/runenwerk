use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn material_compiler_mod_rs_is_public_surface_only() {
    let source = read("src/plugins/render/material_compiler/mod.rs");
    let line_count = source.lines().count();

    assert!(
        line_count <= 90,
        "material_compiler/mod.rs must stay a small public surface; found {line_count} lines"
    );

    for forbidden in [
        "struct WgslProgramCompiler",
        "fn material_program_wgsl",
        "fn material_scene_product_wgsl",
        "CanonicalShaderIdentityEncoder",
        "naga::front::wgsl::parse_str",
        "fn literal_to_wgsl",
    ] {
        assert!(
            !source.contains(forbidden),
            "material_compiler/mod.rs must not own implementation detail: {forbidden}"
        );
    }
}

#[test]
fn material_compiler_keeps_responsibility_modules() {
    let root = Path::new("src/plugins/render/material_compiler");
    for expected in [
        "bindings.rs",
        "diagnostics.rs",
        "identity.rs",
        "types.rs",
        "validation.rs",
        "wgsl/mod.rs",
        "wgsl/literals.rs",
        "wgsl/preview.rs",
        "wgsl/program.rs",
        "wgsl/scene.rs",
    ] {
        assert!(
            root.join(expected).exists(),
            "material compiler responsibility module is missing: {expected}"
        );
    }

    let wgsl_program = read("src/plugins/render/material_compiler/wgsl/program.rs");
    assert!(
        wgsl_program.contains("struct WgslProgramCompiler"),
        "WGSL expression compiler must live in wgsl/program.rs"
    );

    let validation = read("src/plugins/render/material_compiler/validation.rs");
    assert!(
        validation.contains("naga::front::wgsl::parse_str"),
        "WGSL validation must live in validation.rs"
    );
}
