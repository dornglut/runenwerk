use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let output_root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("render-lab"));
    runenwerk_render_lab::run_founding_direct(&output_root)?;
    Ok(())
}
