//! Explicit opt-in integration with an installed rclone. All writes are inside a fresh temp directory.
use drivemount_lib::{models::*, rclone::oauth, systemd::service};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Stdio, time::Duration};
#[tokio::test]
#[ignore = "opt-in: starts a private rclone helper with a temporary config; no cloud credentials or real mounts"]
async fn documented_config_api_works_over_private_unix_socket() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let config = root.path().join("rclone.conf");
    let socket = root.path().join("control.sock");
    drivemount_lib::system::paths::create_private(&config).unwrap();
    let mut command = tokio::process::Command::new("rclone");
    unsafe {
        command.pre_exec(|| {
            libc::umask(0o077);
            Ok(())
        });
    }
    let mut child = command
        .arg("rcd")
        .arg("--config")
        .arg(&config)
        .args([
            "--rc-addr",
            &format!("unix://{}", socket.display()),
            "--rc-no-auth",
            "--log-level",
            "ERROR",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    for _ in 0..50 {
        if socket.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let client = oauth::client(&socket).await.unwrap();
    let created=oauth::rc(&client,"config/create",json!({"name":"fixture_local","type":"local","parameters":{},"opt":{"nonInteractive":true}})).await.unwrap();
    assert!(oauth::answer(&created).unwrap().is_none());
    let list = oauth::rc(&client, "config/listremotes", json!({}))
        .await
        .unwrap();
    assert!(list["remotes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "fixture_local"));
    let updated = oauth::rc(
        &client,
        "config/update",
        json!({"name":"fixture_local","parameters":{"nounc":"true"},"opt":{"nonInteractive":true}}),
    )
    .await
    .unwrap();
    assert!(oauth::answer(&updated).unwrap().is_none());
    assert!(fs::read_to_string(&config)
        .unwrap()
        .contains("[fixture_local]"));
    child.kill().await.unwrap();
    child.wait().await.unwrap();
}
#[test]
#[ignore = "opt-in: requires systemd-analyze, rclone and fusermount3; verifies a temporary unit without loading it"]
fn systemd_accepts_generated_units_with_special_paths() {
    let root = tempfile::tempdir().unwrap();
    let c = Connection {
        id: "fixture".into(),
        remote: "gdrive".into(),
        config_path: root.path().join("config % $ 文/rclone.conf"),
        mount_path: root.path().join("My Drive % $ 文"),
        service: "drivemount-fixture.service".into(),
        cache_path: root.path().join("cache % $ 文"),
        socket_path: root.path().join("rpc.sock"),
        managed: true,
        owned_remote: true,
        authorized: true,
        configured: false,
        preferences: Preferences::default(),
    };
    let unit = service::generate(
        &c,
        Path::new("/usr/bin/rclone"),
        Path::new("/usr/bin/fusermount3"),
    )
    .unwrap();
    let path = root.path().join(&c.service);
    fs::write(&path, unit).unwrap();
    let output = std::process::Command::new("systemd-analyze")
        .args(["--user", "verify"])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
