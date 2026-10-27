use crate::{
    engine::Engine,
    errors::{AppError, Result},
    models::*,
};
use std::sync::Arc;
use tauri::State;
#[tauri::command]
pub async fn inspect_system(engine: State<'_, Arc<Engine>>) -> Result<Inspection> {
    engine.inspect().await
}
#[tauri::command]
pub async fn install_dependencies(engine: State<'_, Arc<Engine>>, consent: bool) -> Result<()> {
    engine.install_dependencies(consent).await
}
#[tauri::command]
pub async fn begin_auth(
    engine: State<'_, Arc<Engine>>,
    request: AuthRequest,
) -> Result<AuthStatus> {
    engine.begin_auth(request).await
}
#[tauri::command]
pub async fn auth_status(engine: State<'_, Arc<Engine>>) -> Result<Option<AuthStatus>> {
    Ok(engine.oauth.status().await)
}
#[tauri::command]
pub async fn cancel_auth(engine: State<'_, Arc<Engine>>) -> Result<()> {
    engine.oauth.cancel().await;
    Ok(())
}
#[tauri::command]
pub async fn validate_mount_path(
    engine: State<'_, Arc<Engine>>,
    path: String,
) -> Result<PathValidation> {
    engine.validate_path(&path)
}
#[tauri::command]
pub async fn import_remote(
    engine: State<'_, Arc<Engine>>,
    name: String,
    consent: bool,
) -> Result<Connection> {
    engine.import_remote(&name, consent).await
}
#[tauri::command]
pub async fn connect_drive(
    engine: State<'_, Arc<Engine>>,
    request: SetupRequest,
) -> Result<ConnectionStatus> {
    engine.setup(request).await
}
#[tauri::command]
pub async fn setup_progress(engine: State<'_, Arc<Engine>>) -> Result<Progress> {
    Ok(engine.progress.lock().await.clone())
}
#[tauri::command]
pub async fn connection_status(
    engine: State<'_, Arc<Engine>>,
    id: String,
) -> Result<ConnectionStatus> {
    engine.status(&id).await
}
#[tauri::command]
pub async fn connection_action(
    engine: State<'_, Arc<Engine>>,
    id: String,
    action: String,
    acknowledge_unknown: bool,
) -> Result<ConnectionStatus> {
    engine.lifecycle(&id, &action, acknowledge_unknown).await
}
#[tauri::command]
pub async fn set_autostart(
    engine: State<'_, Arc<Engine>>,
    id: String,
    enabled: bool,
) -> Result<()> {
    engine.autostart(&id, enabled).await
}
#[tauri::command]
pub async fn read_diagnostics(engine: State<'_, Arc<Engine>>, id: String) -> Result<Diagnostics> {
    engine.diagnostics(&id).await
}
#[tauri::command]
pub async fn open_drive(engine: State<'_, Arc<Engine>>, id: String) -> Result<()> {
    engine.open_drive(&id).await
}
#[tauri::command]
pub async fn choose_folder(app: tauri::AppHandle) -> Result<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Choose where Google Drive appears")
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok())
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .map_err(|_| {
        AppError::new(
            "folder_picker",
            "The folder picker could not be opened. You can enter the folder path instead.",
        )
    })
}
#[tauri::command]
pub async fn open_help(engine: State<'_, Arc<Engine>>, page: String) -> Result<()> {
    let url = match page.as_str() {
        "cloud" => "https://console.cloud.google.com/projectcreate",
        "api" => "https://console.cloud.google.com/apis/library/drive.googleapis.com",
        "consent" => "https://console.cloud.google.com/auth/overview",
        "credentials" => "https://console.cloud.google.com/auth/clients",
        "guide" => "https://rclone.org/drive/#making-your-own-client-id",
        _ => return Err(AppError::new("help_link", "This help link is unavailable.")),
    };
    engine
        .runner
        .run(crate::system::process::CommandSpec::new(
            crate::system::process::binary("xdg-open")?,
            [url],
        ))
        .await?
        .checked()?;
    Ok(())
}
