//! Ephemeral, typed ingress for native Host command-line requests.
//! Effective configuration remains in the owning App/Host resource.
use crate::app::App;
use crate::app::domain::host::AppHostSelection;
use crate::app::platform::windowed::AppNativeHostExt;
use crate::runtime::frame_pacing::{DEFAULT_ANIMATION_FPS, FramePacingPolicyResource};
use anyhow::{Context, Result, bail, ensure};
use std::ffi::{OsStr, OsString};

/// An optional launch request is not persistent application configuration state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeHostLaunchOverrides {
    frame_pacing: Option<FramePacingPolicyResource>,
}

impl NativeHostLaunchOverrides {
    pub fn frame_pacing(self) -> Option<FramePacingPolicyResource> {
        self.frame_pacing
    }

    /// Parse one Host option when the product's own grammar reaches it.
    /// Product value-taking options must consume their values first.
    /// No unrelated token is scanned or consumed.
    pub fn parse_option<I>(arg: &OsStr, args: &mut I) -> Result<Option<Self>>
    where
        I: Iterator<Item = OsString>,
    {
        let value = if arg == OsStr::new("--host-pacing") {
            Some(args.next().context("--host-pacing requires MODE")?)
        } else {
            arg.to_str()
                .and_then(|s| s.strip_prefix("--host-pacing="))
                .map(OsString::from)
        };
        value
            .map(|mode| -> Result<Self> {
                mode.to_str()
                    .context("--host-pacing MODE must be UTF-8")?
                    .parse()
            })
            .transpose()
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

impl std::str::FromStr for NativeHostLaunchOverrides {
    type Err = anyhow::Error;

    fn from_str(mode: &str) -> Result<Self> {
        Ok(Self {
            frame_pacing: Some(parse_pacing(mode)?),
        })
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

    #[test]
    fn absent_override_preserves_programmatic_configuration() {
        let mut empty = std::iter::empty();
        assert_eq!(
            NativeHostLaunchOverrides::parse_option(OsStr::new("--species"), &mut empty).unwrap(),
            None
        );
        let mut app = App::new();
        app.with_frame_pacing(FramePacingPolicyResource::on_demand());
        NativeHostLaunchOverrides::default()
            .apply_to(&mut app)
            .unwrap();
        assert_eq!(
            app.world()
                .resource::<FramePacingPolicyResource>()
                .unwrap()
                .mode,
            FramePacingMode::OnDemand
        );
    }

    #[test]
    fn modes_override_programmatic_defaults_and_reach_runtime() {
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
            let request: NativeHostLaunchOverrides = mode.parse().unwrap();
            let mut app = App::new();
            app.with_frame_pacing(FramePacingPolicyResource::on_demand());
            request.apply_to(&mut app).unwrap();
            app.prepare_windowed_frame_pacing();
            assert_eq!(
                app.world()
                    .resource::<FramePacingPolicyResource>()
                    .unwrap()
                    .mode,
                expected
            );
            assert_eq!(
                app.world()
                    .resource::<FramePacingRuntimeStateResource>()
                    .unwrap()
                    .mode,
                expected
            );
        }
    }

    #[test]
    fn option_parser_supports_both_forms_and_strict_values() {
        let mut values = [OsString::from("uncapped")].into_iter();
        assert_eq!(
            NativeHostLaunchOverrides::parse_option(OsStr::new("--host-pacing"), &mut values)
                .unwrap()
                .unwrap()
                .frame_pacing(),
            Some(FramePacingPolicyResource::continuous_uncapped())
        );
        let mut empty = std::iter::empty();
        assert_eq!(
            NativeHostLaunchOverrides::parse_option(
                OsStr::new("--host-pacing=capped:144"),
                &mut empty
            )
            .unwrap()
            .unwrap()
            .frame_pacing(),
            Some(FramePacingPolicyResource::continuous_capped(144))
        );
        for invalid in [
            "",
            "bogus",
            "capped:",
            "capped:0",
            "capped:241",
            "capped:-1",
            "capped:abc",
        ] {
            assert!(invalid.parse::<NativeHostLaunchOverrides>().is_err());
        }
        assert!(
            NativeHostLaunchOverrides::parse_option(OsStr::new("--host-pacing"), &mut empty)
                .is_err()
        );
    }

    #[test]
    fn product_value_is_not_preempted_by_host_flag() {
        // A product grammar consumes its own option value before considering
        // the next token as a possible Host option.
        let mut args = [
            OsString::from("--host-pacing"),
            OsString::from("--host-pacing=uncapped"),
        ]
        .into_iter();
        let product_value = args.next().unwrap();
        assert_eq!(product_value, OsStr::new("--host-pacing"));
        let host_option = args.next().unwrap();
        let host = NativeHostLaunchOverrides::parse_option(&host_option, &mut args)
            .unwrap()
            .unwrap();
        assert_eq!(
            host.frame_pacing(),
            Some(FramePacingPolicyResource::continuous_uncapped())
        );
    }

    #[test]
    fn headless_and_sealed_apps_reject_without_mutation() {
        let request: NativeHostLaunchOverrides = "uncapped".parse().unwrap();
        let mut headless = App::headless();
        assert!(request.apply_to(&mut headless).is_err());
        assert!(
            headless
                .world()
                .resource::<FramePacingPolicyResource>()
                .is_err()
        );
        NativeHostLaunchOverrides::default()
            .apply_to(&mut headless)
            .unwrap();

        let mut sealed = App::new();
        sealed.prepare_lifecycle_for_execution().unwrap();
        assert!(request.apply_to(&mut sealed).is_err());
        assert!(
            sealed
                .world()
                .resource::<FramePacingPolicyResource>()
                .is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_mode_is_rejected_without_consuming_foreign_arguments() {
        use std::os::unix::ffi::OsStringExt;
        let raw = OsString::from_vec(vec![0xff]);
        let mut empty = std::iter::empty();
        assert_eq!(
            NativeHostLaunchOverrides::parse_option(&raw, &mut empty).unwrap(),
            None
        );
        assert!(
            NativeHostLaunchOverrides::parse_option(
                OsStr::new("--host-pacing"),
                &mut [raw].into_iter()
            )
            .is_err()
        );
    }
}
