
mod project;
mod runtime;

use anyhow::{Context, Result, bail};
use std::{
    env,
    path::Path,
    process::{self, Command},
};

fn main() {
    if let Err(error) = execute() {
        eprintln!("rune: {error:#}");
        process::exit(1);
    }
}

fn execute() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => {
            print_help();
            Ok(())
        }

        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            Ok(())
        }

        [command, name] if command == "new" => {
            let parent = env::current_dir()?;
            let root = project::create(&parent, name)?;

            println!("Created {}", root.display());
            println!("Next:\n  cd {name}\n  rune run");

            Ok(())
        }

        [command] if command == "run" => run_project(),

        // Internal child-process entry point.
        [command, root] if command == "__runtime" => {
            runtime::run(Path::new(root))
        }

        _ => bail!("unknown command; use rune --help"),
    }
}

fn run_project() -> Result<()> {
    let project_root = env::current_dir()?;
    let executable = env::current_exe()?;

    let status = Command::new(executable)
      .arg("__runtime")
      .arg(project_root)
      .status()
      .context("failed to launch Runenwerk runtime")?;

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn print_help() {
    println!(
        "Runenwerk CLI\n\n\
         Usage:\n\
           rune new <name>   Create a project\n\
           rune run          Run the current project"
    );
}
