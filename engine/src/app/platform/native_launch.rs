//! Ephemeral, typed ingress for native Host command-line requests.
//! Effective configuration remains in the owning App/Host resource.
use crate::app::App;
use crate::app::domain::host::AppHostSelection;
use crate::app::platform::windowed::AppNativeHostExt;
use crate::runtime::frame_pacing::{DEFAULT_ANIMATION_FPS, FramePacingPolicyResource};
use anyhow::{Context, Result, bail, ensure};
use std::ffi::{OsStr, OsString};

/// Extracted launch requests are not persistent application configuration state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeHostLaunchOverrides {
    frame_pacing: Option<FramePacingPolicyResource>,
}

impl NativeHostLaunchOverrides {
    pub fn frame_pacing(self) -> Option<FramePacingPolicyResource> {
        self.frame_pacing
    }

    /// Extract --host-pacing MODE or --host-pacing=MODE. All unrelated arguments
    /// remain unchanged for the application parser. "--" ends Host flag parsing.
    pub fn extract<I, S>(args: I) -> Result<(Self, Vec<OsString>)>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut args = args.into_iter().map(|arg| -> OsString { arg.into() });
        let mut request = Self::default();
        let mut remaining = Vec::new();
        let mut stop = false;
        while let Some(arg) = args.next() {
            if stop {
                remaining.push(arg);
                continue;
            }
            if arg == OsStr::new("--") {
                stop = true;
                remaining.push(arg);
                continue;
            }
            let value = if arg == OsStr::new("--host-pacing") {
                Some(args.next().context("--host-pacing requires MODE")?)
            } else {
                arg.to_str()
                    .and_then(|value| value.strip_prefix("--host-pacing="))
                    .map(OsString::from)
            };
            if let Some(value) = value {
                ensure!(
                    request.frame_pacing.is_none(),
                    "duplicate --host-pacing argument"
                );
                let mode = value.to_str().context("--host-pacing MODE must be UTF-8")?;
                request.frame_pacing = Some(parse_pacing(mode)?);
            } else {
                remaining.push(arg);
            }
        }
        Ok((request, remaining))
    }

    /// Use after programmatic defaults and before App startup. An omitted
    /// launch flag leaves the selected App configuration unchanged.
    pub fn apply_to(self, app: &mut App) -> Result<()> {
        if let Some(policy) = self.frame_pacing {
            ensure!(
                matches!(app.host_selection, AppHostSelection::NativeWindow),
                "--host-pacing requires a native-window App"
            );
            ensure!(
                app.lifecycle.is_configuring(),
                "--host-pacing requires an App that is still configuring"
            );
            app.with_frame_pacing(policy);
        }
        Ok(())
    }
}

