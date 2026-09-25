use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    match parse_command(env::args_os().skip(1))? {
        Command::Native => runenwerk_render_lab::run_native(),
        Command::NativeMeasurement {
            output_path,
            submitted_frame_limit,
            window_size_px,
        } => runenwerk_render_lab::run_native_measurement(
            output_path,
            submitted_frame_limit,
            window_size_px,
        ),
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
        window_size_px: Option<(u32, u32)>,
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
        let mut args = args.peekable();
        let output_path = match args.peek() {
            Some(value) if !value.to_string_lossy().starts_with("--") => {
                PathBuf::from(args.next().expect("peeked measurement output path"))
            }
            _ => default_output,
        };
        let mut submitted_frame_limit = None;
        let mut window_size_px = None;
        while let Some(flag) = args.next() {
            if flag == "--submitted-frames" {
                if submitted_frame_limit.is_some() {
                    anyhow::bail!("duplicate --submitted-frames argument");
                }
                submitted_frame_limit = Some(parse_frame_limit(args.next())?);
            } else if flag == "--window-size-px" {
                if window_size_px.is_some() {
                    anyhow::bail!("duplicate --window-size-px argument");
                }
                window_size_px = Some(parse_window_size_px(args.next())?);
            } else {
                anyhow::bail!(
                    "unexpected RL2 measurement argument '{}'",
                    flag.to_string_lossy()
                );
            }
        }
        return Ok(Command::NativeMeasurement {
            output_path,
            submitted_frame_limit,
            window_size_px,
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

fn parse_window_size_px(value: Option<OsString>) -> anyhow::Result<(u32, u32)> {
    let Some(value) = value else {
        anyhow::bail!("--window-size-px requires WIDTHxHEIGHT");
    };
    let Some(value) = value.to_str() else {
        anyhow::bail!("--window-size-px requires a UTF-8 WIDTHxHEIGHT value");
    };
    let Some((width, height)) = value.split_once('x') else {
        anyhow::bail!("invalid --window-size-px value '{value}'; expected WIDTHxHEIGHT");
    };
    let width = width
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --window-size-px width in '{value}'"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --window-size-px height in '{value}'"))?;
    if width == 0 || height == 0 {
        anyhow::bail!("--window-size-px requires positive WIDTHxHEIGHT");
    }
    Ok((width, height))
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
                window_size_px: None,
            }
        );
        assert_eq!(
            parse_command(args(&["--rl2-measure"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: None,
                window_size_px: None,
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
                window_size_px: None,
            }
        );
        assert_eq!(
            parse_command(args(&["--rl2-measure", "--submitted-frames", "420"])).unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: Some(420),
                window_size_px: None,
            }
        );
    }

    #[test]
    fn native_measurement_mode_accepts_explicit_physical_window_size() {
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "evidence/run.json",
                "--window-size-px",
                "1600x1200",
                "--submitted-frames",
                "600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("evidence/run.json"),
                submitted_frame_limit: Some(600),
                window_size_px: Some((1600, 1200)),
            }
        );
        assert_eq!(
            parse_command(args(&[
                "--rl2-measure",
                "--submitted-frames",
                "420",
                "--window-size-px",
                "800x600"
            ]))
            .unwrap(),
            Command::NativeMeasurement {
                output_path: PathBuf::from("render-lab/rl2-measurement.json"),
                submitted_frame_limit: Some(420),
                window_size_px: Some((800, 600)),
            }
        );
    }

    #[test]
    fn native_measurement_mode_rejects_invalid_physical_window_sizes() {
        for values in [
            vec!["--rl2-measure", "--window-size-px"],
            vec!["--rl2-measure", "--window-size-px", "0x1200"],
            vec!["--rl2-measure", "--window-size-px", "1600x0"],
            vec!["--rl2-measure", "--window-size-px", "1600"],
            vec!["--rl2-measure", "--window-size-px", "1600X1200"],
            vec!["--rl2-measure", "--window-size-px", "watx1200"],
            vec![
                "--rl2-measure",
                "--window-size-px",
                "800x600",
                "--window-size-px",
                "1600x1200",
            ],
        ] {
            assert!(parse_command(args(&values)).is_err(), "{values:?}");
        }
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
