use anyhow::{Result, bail, ensure};
use engine::app::NativeHostLaunchOverrides;
use std::ffi::OsString;

mod rendering;
mod runtime;

const HELP: &str = "Usage: boids_render_flow [--species] [--evidence] [--host-pacing MODE]

Options:
  --species            Use species-interaction rules instead of flocking
  --evidence           Print production evidence without launching a window
  --host-pacing MODE   capped[:FPS] (1..=240), uncapped, or on-demand
  -h, --help           Print this help text";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoidsOptions {
    host: NativeHostLaunchOverrides,
    evidence: bool,
    species: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Launch {
    Help,
    Run(BoidsOptions),
}

/// The product owns one argument grammar. Native Host values are parsed only
/// when this grammar encounters its Host option, never in a prior token scan.
fn parse_launch<I, S>(args: I) -> Result<Launch>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into);
    let (mut evidence, mut species) = (false, false);
    let mut host = NativeHostLaunchOverrides::default();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("-h" | "--help") => return Ok(Launch::Help),
            Some("--species") if !species => species = true,
            Some("--evidence") if !evidence => evidence = true,
            Some("--species" | "--evidence") => {
                bail!(
                    "duplicate Boids launch argument '{}'",
                    arg.to_string_lossy()
                )
            }
            _ => {
                if let Some(request) = NativeHostLaunchOverrides::parse_option(&arg, &mut args)? {
                    ensure!(
                        host.frame_pacing().is_none(),
                        "duplicate --host-pacing argument"
                    );
                    host = request;
                } else {
                    bail!(
                        "unexpected Boids launch argument '{}'",
                        arg.to_string_lossy()
                    );
                }
            }
        }
    }
    ensure!(
        !evidence || host.frame_pacing().is_none(),
        "--host-pacing is only valid for interactive Boids, not --evidence"
    );
    Ok(Launch::Run(BoidsOptions {
        host,
        evidence,
        species,
    }))
}

fn main() -> Result<()> {
    let Launch::Run(BoidsOptions {
        host,
        evidence,
        species,
    }) = parse_launch(std::env::args_os().skip(1))?
    else {
        println!("{HELP}");
        return Ok(());
    };
    if evidence {
        println!("{}", rendering::production_evidence_report()?.format_text());
        return Ok(());
    }
    let rule = if species {
        rendering::BoidsRuleFamily::SpeciesInteraction
    } else {
        rendering::BoidsRuleFamily::Flocking
    };
    runtime::run(rule, host)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::runtime::FramePacingPolicyResource;

    fn run(args: &[&str]) -> Result<BoidsOptions> {
        match parse_launch(args.iter().copied())? {
            Launch::Run(options) => Ok(options),
            Launch::Help => bail!("expected an executable launch, got help"),
        }
    }

    #[test]
    fn species_and_host_selection_are_independent() {
        let options = run(&["--species", "--host-pacing=uncapped"]).unwrap();
        assert!(!options.evidence);
        assert!(options.species);
        assert_eq!(
            options.host.frame_pacing(),
            Some(FramePacingPolicyResource::continuous_uncapped())
        );
        let other = run(&["--host-pacing", "capped:120", "--species"]).unwrap();
        assert_eq!(
            other.host.frame_pacing(),
            Some(FramePacingPolicyResource::continuous_capped(120))
        );
    }

    #[test]
    fn empty_args_preserve_default_flocking() {
        let options = run(&[]).unwrap();
        assert!(!options.evidence && !options.species && options.host.frame_pacing().is_none());
    }

    #[test]
    fn help_returns_before_any_product_or_host_startup() {
        assert_eq!(parse_launch(["--help"]).unwrap(), Launch::Help);
        assert_eq!(parse_launch(["-h"]).unwrap(), Launch::Help);
        assert_eq!(parse_launch(["--species", "--help"]).unwrap(), Launch::Help);
    }

    #[test]
    fn evidence_does_not_silently_ignore_native_host_requests() {
        assert!(run(&["--evidence"]).unwrap().evidence);
        assert!(run(&["--evidence", "--host-pacing=uncapped"]).is_err());
        assert!(run(&["--host-pacing", "capped", "--evidence"]).is_err());
    }

    #[test]
    fn strict_duplicate_unknown_value_and_terminator_errors() {
        for args in [
            vec!["--unknown"],
            vec!["--species", "--species"],
            vec!["--evidence", "--evidence"],
            vec!["--host-pacing=uncapped", "--host-pacing=capped"],
            vec!["--host-pacing", "uncapped", "--host-pacing", "uncapped"],
            vec!["--host-pacing"],
            vec!["--host-pacing", "--species"],
            vec!["--host-pacing=capped:0"],
            vec!["--host-pacing=capped:241"],
            vec!["--"],
            vec!["--", "--species"],
        ] {
            assert!(run(&args).is_err(), "accepted invalid Boids args: {args:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_process_arguments_fail_before_startup() {
        use std::os::unix::ffi::OsStringExt;
        let invalid = OsString::from_vec(vec![0xff]);
        assert!(parse_launch([invalid.clone()]).is_err());
        assert!(parse_launch([OsString::from("--host-pacing"), invalid]).is_err());
    }
}
