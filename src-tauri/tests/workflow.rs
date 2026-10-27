//! All service operations are simulated. Real host configurations and services are never modified.
use async_trait::async_trait;
use drivemount_lib::{
    engine::Engine,
    errors::{AppError, Result},
    models::*,
    rclone::mount::Mount,
    system::{
        host::Host,
        paths::Paths,
        process::{CommandSpec, Output, Runner},
    },
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::{Arc, Mutex},
};
#[derive(Default)]
struct FixtureHost {
    mounts: Mutex<Vec<Mount>>,
    online: bool,
}
#[async_trait]
impl Host for FixtureHost {
    fn mounts(&self) -> Result<Vec<Mount>> {
        Ok(self.mounts.lock().unwrap().clone())
    }
    async fn online(&self) -> bool {
        self.online
    }
}
struct FixtureRunner {
    host: Arc<FixtureHost>,
    connection: Connection,
    fail_mount: bool,
    auth_error: bool,
    calls: Mutex<Vec<CommandSpec>>,
}
#[async_trait]
impl Runner for FixtureRunner {
    async fn run(&self, s: CommandSpec) -> Result<Output> {
        let mut out = String::new();
        if s.args.contains(&"about".into()) && self.auth_error {
            return Err(AppError::new(
                "authentication",
                "Google connection needs renewal.",
            ));
        }
        if s.args.contains(&"show".into()) {
            out = format!(
                "ActiveState={}\nUnitFileState=enabled\n",
                if self.host.mounts.lock().unwrap().is_empty() {
                    "inactive"
                } else {
                    "active"
                }
            );
        }
        if s.args.contains(&"start".into()) {
            if self.fail_mount {
                return Err(AppError::new("mount_failed", "Fixture mount failure"));
            }
            self.host.mounts.lock().unwrap().push(Mount {
                target: self.connection.mount_path.clone(),
                source: format!("{}:", self.connection.remote),
                kind: "fuse.rclone".into(),
            });
        }
        if s.args.contains(&"stop".into()) {
            self.host.mounts.lock().unwrap().clear();
        }
        self.calls.lock().unwrap().push(s);
        Ok(Output {
            success: true,
            stdout: out,
            stderr: String::new(),
        })
    }
}
fn fixture(
    root: &Path,
    fail: bool,
    auth_error: bool,
    online: bool,
) -> (Engine, Arc<FixtureRunner>, Connection) {
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
    let p = Paths {
        home: root.into(),
        config: root.join("config"),
        cache: root.join("cache"),
        units: root.join("units"),
        runtime: root.join("run"),
    };
    let c = Connection {
        id: "fixture".into(),
        remote: "fixture_drive".into(),
        config_path: root.join("rclone.conf"),
        mount_path: root.join("My Drive 文 % $"),
        service: "drivemount-fixture.service".into(),
        cache_path: p.cache.join("fixture"),
        socket_path: p.runtime.join("fixture.sock"),
        managed: true,
        owned_remote: true,
        authorized: true,
        configured: false,
        preferences: Preferences::default(),
    };
    let host = Arc::new(FixtureHost {
        online,
        ..Default::default()
    });
    let runner = Arc::new(FixtureRunner {
        host: host.clone(),
        connection: c.clone(),
        fail_mount: fail,
        auth_error,
        calls: Mutex::new(vec![]),
    });
    p.save(&Registry {
        version: 1,
        connections: vec![c.clone()],
    })
    .unwrap();
    let mut engine = Engine::new(p, runner.clone());
    engine.host = host;
    (engine, runner, c)
}
fn request(c: &Connection) -> SetupRequest {
    SetupRequest {
        id: c.id.clone(),
        mount_path: c.mount_path.to_string_lossy().into(),
        preferences: c.preferences.clone(),
    }
}
#[tokio::test]
async fn fresh_setup_verifies_mount_and_survives_app_restart() {
    let t = tempfile::tempdir().unwrap();
    let (e, r, c) = fixture(t.path(), false, false, true);
    let s = e.setup(request(&c)).await.unwrap();
    assert!(s.mounted);
    assert_eq!(s.state, "connected");
    assert!(e.paths.load().unwrap().connections[0].configured);
    let unit = fs::read_to_string(e.paths.units.join(&c.service)).unwrap();
    assert!(unit.contains("My Drive 文 %% $$"));
    let reopened = Engine::new(e.paths.clone(), r.clone());
    assert!(reopened.connection(&c.id).unwrap().configured);
    assert!(r
        .calls
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.args.contains(&"enable".into())));
}
#[tokio::test]
async fn failed_start_rolls_back_owned_unit_and_keeps_draft() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, c) = fixture(t.path(), true, false, true);
    assert_eq!(e.setup(request(&c)).await.unwrap_err().code, "mount_failed");
    assert!(!e.paths.units.join(&c.service).exists());
    assert!(!e.connection(&c.id).unwrap().configured);
    assert!(c.cache_path.exists());
}
#[tokio::test]
async fn auth_failure_never_writes_service() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, c) = fixture(t.path(), false, true, true);
    assert_eq!(
        e.setup(request(&c)).await.unwrap_err().code,
        "authentication"
    );
    assert!(!e.paths.units.exists());
}
#[tokio::test]
async fn unknown_upload_state_requires_explicit_confirmation() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, c) = fixture(t.path(), false, false, true);
    e.setup(request(&c)).await.unwrap();
    assert_eq!(
        e.lifecycle(&c.id, "stop", false).await.unwrap_err().code,
        "pending_unknown"
    );
    assert!(!e.lifecycle(&c.id, "stop", true).await.unwrap().mounted);
    assert!(e.lifecycle(&c.id, "start", false).await.unwrap().mounted);
}
#[tokio::test]
async fn offline_mount_is_not_misreported_as_disconnected() {
    let t = tempfile::tempdir().unwrap();
    let (e, r, c) = fixture(t.path(), false, false, false);
    r.host.mounts.lock().unwrap().push(Mount {
        target: c.mount_path.clone(),
        source: format!("{}:", c.remote),
        kind: "fuse.rclone".into(),
    });
    let s = e.status(&c.id).await.unwrap();
    assert_eq!(s.state, "offline");
    assert!(s.mounted);
    assert!(!r
        .calls
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.args.contains(&"about".into())));
}
#[tokio::test]
async fn competing_setup_operations_are_rejected() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, c) = fixture(t.path(), false, false, true);
    let _guard = e.operation.lock().await;
    assert_eq!(
        e.setup(request(&c)).await.unwrap_err().code,
        "operation_busy"
    );
}
#[tokio::test]
async fn imported_manual_service_is_never_rewritten() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, mut c) = fixture(t.path(), false, false, true);
    c.managed = false;
    c.owned_remote = false;
    e.paths
        .save(&Registry {
            version: 1,
            connections: vec![c.clone()],
        })
        .unwrap();
    fs::create_dir_all(&e.paths.units).unwrap();
    let unit = e.paths.units.join(&c.service);
    fs::write(&unit, "user-owned unit").unwrap();
    assert_eq!(
        e.setup(request(&c)).await.unwrap_err().code,
        "external_service"
    );
    assert_eq!(fs::read_to_string(unit).unwrap(), "user-owned unit");
}
#[tokio::test]
async fn populated_folder_and_existing_mount_never_get_hidden() {
    let t = tempfile::tempdir().unwrap();
    let (e, r, c) = fixture(t.path(), false, false, true);
    fs::create_dir(&c.mount_path).unwrap();
    fs::write(c.mount_path.join("precious.txt"), "keep").unwrap();
    assert_eq!(e.setup(request(&c)).await.unwrap_err().code, "mount_path");
    assert_eq!(
        fs::read_to_string(c.mount_path.join("precious.txt")).unwrap(),
        "keep"
    );
    r.host.mounts.lock().unwrap().push(Mount {
        target: c.mount_path.clone(),
        source: "other:".into(),
        kind: "fuse.rclone".into(),
    });
    assert!(
        !e.validate_path(c.mount_path.to_str().unwrap())
            .unwrap()
            .valid
    );
}
#[tokio::test]
async fn partial_owned_service_is_recoverable_but_never_mounted_twice() {
    let t = tempfile::tempdir().unwrap();
    let (e, r, c) = fixture(t.path(), false, false, true);
    e.setup(request(&c)).await.unwrap();
    let mut record = e.paths.load().unwrap();
    record.connections[0].configured = false;
    e.paths.save(&record).unwrap();
    let count = r
        .calls
        .lock()
        .unwrap()
        .iter()
        .filter(|s| s.args.contains(&"start".into()))
        .count();
    assert_eq!(
        e.setup(request(&c)).await.unwrap_err().code,
        "stop_required"
    );
    assert_eq!(
        r.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.args.contains(&"start".into()))
            .count(),
        count
    );
    assert!(!e.lifecycle(&c.id, "stop", true).await.unwrap().mounted);
}
#[tokio::test]
async fn replaced_owned_unit_cannot_be_controlled() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, c) = fixture(t.path(), false, false, true);
    fs::create_dir_all(&e.paths.units).unwrap();
    fs::write(
        e.paths.units.join(&c.service),
        "[Service]\nExecStart=/usr/bin/true\n",
    )
    .unwrap();
    assert_eq!(
        e.lifecycle(&c.id, "start", false).await.unwrap_err().code,
        "unowned_service"
    );
}
#[tokio::test]
async fn disconnected_offline_service_does_not_promise_automatic_reconnection() {
    let t = tempfile::tempdir().unwrap();
    let (e, _, c) = fixture(t.path(), false, false, false);
    let status = e.status(&c.id).await.unwrap();
    assert_eq!(status.state, "offline");
    assert!(!status.mounted);
    assert!(status.detail.contains("Reconnect Drive"));
}
