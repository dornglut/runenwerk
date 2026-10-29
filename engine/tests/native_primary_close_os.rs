//! Physical X11 proof that the selected native Host close policy honors a
//! real WM_DELETE_WINDOW request without defeating product Update veto.
#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::{Child, Command, Stdio};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use engine::prelude::{
    App, AppNativeHostExt, FramePacingPolicyResource, NativePrimaryClosePolicy, ResMut,
    WindowStateRegistryResource, default_plugins,
};
#[cfg(target_os = "linux")]
use engine::runtime::schedules::Update;
#[cfg(target_os = "linux")]
use x11rb::connection::Connection as _;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask};

#[cfg(target_os = "linux")]
const CHILD_MODE: &str = "RUNENWERK_CLOSE_PROOF_CHILD_MODE";
#[cfg(target_os = "linux")]
const VETO_MARKER: &str = "RUNENWERK_CLOSE_PROOF_VETO_MARKER";

#[cfg(target_os = "linux")]
struct ChildGuard(Child);

#[cfg(target_os = "linux")]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(target_os = "linux")]
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


#[cfg(target_os = "linux")]
fn run_child(mode: &str) {
    let mut app = App::new();
    app.add_plugins(default_plugins());
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

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn request_wm_delete_window(window_id: &str) {
    let window: u32 = window_id.parse().expect("valid X11 window ID");
    let (conn, _) = x11rb::connect(None).expect("open the X11 display");
    let protocols = conn.intern_atom(false, b"WM_PROTOCOLS").unwrap().reply().unwrap().atom;
    let delete = conn.intern_atom(false, b"WM_DELETE_WINDOW").unwrap().reply().unwrap().atom;
    let supported = conn.get_property(false, window, protocols, AtomEnum::ATOM, 0, 32)
        .unwrap().reply().unwrap();
    assert!(
        supported.value32().is_some_and(|mut atoms| atoms.any(|a| a == delete)),
        "native X11 window must advertise WM_DELETE_WINDOW"
    );
    let message = ClientMessageEvent::new(32, window, protocols, [delete, 0, 0, 0, 0]);
    conn.send_event(false, window, EventMask::NO_EVENT, message).unwrap().check().unwrap();
    conn.flush().unwrap();
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn wait_for_veto(marker: &PathBuf, child: &mut Child, until: Instant) {
    while Instant::now() < until {
        assert!(
            child.try_wait().expect("poll veto App child").is_none(),
            "native App exited despite its product Update veto"
        );
        if marker.exists() {
            assert_eq!(
                std::fs::read(marker)
                    .expect("read veto observation")
                    .as_slice(),
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

#[cfg(not(target_os = "linux"))]
fn main() {}

#[cfg(target_os = "linux")]
fn main() {
    if let Ok(mode) = env::var(CHILD_MODE) {
        run_child(&mode);
        return;
    }
    if env::var("RUNENWERK_NATIVE_CLOSE_OS_SMOKE").as_deref() != Ok("1") {
        println!("native_primary_close_os=skipped explicit Linux X11 opt-in absent");
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
            .env(CHILD_MODE, mode)
            .env(VETO_MARKER, &marker)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("launch native App child in same real X11 display");
        let mut child = ChildGuard(child);
        let window = visible_window_id(&title, Instant::now() + Duration::from_secs(30));
        request_wm_delete_window(&window);

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
