use anyhow::{Result, ensure};
use engine::app::NativeHostLaunchOverrides;
use runenwerk_arena::build_game_app;
use std::ffi::OsString;

fn parse_launch<I, S>(args: I) -> Result<NativeHostLaunchOverrides>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let (host, product_args) = NativeHostLaunchOverrides::extract(args)?;
    ensure!(
        product_args.is_empty(),
        "unexpected Arena launch argument '{}'",
        product_args[0].to_string_lossy()
    );
    Ok(host)
}

fn main() -> Result<()> {
    let host = parse_launch(std::env::args_os().skip(1))?;
    let mut app = build_game_app(false);
    host.apply_to(&mut app)?;
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::runtime::FramePacingPolicyResource;

    #[test]
    fn empty_args_preserve_default_game() {
        assert_eq!(parse_launch([] as [&str; 0]).unwrap().frame_pacing(), None);
    }

    #[test]
    fn capped_uncapped_and_on_demand_are_supported() {
        for (mode, policy) in [
            (
                "capped:144",
                FramePacingPolicyResource::continuous_capped(144),
            ),
            ("uncapped", FramePacingPolicyResource::continuous_uncapped()),
            ("on-demand", FramePacingPolicyResource::on_demand()),
        ] {
            assert_eq!(
                parse_launch(["--host-pacing", mode])
                    .unwrap()
                    .frame_pacing(),
                Some(policy)
            );
        }
    }

    #[test]
    fn unknown_invalid_or_duplicate_flags_fail_before_startup() {
        for args in [
            vec!["--unknown"],
            vec!["--host-pacing=capped:0"],
            vec!["--host-pacing"],
            vec!["--"],
            vec!["--host-pacing=uncapped", "--host-pacing=on-demand"],
        ] {
            assert!(parse_launch(args).is_err());
        }
    }
}
