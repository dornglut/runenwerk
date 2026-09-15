use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut args = env::args_os().skip(1);
    if matches!(args.next().as_deref(), Some(value) if value == "--rl2" || value == "--native") {
        return runenwerk_render_lab::run_native();
    }
    let output_root = args
        .next()
        .map(PathBuf::from)
        .or_else(|| std::env::args_os().nth(1).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("render-lab"));
    runenwerk_render_lab::run_founding_direct(&output_root)?;
    Ok(())
}
