use anyhow::{Result, bail, ensure};
use engine::app::NativeHostLaunchOverrides;
use std::ffi::OsString;

mod rendering;
mod runtime;

fn parse_launch<I, S>(args: I) -> Result<(NativeHostLaunchOverrides, bool, bool)>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let (host, product_args) = NativeHostLaunchOverrides::extract(args)?;
    let (mut evidence, mut species) = (false, false);
    for arg in product_args {
        match arg.to_str() {
            Some("--evidence") if !evidence => evidence = true,
            Some("--species") if !species => species = true,
            Some("--evidence") | Some("--species") => {
                bail!(
                    "duplicate Boids launch argument '{}'",
                    arg.to_string_lossy()
                );
            }
            _ => bail!(
                "unexpected Boids launch argument '{}'",
                arg.to_string_lossy()
            ),
        }
    }
    ensure!(
        !evidence || host.frame_pacing().is_none(),
        "--host-pacing is only valid for interactive Boids, not --evidence"
    );
    Ok((host, evidence, species))
}

fn main() -> Result<()> {
    let (host, evidence, species) = parse_launch(std::env::args_os().skip(1))?;
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

    #[test]
    fn species_and_host_selection_are_independent() {
        let (host, evidence, species) =
            parse_launch(["--species", "--host-pacing=uncapped"]).unwrap();
        assert!(!evidence);
        assert!(species);
        assert_eq!(
            host.frame_pacing(),
            Some(FramePacingPolicyResource::continuous_uncapped())
        );
    }

    #[test]
    fn empty_args_preserve_default_flocking() {
        let (host, evidence, species) = parse_launch([] as [&str; 0]).unwrap();
        assert!(!evidence && !species && host.frame_pacing().is_none());
    }

    #[test]
    fn evidence_does_not_silently_ignore_native_host_requests() {
        assert!(parse_launch(["--evidence"]).unwrap().1);
        assert!(parse_launch(["--evidence", "--host-pacing=uncapped"]).is_err());
    }

    #[test]
    fn product_unknown_duplicates_and_double_dash_fail() {
        for args in [
            vec!["--unknown"],
            vec!["--species", "--species"],
            vec!["--evidence", "--evidence"],
            vec!["--", "--species"],
            vec!["--species", "--host-pacing=capped:0"],
        ] {
            assert!(parse_launch(args).is_err());
        }
    }
}
