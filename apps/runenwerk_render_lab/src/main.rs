use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    match parse_command(env::args_os().skip(1)) {
        Command::Native => runenwerk_render_lab::run_native(),
        Command::FoundingDirect(output_root) => {
            runenwerk_render_lab::run_founding_direct(output_root)?;
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    FoundingDirect(PathBuf),
    Native,
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> Command {
    let mut args = args.into_iter();
    let first = args.next();
    if matches!(first.as_deref(), Some(value) if value == "--rl2" || value == "--native") {
        return Command::Native;
    }
    Command::FoundingDirect(
        first
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("render-lab")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn zero_arguments_keep_the_default_output_root() {
        assert_eq!(
            parse_command(args(&[])),
            Command::FoundingDirect("render-lab".into())
        );
    }

    #[test]
    fn one_positional_argument_is_the_output_root() {
        assert_eq!(
            parse_command(args(&["artifacts"])),
            Command::FoundingDirect("artifacts".into())
        );
    }

    #[test]
    fn multiple_positional_arguments_preserve_the_first_root() {
        assert_eq!(
            parse_command(args(&["first", "second"])),
            Command::FoundingDirect("first".into())
        );
    }

    #[test]
    fn native_mode_is_explicit() {
        assert_eq!(parse_command(args(&["--rl2"])), Command::Native);
        assert_eq!(parse_command(args(&["--native"])), Command::Native);
    }
}
