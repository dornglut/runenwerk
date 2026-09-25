use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    match parse_command(env::args_os().skip(1))? {
        Command::Native => runenwerk_render_lab::run_native(),
        Command::NativeMeasurement {
            output_path,
            submitted_frame_limit,
        } => runenwerk_render_lab::run_native_measurement(output_path, submitted_frame_limit),
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
    NativeMeasurement {
        output_path: PathBuf,
        submitted_frame_limit: Option<usize>,
    },
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> anyhow::Result<Command> {
    let mut args = args.into_iter();
    let first = args.next();
    if matches!(first.as_deref(), Some(value) if value == "--rl2" || value == "--native") {
        return Ok(Command::Native);
    }
    if matches!(first.as_deref(), Some(value) if value == "--rl2-measure") {
        let default_output = PathBuf::from("render-lab/rl2-measurement.json");
        let next = args.next();
        let (output_path, submitted_frame_limit) = if matches!(next.as_deref(), Some(value) if value == "--submitted-frames")
        {
            (default_output, Some(parse_frame_limit(args.next())?))
        } else {
            let output_path = next.map(PathBuf::from).unwrap_or(default_output);
            let submitted_frame_limit = match args.next() {
                Some(flag) if flag == "--submitted-frames" => Some(parse_frame_limit(args.next())?),
                Some(unexpected) => {
                    anyhow::bail!(
                        "unexpected RL2 measurement argument '{}'",
                        unexpected.to_string_lossy()
                    )
                }
                None => None,
            };
            (output_path, submitted_frame_limit)
        };
        if let Some(unexpected) = args.next() {
            anyhow::bail!(
                "unexpected RL2 measurement argument '{}'",
                unexpected.to_string_lossy()
            );
        }
        return Ok(Command::NativeMeasurement {
            output_path,
            submitted_frame_limit,
        });
    }
    Ok(Command::FoundingDirect(
        first
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("render-lab")),
    ))
}

fn parse_frame_limit(value: Option<OsString>) -> anyhow::Result<usize> {
    let Some(value) = value else {
        anyhow::bail!("--submitted-frames requires a positive submitted-frame count");
    };
    let Some(value) = value.to_str() else {
        anyhow::bail!("--submitted-frames requires a UTF-8 integer");
    };
    let limit = value
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("invalid --submitted-frames value '{value}'"))?;
    if limit == 0 {
        anyhow::bail!("--submitted-frames requires a positive submitted-frame count");
    }
    Ok(limit)
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
            parse_command(args(&[])).unwrap(),
            Command::FoundingDirect("render-lab".into())
        );
    }

    #[test]
    fn one_positional_argument_is_the_output_root() {
        assert_eq!(
            parse_command(args(&["artifacts"])).unwrap(),
            Command::FoundingDirect("artifacts".into())
        );
    }

    #[test]
    fn multiple_positional_arguments_preserve_the_first_root() {
        assert_eq!(
            parse_command(args(&["first", "second"])).unwrap(),
            Command::FoundingDirect("first".into())
        );
    }

    #[test]
    fn native_mode_is_explicit() {
        assert_eq!(parse_command(args(&["--rl2"])).unwrap(), Command::Native);
        assert_eq!(parse_command(args(&["--native"])).unwrap(), Command::Native);
    }

    #[test]
    fn native_measurement_mode_has_explicit_and_default_output_paths() {
        assert_eq!(
            parse_command(args(&["--rl2-measure", "evidence/run.json"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: None,
            }
        );
        assert_eq!(
            parse_command(args(&["--rl2-measure"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: None,
            }
        );
    }

    #[test]
    fn native_measurement_mode_accepts_a_positive_bounded_frame_target() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--submitted-frames",
                "600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: Some(600),
            }
        );
        assert_eq!(
            parse_command(args(&["--rl2-measure", "--submitted-frames", "420"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: Some(420),
            }
        );
    }

    #[test]
    fn native_measurement_mode_rejects_invalid_or_ambiguous_frame_targets() {
        assert!(parse_command(args(&["--rl2-measure", "--submitted-frames"])).is_err());
        assert!(parse_command(args(&["--rl2-measure", "--submitted-frames", "0"])).is_err());
        assert!(parse_command(args(&["--rl2-measure", "--submitted-frames", "nope"])).is_err());
        assert!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--submitted-frames",
                "60",
                "extra"
            ]))
            .is_err()
        );
    }
}