fn parse_pacing(mode: &str) -> Result<FramePacingPolicyResource> {
    match mode {
        "capped" => Ok(FramePacingPolicyResource::continuous_capped(
            DEFAULT_ANIMATION_FPS,
        )),
        "uncapped" => Ok(FramePacingPolicyResource::continuous_uncapped()),
        "on-demand" => Ok(FramePacingPolicyResource::on_demand()),
        _ => {
            let Some(raw_fps) = mode.strip_prefix("capped:") else {
                bail!(
                    "invalid --host-pacing mode '{mode}'; expected capped[:FPS], uncapped, on-demand"
                );
            };
            let fps: u32 = raw_fps
                .parse()
                .with_context(|| format!("invalid --host-pacing capped FPS '{raw_fps}'"))?;
            FramePacingPolicyResource::checked_continuous_capped(fps)
                .with_context(|| format!("--host-pacing capped FPS {fps} is outside 1..=240"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame_pacing::{FramePacingMode, FramePacingRuntimeStateResource};

    fn parse(args: &[&str]) -> Result<(NativeHostLaunchOverrides, Vec<OsString>)> {
        NativeHostLaunchOverrides::extract(args.iter().copied())
    }

    #[test]
    fn absent_override_preserves_programmatic_configuration() {
        let (request, remaining) = parse(&["--species", "--custom=x"]).unwrap();
        assert_eq!(request.frame_pacing(), None);
        assert_eq!(remaining, ["--species", "--custom=x"]);
        let mut app = App::new();
        app.with_frame_pacing(FramePacingPolicyResource::on_demand());
        request.apply_to(&mut app).unwrap();
        assert_eq!(
            app.world()
                .resource::<FramePacingPolicyResource>()
                .unwrap()
                .mode,
            FramePacingMode::OnDemand
        );
    }

    #[test]
    fn all_native_host_modes_override_defaults_and_reach_runtime_observation() {
        for (mode, expected) in [
            (
                "capped",
                FramePacingMode::ContinuousCapped { target_fps: 60 },
            ),
            (
                "capped:1",
                FramePacingMode::ContinuousCapped { target_fps: 1 },
            ),
            (
                "capped:120",
                FramePacingMode::ContinuousCapped { target_fps: 120 },
            ),
            (
                "capped:240",
                FramePacingMode::ContinuousCapped { target_fps: 240 },
            ),
            ("uncapped", FramePacingMode::ContinuousUncapped),
            ("on-demand", FramePacingMode::OnDemand),
        ] {
            let (request, rest) = parse(&["--host-pacing", mode]).unwrap();
            assert!(rest.is_empty());
            let mut app = App::new();
            app.with_frame_pacing(FramePacingPolicyResource::on_demand());
            request.apply_to(&mut app).unwrap();
            app.prepare_windowed_frame_pacing();
            assert_eq!(
                app.world()
                    .resource::<FramePacingPolicyResource>()
                    .unwrap()
                    .mode,
                expected,
                "{mode}"
            );
            assert_eq!(
                app.world()
                    .resource::<FramePacingRuntimeStateResource>()
                    .unwrap()
                    .mode,
                expected,
                "{mode}"
            );
        }
    }

    #[test]
    fn equals_form_and_foreign_product_arguments_remain_independent() {
        let (request, rest) =
            parse(&["--species", "--host-pacing=uncapped", "--evidence"]).unwrap();
        assert_eq!(
            request.frame_pacing(),
            Some(FramePacingPolicyResource::continuous_uncapped())
        );
        assert_eq!(rest, ["--species", "--evidence"]);
    }

    #[test]
    fn missing_invalid_duplicate_and_out_of_range_requests_fail() {
        for args in [
            vec!["--host-pacing"],
            vec!["--host-pacing="],
            vec!["--host-pacing=bogus"],
            vec!["--host-pacing=capped:"],
            vec!["--host-pacing=capped:0"],
            vec!["--host-pacing=capped:241"],
            vec!["--host-pacing=capped:-1"],
            vec!["--host-pacing=capped:abc"],
            vec!["--host-pacing", "uncapped", "--host-pacing=on-demand"],
        ] {
            assert!(parse(&args).is_err(), "accepted invalid args: {args:?}");
        }
    }

    #[test]
    fn double_dash_prevents_host_flag_capture() {
        let (request, remaining) =
            parse(&["--host-pacing=uncapped", "--", "--host-pacing=on-demand"]).unwrap();
        assert_eq!(
            request.frame_pacing(),
            Some(FramePacingPolicyResource::continuous_uncapped())
        );
        assert_eq!(remaining, ["--", "--host-pacing=on-demand"]);
    }

    #[test]
    fn headless_request_is_rejected_without_partial_mutation() {
        let (request, _) = parse(&["--host-pacing=uncapped"]).unwrap();
        let mut app = App::headless();
        assert!(request.apply_to(&mut app).is_err());
        assert!(app.world().resource::<FramePacingPolicyResource>().is_err());
        let (absent, _) = parse(&[]).unwrap();
        absent.apply_to(&mut app).unwrap();
    }

    #[test]
    fn post_configuration_override_cannot_mutate_host_policy() {
        let (request, _) = parse(&["--host-pacing=uncapped"]).unwrap();
        let mut app = App::new();
        app.prepare_lifecycle_for_execution().unwrap();
        assert!(request.apply_to(&mut app).is_err());
        assert!(app.world().resource::<FramePacingPolicyResource>().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn preserve_non_utf8_product_args_but_reject_non_utf8_mode() {
        use std::os::unix::ffi::OsStringExt;
        let raw = OsString::from_vec(vec![0xff]);
        let (_, rest) = NativeHostLaunchOverrides::extract([raw.clone()]).unwrap();
        assert_eq!(rest, [raw.clone()]);
        assert!(
            NativeHostLaunchOverrides::extract([OsString::from("--host-pacing"), raw]).is_err()
        );
    }
}
