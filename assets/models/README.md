# Model Assets

Place `.blend`, `.glb`, or `.gltf` files in this folder.

When a `.blend` file is present, the engine treats it as source-of-truth and auto-exports a sibling `.glb` (same stem) using Blender CLI (`blender -b`). The generated `.glb` is then imported into the runtime model pipeline.

If both `.glb` and `.gltf` exist with the same file stem, `.glb` is preferred.

Blender executable lookup order:
1. `BLENDER_BIN` environment variable
2. `assets/editor/config.ron` field `blender_bin`
3. Common macOS install paths (system app, user app, Steam app)
4. `blender` from `PATH`

The engine imports these models both as:
- SDF proxy volumes (bounding spheres per primitive) for world SDF/raymarch integration.
- Triangle meshes for a runtime mesh overlay render pass.

Current mesh material support:
- PBR base color factor.
- Base color texture (UV set from glTF material binding).

Runtime commands:
- `models`
- `reload_models`
- `model_watch <on|off>`
- `model_status`
