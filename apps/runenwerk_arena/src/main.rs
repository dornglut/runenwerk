use anyhow::{Result, bail, ensure};
use engine::app::NativeHostLaunchOverrides;
use runenwerk_arena::build_game_app;
use std::ffi::OsString;

#[cfg(debug_assertions)]
use anyhow::Context;
#[cfg(debug_assertions)]
use engine::prelude::{ResMut, Update};
#[cfg(debug_assertions)]
use engine::runtime::{FramePacingPolicyResource, NativeWindowId, WindowStateRegistryResource};
#[cfg(debug_assertions)]
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicUsize, Ordering};

const HELP: &str = "Usage: runenwerk_arena [--host-pacing MODE]

Options:
  --host-pacing MODE   capped[:FPS] (1..=240), uncapped, or on-demand
  -h, --help           Print this help text";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Launch {
    Help,
    Run(NativeHostLaunchOverrides),
}

fn parse_launch<I, S>(args: I) -> Result<Launch>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into);
    let mut host = NativeHostLaunchOverrides::default();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("-h" | "--help") => return Ok(Launch::Help),
            _ => {
                if let Some(request) = NativeHostLaunchOverrides::parse_option(&arg, &mut args)? {
                    ensure!(
                        host.frame_pacing().is_none(),
                        "duplicate --host-pacing argument"
                    );
                    host = request;
                } else {
                    bail!(
                        "unexpected Arena launch argument '{}'",
                        arg.to_string_lossy()
                    );
                }
            }
        }
    }
    Ok(Launch::Run(host))
}

#[cfg(debug_assertions)]
#[derive(runen_ecs::Resource)]
struct ArenaNativeLaunchProbe {
    frames: Arc<AtomicUsize>,
}

#[cfg(debug_assertions)]
fn observe_native_launch(
    probe: ResMut<ArenaNativeLaunchProbe>,
    mut windows: ResMut<WindowStateRegistryResource>,
) -> Result<()> {
    let frame = probe.frames.fetch_add(1, Ordering::SeqCst) + 1;
    if frame == 4 {
        windows
            .record_mut(NativeWindowId::primary())
            .context("native Arena primary window missing during Host CLI smoke")?
            .request_close();
    }
    Ok(())
}

fn main() -> Result<()> {
    let Launch::Run(host) = parse_launch(std::env::args_os().skip(1))? else {
        println!("{HELP}");
        return Ok(());
    };
    let mut app = build_game_app(false);
    host.apply_to(&mut app)?;
    #[cfg(debug_assertions)]
    let smoke = std::env::var_os("RUNENWERK_ARENA_NATIVE_LAUNCH_SMOKE").is_some();
    #[cfg(debug_assertions)]
    let observed = Arc::new(AtomicUsize::new(0));
    #[cfg(debug_assertions)]
    if smoke {
        let requested = host
            .frame_pacing()
            .context("native launch smoke requires Host override")?;
        ensure!(
            app.world().resource::<FramePacingPolicyResource>()?.mode == requested.mode,
            "native Arena launch policy was not applied before startup"
        );
        app.init_resource::<WindowStateRegistryResource>();
        app.insert_resource(ArenaNativeLaunchProbe {
            frames: Arc::clone(&observed),
        });
        app.add_systems(Update, observe_native_launch);
    }
    let result = app.run();
    #[cfg(debug_assertions)]
    if smoke {
        result?;
        let frames = observed.load(Ordering::SeqCst);
        ensure!(
            frames >= 4,
            "native Arena launch completed only {frames} frames"
        );
        println!(
            "native_arena_host_cli=pass frames={frames} mode={}",
            host.frame_pacing().unwrap().mode.as_str()
        );
        return Ok(());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::runtime::FramePacingPolicyResource;

    fn run(args: &[&str]) -> Result<NativeHostLaunchOverrides> {
        match parse_launch(args.iter().copied())? {
            Launch::Run(host) => Ok(host),
            Launch::Help => bail!("expected executable launch, got help"),
        }
    }

    #[test]
    fn empty_args_preserve_default_game() {
        assert_eq!(run(&[]).unwrap().frame_pacing(), None);
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
                run(&["--host-pacing", mode]).unwrap().frame_pacing(),
                Some(policy)
            );
        }
        assert_eq!(
            run(&["--host-pacing=capped"]).unwrap().frame_pacing(),
            Some(FramePacingPolicyResource::continuous_capped(60))
        );
    }

    #[test]
    fn help_exits_without_starting_a_window() {
        assert_eq!(parse_launch(["--help"]).unwrap(), Launch::Help);
        assert_eq!(parse_launch(["-h"]).unwrap(), Launch::Help);
    }

    #[test]
    fn unknown_invalid_and_duplicate_flags_fail() {
        for args in [
            vec!["--unknown"],
            vec!["--host-pacing=capped:0"],
            vec!["--host-pacing=capped:241"],
            vec!["--host-pacing"],
            vec!["--host-pacing", "--help"],
            vec!["--host-pacing=uncapped", "--host-pacing=on-demand"],
            vec!["--"],
            vec!["--", "--host-pacing=uncapped"],
        ] {
            assert!(run(&args).is_err(), "accepted invalid Arena args: {args:?}");
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
