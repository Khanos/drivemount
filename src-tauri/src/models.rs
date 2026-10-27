use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CacheSettings {
    pub mode: String,
    pub size_gib: u32,
    pub age_hours: u32,
}
impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            mode: "full".into(),
            size_gib: 10,
            age_hours: 24,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub autostart: bool,
    pub read_only: bool,
    pub cache: CacheSettings,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            autostart: true,
            read_only: false,
            cache: CacheSettings::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub remote: String,
    pub config_path: PathBuf,
    pub mount_path: PathBuf,
    pub service: String,
    pub cache_path: PathBuf,
    pub socket_path: PathBuf,
    pub managed: bool,
    pub owned_remote: bool,
    pub authorized: bool,
    pub configured: bool,
    pub preferences: Preferences,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    pub version: u32,
    pub connections: Vec<Connection>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: String,
    pub label: String,
    pub state: String,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Remote {
    pub name: String,
    pub backend: String,
    pub config_path: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingMount {
    pub target: PathBuf,
    pub source: String,
    pub service: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingService {
    pub name: String,
    pub remote: String,
    pub mount_path: PathBuf,
    pub config_path: Option<PathBuf>,
    pub autostart: bool,
    pub managed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub distro: String,
    pub package_manager: Option<String>,
    pub packages: Vec<String>,
    pub checks: Vec<Check>,
    pub remotes: Vec<Remote>,
    pub mounts: Vec<ExistingMount>,
    pub services: Vec<ExistingService>,
    pub connections: Vec<Connection>,
    pub default_mount: PathBuf,
    pub app_oauth_available: bool,
    pub free_cache_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathValidation {
    pub path: PathBuf,
    pub valid: bool,
    pub reason: String,
    pub exists: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub id: String,
    pub state: String,
    pub mounted: bool,
    pub active_state: String,
    pub autostart: bool,
    pub pending_uploads: Option<u64>,
    pub open_files: Option<u64>,
    pub cache_bytes: Option<u64>,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub id: String,
    pub state: String,
    pub message: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    pub client_id: String,
    pub client_secret: String,
    pub connection_id: Option<String>,
}
impl Drop for AuthRequest {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.client_id.zeroize();
        self.client_secret.zeroize();
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupRequest {
    pub id: String,
    pub mount_path: String,
    pub preferences: Preferences,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostics {
    pub report: String,
    pub details: String,
}
