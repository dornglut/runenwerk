use super::*;

// Owner: Engine Render Shader Registry - Helpers
pub(super) fn build_shader_index(assets: &[ShaderAssetComponent]) -> HashMap<String, usize> {
    assets
        .iter()
        .enumerate()
        .map(|(index, asset)| (asset.id.clone(), index))
        .collect()
}

pub(super) fn normalize_roots<I, S>(roots: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut out = Vec::<String>::new();
    for root in roots {
        let root = root.into();
        let root = root.trim();
        if root.is_empty() {
            continue;
        }
        if !out.iter().any(|value| value == root) {
            out.push(root.to_string());
        }
    }
    if out.is_empty() {
        out.push(DEFAULT_SHADER_ASSET_ROOT.to_string());
    }
    out
}

pub(super) fn discover_shader_files(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("wgsl"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

pub(super) fn derive_shader_id_for_root(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).ok().unwrap_or(path);
    let no_ext = relative.with_extension("");
    normalize_shader_id(no_ext.to_string_lossy())
}

pub(super) fn derive_shader_id_from_path(path: &Path) -> String {
    let no_ext = path.with_extension("");
    normalize_shader_id(no_ext.to_string_lossy())
}

pub(super) fn normalize_shader_id(value: impl AsRef<str>) -> String {
    let raw = value.as_ref();
    let mut id = String::new();
    let mut pending_separator = false;

    for ch in raw.chars() {
        if ch == '/' || ch == '\\' || ch == '.' {
            pending_separator = !id.is_empty() && !id.ends_with('.');
            continue;
        }

        if pending_separator {
            id.push('.');
            pending_separator = false;
        }

        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
        } else if !id.ends_with('_') && !id.ends_with('.') {
            id.push('_');
        }
    }

    id.trim_matches(|ch| ch == '_' || ch == '.').to_string()
}

pub(super) fn shader_event_state_label(kind: ShaderRegistryEventKind) -> &'static str {
    match kind {
        ShaderRegistryEventKind::Discovered => "discovered",
        ShaderRegistryEventKind::Registered => "registered",
        ShaderRegistryEventKind::PathUpdated => "path_updated",
        ShaderRegistryEventKind::DuplicateId => "duplicate_id",
        ShaderRegistryEventKind::Reloaded => "reloaded",
        ShaderRegistryEventKind::SkippedEmpty => "skipped_empty",
        ShaderRegistryEventKind::Failed => "failed",
    }
}


