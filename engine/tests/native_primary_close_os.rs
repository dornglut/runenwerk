//! Physical X11 proof that the selected native Host close policy honors a
//! real WM_DELETE_WINDOW request without defeating product Update veto.
#![cfg(target_os = "linux")]

use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use engine::prelude::{
    App, AppNativeHostExt, FramePacingPolicyResource, NativePrimaryClosePolicy, ResMut,
    Update, WindowStateRegistryResource,
};

const CHILD_MODE: &str = "RUNENWERK_CLOSE_PROOF_CHILD_MODE";
const VETO_MARKER: &str = "RUNENWERK_CLOSE_PROOF_VETO_MARKER";

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn veto_primary_intent(mut windows: ResMut<WindowStateRegistryResource>) {
    if let Some(id) = windows.primary_window_id()
        && let Some(record) = windows.record_mut(id)
        && record.close_intent_pending
    {
        if let Some(marker) = env::var_os(VETO_MARKER) {
            std::fs::write(marker, b"veto observed").expect("record a genuine native veto");
        }
        record.veto_close();
    }
}

fn run_child(mode: &str) {
    let mut app = App::new();
    let title = format!("Runenwerk OS Close Proof {mode}");
    app.set_title(title);
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());
    app.with_primary_close_policy(NativePrimaryClosePolicy::ApproveUnhandled)
        .expect("primary-close policy selected");
    if mode == "veto" {
        app.add_systems(Update, veto_primary_intent);
    }
    app.run().expect("native window exits after approved close");
}

fn visible_window_id(title: &str, until: Instant) -> String {
    while Instant::now() < until {
        let output = Command::new("xdotool")
            .args(["search", "--onlyvisible", "--name", title])
            .output()
            .expect("xdotool must be installed in native proof lane");
        if output.status.success()
            && let Some(id) = String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|id| !id.is_empty())
        {
            return id.to_owned();
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("native window did not become visible: {title}");
}

fn wait_for_success(child: &mut Child, until: Instant) {
    while Instant::now() < until {
        if let Some(exit) = child.try_wait().expect("poll native App child") {
            assert!(exit.success(), "native App failed after OS close: {exit}");
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("native App did not exit after physical X11 CloseRequested");
}

fn wait_for_veto(marker: &PathBuf, child: &mut Child, until: Instant) {
    while Instant::now() < until {
        assert!(
            child.try_wait().expect("poll veto App child").is_none(),
            "native App exited despite its product Update veto"
        );
        if marker.exists() {
            assert_eq!(
                std::fs::read(marker).expect("read veto observation"),
                b"veto observed"
            );
            thread::sleep(Duration::from_millis(200));
            assert!(
                child.try_wait().expect("poll after veto").is_none(),
                "native App exited after veto"
            );
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("native CloseRequested did not reach product Update veto");
}

#[test]
#[ignore = "requires Linux X11/Xvfb and xdotool; enforced in native CI"]
fn physical_native_primary_close_approval_and_veto() {
    if let Ok(mode) = env::var(CHILD_MODE) {
        run_child(&mode);
        return;
    }

    for mode in ["approve", "veto"] {
        let title = format!("Runenwerk OS Close Proof {mode}");
        let marker = env::temp_dir().join(format!(
            "runenwerk-native-close-veto-{}-{mode}.marker",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&marker);

        let child = Command::new(env::current_exe().expect("find exact test binary"))
            .args([
                "--ignored",
                "--exact",
                "physical_native_primary_close_approval_and_veto",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_MODE, mode)
            .env(VETO_MARKER, &marker)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("launch native App child in same real X11 display");
        let mut child = ChildGuard(child);
        let window = visible_window_id(&title, Instant::now() + Duration::from_secs(30));
        let close = Command::new("xdotool")
            .args(["windowclose", &window])
            .status()
            .expect("request real X11 WM_DELETE_WINDOW");
        assert!(close.success(), "X11 close request was not delivered");

        if mode == "approve" {
            wait_for_success(&mut child.0, Instant::now() + Duration::from_secs(20));
        } else {
            wait_for_veto(
                &marker,
                &mut child.0,
                Instant::now() + Duration::from_secs(10),
            );
        }
        let _ = std::fs::remove_file(marker);
    }
    println!("native_primary_close_os=pass approve=physical_x11 veto=physical_x11");
}
