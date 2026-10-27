#[cfg(feature = "desktop")]
mod commands;
pub mod engine;
pub mod errors;
pub mod models;
pub mod rclone;
pub mod security;
pub mod system;
pub mod systemd;
#[cfg(feature = "desktop")]
pub fn run() {
    use std::sync::Arc;
    use tauri::Manager;
    if std::env::args().any(|a| a == "--version") {
        println!("DriveMount {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("Run DriveMount as your normal desktop user, not as root.");
        std::process::exit(1);
    }
    let engine = Arc::new(engine::Engine::new(
        match system::paths::Paths::discover() {
            Ok(paths) => paths,
            Err(error) => {
                eprintln!("DriveMount: {}", error.message);
                std::process::exit(1);
            }
        },
        Arc::new(system::process::SystemRunner),
    ));
    if std::env::args().any(|a| a == "--inspect") {
        let runtime = tokio::runtime::Runtime::new().expect("Runtime unavailable");
        match runtime.block_on(engine.inspect()) {
            Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
            Err(e) => {
                eprintln!("{}", e.message);
                std::process::exit(1);
            }
        }
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::inspect_system,
            commands::install_dependencies,
            commands::begin_auth,
            commands::auth_status,
            commands::cancel_auth,
            commands::validate_mount_path,
            commands::import_remote,
            commands::connect_drive,
            commands::setup_progress,
            commands::connection_status,
            commands::connection_action,
            commands::set_autostart,
            commands::read_diagnostics,
            commands::open_drive,
            commands::choose_folder,
            commands::open_help
        ])
        .run(tauri::generate_context!())
        .expect("DriveMount could not start");
}
#[cfg(test)]
pub mod tests {
    use crate::models::*;
    use std::path::Path;
    pub fn connection(root: &Path) -> Connection {
        Connection {
            id: "11111111-1111-4111-8111-111111111111".into(),
            remote: "gdrive".into(),
            config_path: root.join("rclone.conf"),
            mount_path: root.join("Drive"),
            service: "drivemount-test.service".into(),
            cache_path: root.join("cache"),
            socket_path: root.join("drive.sock"),
            managed: true,
            owned_remote: true,
            authorized: true,
            configured: false,
            preferences: Preferences::default(),
        }
    }
}
