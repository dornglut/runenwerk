
use anyhow::{Context, Result};
use engine::{
	App,
	app::{AppNativeHostExt, NativePrimaryClosePolicy},
	plugins::default_plugins,
};
use std::path::Path;

use crate::project;

pub fn run(project_root: &Path) -> Result<()> {
	let manifest = project::load(project_root)?;

	let mut app = App::new();

	app.set_title(manifest.window.title);
	app.add_plugins(default_plugins());

	app.with_primary_close_policy(
		NativePrimaryClosePolicy::ApproveUnhandled,
	)?;

	app.run().context("Runenwerk runtime failed")
}
