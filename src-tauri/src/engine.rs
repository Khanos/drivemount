use crate::{
    errors::{AppError, Result},
    models::*,
    rclone::{
        detection, mount,
        oauth::{self, OAuthManager},
    },
    security::{
        redaction::{local_details, shareable_report},
        validation,
    },
    system::{
        dependencies, packages,
        paths::{self, Paths},
        platform,
        process::{binary, CommandSpec, Runner},
    },
    systemd::{service, status},
};
use serde_json::json;
use std::{fs, sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub struct Engine {
    pub paths: Paths,
    pub host: Arc<dyn crate::system::host::Host>,
    pub runner: Arc<dyn Runner>,
    pub oauth: OAuthManager,
    pub operation: Mutex<()>,
    pub progress: Mutex<Progress>,
}
impl Engine {
    pub fn new(paths: Paths, runner: Arc<dyn Runner>) -> Self {
        Self {
            paths,
            host: Arc::new(crate::system::host::RealHost),
            runner,
            oauth: OAuthManager::default(),
            operation: Mutex::new(()),
            progress: Mutex::new(Progress {
                stage: "idle".into(),
                message: String::new(),
            }),
        }
    }
    async fn stage(&self, stage: &str, message: &str) {
        *self.progress.lock().await = Progress {
            stage: stage.into(),
            message: message.into(),
        };
    }
    pub async fn inspect(&self) -> Result<Inspection> {
        dependencies::inspect(self.runner.as_ref(), &self.paths).await
    }
    pub fn connection(&self, id: &str) -> Result<Connection> {
        self.paths
            .load()?
            .connections
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| {
                AppError::new(
                    "connection_missing",
                    "This connection could not be found. Refresh the dashboard.",
                )
            })
    }
    fn save_connection(&self, c: Connection) -> Result<()> {
        let mut reg = self.paths.load()?;
        if let Some(old) = reg.connections.iter_mut().find(|old| old.id == c.id) {
            *old = c
        } else {
            reg.connections.push(c)
        }
        self.paths.save(&reg)
    }
    fn draft(&self) -> Connection {
        let id = uuid::Uuid::new_v4().to_string();
        Connection {
            id: id.clone(),
            remote: format!("drivemount_{}", id.replace('-', "_")),
            config_path: self
                .paths
                .config
                .join("connections")
                .join(&id)
                .join("rclone.conf"),
            mount_path: self.paths.home.join("GoogleDrive"),
            service: format!("drivemount-{id}.service"),
            cache_path: self.paths.cache.join(&id),
            socket_path: self.paths.runtime.join(format!("{id}.sock")),
            managed: true,
            owned_remote: true,
            authorized: false,
            configured: false,
            preferences: Preferences::default(),
        }
    }
    pub async fn install_dependencies(&self, consent: bool) -> Result<()> {
        let _op = self
            .operation
            .try_lock()
            .map_err(|_| AppError::new("operation_busy", "An operation is already running."))?;
        let _file = self.paths.lock()?;
        let p = platform::detect(&fs::read_to_string("/etc/os-release")?);
        let manager = p.manager.ok_or_else(|| {
            AppError::new(
                "unsupported",
                "Automatic installation is not supported on this distribution.",
            )
        })?;
        let mut missing = vec![];
        if binary("rclone").is_err() {
            missing.push("rclone".into())
        }
        if binary("fusermount3").is_err() {
            missing.push("fuse3".into())
        }
        if missing.is_empty() {
            return Ok(());
        }
        packages::install(self.runner.as_ref(), &manager, &missing, consent).await
    }
    pub async fn begin_auth(&self, request: AuthRequest) -> Result<AuthStatus> {
        let _op = self.operation.try_lock().map_err(|_| {
            AppError::new("operation_busy", "Another connection operation is running.")
        })?;
        let lock = self.paths.lock()?;
        let c = if let Some(id) = &request.connection_id {
            let c = self.connection(id)?;
            if !c.owned_remote {
                return Err(AppError::new("external_account","Renew this existing account in the tool that configured it. Its credentials are preserved."));
            }
            if c.configured {
                self.ensure_stopped(&c).await?;
            }
            c
        } else {
            let c = self.draft();
            paths::private_dir(c.config_path.parent().unwrap())?;
            paths::create_private(&c.config_path)?;
            self.save_connection(c.clone())?;
            c
        };
        self.oauth
            .start(self.paths.clone(), self.runner.clone(), c, request, lock)
            .await
    }
    pub fn validate_path(&self, value: &str) -> Result<PathValidation> {
        mount::validate(value, &self.paths, &self.host.mounts()?)
    }
    pub async fn import_remote(&self, name: &str, consent: bool) -> Result<Connection> {
        if !consent {
            return Err(AppError::new(
                "consent",
                "Approve managing this existing connection first.",
            ));
        }
        validation::remote_name(name)?;
        let _op = self.operation.try_lock().map_err(|_| {
            AppError::new("operation_busy", "Another connection operation is running.")
        })?;
        let _lock = self.paths.lock()?;
        let inspection = self.inspect().await?;
        let remote = inspection
            .remotes
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| {
                AppError::new(
                    "remote_missing",
                    "This existing account could not be found.",
                )
            })?;
        if let Some(c) = inspection
            .connections
            .iter()
            .find(|c| c.remote == name && c.config_path == remote.config_path)
        {
            return Ok(c.clone());
        }
        let mut c = self.draft();
        c.remote = name.into();
        c.config_path = remote.config_path.clone();
        c.owned_remote = false;
        c.authorized = true;
        if let Some(m) = inspection
            .mounts
            .iter()
            .find(|m| m.source == format!("{name}:"))
        {
            let unit=m.service.as_ref().ok_or_else(||AppError::new("external_mount","Drive is mounted by another process. Open its folder or disconnect it in its original application before creating a managed connection."))?;
            c.mount_path = m.target.clone();
            c.service = unit.clone();
            c.managed = false;
            c.configured = true;
            let s = status::show(self.runner.as_ref(), unit).await?;
            c.preferences.autostart = s.get("UnitFileState").is_some_and(|v| v == "enabled");
        }
        if !c.configured {
            if inspection
                .checks
                .iter()
                .any(|check| check.id == "existing_services" && check.state == "attention")
            {
                return Err(AppError::new("existing_service", "An existing custom rclone service needs review in its original tool before DriveMount can safely import this account."));
            }
            let services: Vec<_> = inspection
                .services
                .iter()
                .filter(|s| {
                    s.remote == name
                        && s.config_path
                            .as_ref()
                            .is_none_or(|path| path == &c.config_path)
                })
                .collect();
            if services.len() > 1 {
                return Err(AppError::new("competing_services", "Multiple services use this account. Review them in their original configuration before importing."));
            }
            if let Some(service) = services.first() {
                c.mount_path = service.mount_path.clone();
                c.service = service.name.clone();
                c.managed = false;
                c.configured = true;
                c.preferences.autostart = service.autostart;
            }
        }
        self.save_connection(c.clone())?;
        Ok(c)
    }
    pub async fn setup(&self, request: SetupRequest) -> Result<ConnectionStatus> {
        let _op = self.operation.try_lock().map_err(|_| {
            AppError::new("operation_busy", "Another connection operation is running.")
        })?;
        let _lock = self.paths.lock()?;
        let mut c = self.connection(&request.id)?;
        let previous = c.clone();
        if !c.managed {
            return Err(AppError::new(
                "external_service",
                "This service is managed externally. Its settings will not be overwritten.",
            ));
        }
        if !c.authorized {
            return Err(AppError::new(
                "authentication",
                "Connect your Google account before mounting Drive.",
            ));
        }
        validation::preferences(&request.preferences)?;
        self.ensure_stopped(&c).await?;
        self.stage("validate", "Checking your folder and account…")
            .await;
        let v = self.validate_path(&request.mount_path)?;
        if !v.valid {
            return Err(AppError::new("mount_path", v.reason));
        }
        detection::verify_remote(self.runner.as_ref(), &c.config_path, &c.remote).await?;
        c.mount_path = v.path;
        c.preferences = request.preferences;
        // Persist recovery information before touching a service. Drafts remain visible after interruption.
        self.save_connection(c.clone())?;
        self.stage("directories", "Preparing private connection folders…")
            .await;
        fs::create_dir_all(&c.mount_path)?;
        paths::private_dir(&c.cache_path)?;
        self.paths.prepare_runtime()?;
        fs::create_dir_all(&self.paths.units)?;
        // Recheck after directory creation so an intervening mount or local file does not get hidden.
        let v = self.validate_path(c.mount_path.to_str().ok_or_else(|| {
            AppError::new("mount_path", "The folder name cannot be represented.")
        })?)?;
        if !v.valid {
            return Err(AppError::new("mount_path", v.reason));
        }
        let unit = service::generate(&c, &binary("rclone")?, &binary("fusermount3")?)?;
        let unit_path = self.paths.units.join(&c.service);
        let prior = fs::read(&unit_path).ok();
        self.stage("service", "Saving your automatic connection…")
            .await;
        service::write_owned(&unit_path, &c, &unit)?;
        let result=async {
            service::action(self.runner.as_ref(),"daemon-reload",None).await?;
            service::action(self.runner.as_ref(),if c.preferences.autostart{"enable"}else{"disable"},Some(&c.service)).await?;
            self.stage("mount","Connecting Google Drive…").await;service::action(self.runner.as_ref(),"start",Some(&c.service)).await?;
            self.stage("verify","Checking that your Drive folder is ready…").await;
            for _ in 0..30 {if mount::matches(&self.host.mounts()?,&c.mount_path,&c.remote){detection::verify_remote(self.runner.as_ref(),&c.config_path,&c.remote).await?;
                self.runner.run(CommandSpec::new(binary("stat")?,["--format=%F".to_string(),"--".to_string(),c.mount_path.to_string_lossy().to_string()]).seconds(5)).await?.checked()?;
                return Ok(())}tokio::time::sleep(Duration::from_millis(500)).await;}
            Err(AppError::new("mount_failed","Google Drive did not appear at the chosen folder. Refresh diagnostics and try again."))
        }.await;
        if let Err(error) = result {
            // A failed readiness check can still leave a working mount or queued writes. Preserve it.
            if !mount::matches(&self.host.mounts()?, &c.mount_path, &c.remote) {
                let _ = service::action(self.runner.as_ref(), "stop", Some(&c.service)).await;
                let _ = service::action(self.runner.as_ref(), "disable", Some(&c.service)).await;
                if let Some(bytes) = prior {
                    let _ = paths::atomic_write(&unit_path, &bytes, 0o600);
                    let _ = service::action(
                        self.runner.as_ref(),
                        if previous.preferences.autostart {
                            "enable"
                        } else {
                            "disable"
                        },
                        Some(&c.service),
                    )
                    .await;
                    let _ = self.save_connection(previous);
                } else {
                    let _ = fs::remove_file(&unit_path);
                }
                let _ = service::action(self.runner.as_ref(), "daemon-reload", None).await;
            }
            self.stage("error", &error.message).await;
            return Err(error);
        }
        c.configured = true;
        self.save_connection(c.clone())?;
        self.stage("complete", "Your Google Drive is ready!").await;
        self.status(&c.id).await
    }
    async fn ensure_stopped(&self, c: &Connection) -> Result<()> {
        let m = self.host.mounts()?;
        let s = status::show(self.runner.as_ref(), &c.service).await?;
        if m.iter().any(|m| m.target == c.mount_path)
            || s.get("ActiveState").is_some_and(|v| {
                ["active", "activating", "deactivating", "reloading"].contains(&v.as_str())
            })
        {
            return Err(AppError::new(
                "stop_required",
                "Disconnect Drive before changing its connection settings.",
            ));
        }
        Ok(())
    }
    pub async fn status(&self, id: &str) -> Result<ConnectionStatus> {
        let c = self.connection(id)?;
        let s = status::show(self.runner.as_ref(), &c.service).await?;
        let list = self.host.mounts()?;
        let mounted = mount::matches(&list, &c.mount_path, &c.remote);
        let active = s
            .get("ActiveState")
            .cloned()
            .unwrap_or_else(|| "unknown".into());
        let autostart = s.get("UnitFileState").is_some_and(|v| v == "enabled");
        let net = self.host.online().await;
        let mut state = if !net {
            "offline"
        } else if mounted {
            "connected"
        } else if s.get("SubState").is_some_and(|v| v == "auto-restart") {
            "reconnecting"
        } else if active == "activating" {
            "connecting"
        } else if active == "failed" {
            "error"
        } else {
            "disconnected"
        }
        .to_string();
        let mut detail = if !net && !mounted && active=="inactive" {
            "You're offline and Drive is disconnected. Reconnect Drive when your internet connection returns."
        } else if !net {
            "You're offline. Drive will try requests again when your connection returns."
        } else if mounted {
            "Your Drive folder is mounted."
        } else {
            "Your Drive folder is not mounted."
        }
        .to_string();
        if (mounted || active == "failed") && net {
            if let Err(e) =
                detection::verify_remote(self.runner.as_ref(), &c.config_path, &c.remote).await
            {
                state = if e.code == "authentication" {
                    "authenticationRequired"
                } else if e.code == "offline" {
                    "offline"
                } else {
                    "error"
                }
                .into();
                detail = e.message;
            }
        }
        let mut pending = None;
        let mut open = None;
        let mut bytes = None;
        if mounted && c.managed {
            if let Ok(http) = oauth::client(&c.socket_path).await {
                let query = oauth::rc(&http, "vfs/stats", json!({}));
                if let Ok(Ok(v)) = tokio::time::timeout(Duration::from_secs(3), query).await {
                    pending = cache_pending(&v);
                    open = v.get("inUse").and_then(|v| v.as_u64());
                    bytes = v.pointer("/diskCache/bytesUsed").and_then(|v| v.as_u64());
                }
            }
        }
        Ok(ConnectionStatus {
            id: id.into(),
            state,
            mounted,
            active_state: active,
            autostart,
            pending_uploads: pending,
            open_files: open,
            cache_bytes: bytes,
            detail,
        })
    }
    fn check_service_owner(&self, c: &Connection) -> Result<()> {
        if c.managed {
            let text = fs::read_to_string(self.paths.units.join(&c.service))?;
            if text.lines().next() != Some(service::marker(&c.id).as_str()) {
                return Err(AppError::new("unowned_service","This service was changed outside DriveMount. Review it in its original configuration before controlling it."));
            }
        }
        Ok(())
    }
    pub async fn lifecycle(
        &self,
        id: &str,
        action: &str,
        acknowledge_unknown: bool,
    ) -> Result<ConnectionStatus> {
        let _op = self.operation.try_lock().map_err(|_| {
            AppError::new("operation_busy", "Another connection operation is running.")
        })?;
        let _lock = self.paths.lock()?;
        let c = self.connection(id)?;
        self.check_service_owner(&c)?;
        if !["start", "stop", "restart"].contains(&action) {
            return Err(AppError::new(
                "service_action",
                "This connection action is not supported.",
            ));
        }
        if action == "start" && mount::matches(&self.host.mounts()?, &c.mount_path, &c.remote) {
            return self.status(id).await;
        }
        if action == "stop" || action == "restart" {
            let s = self.status(id).await?;
            if s.mounted
                && !["active", "activating", "reloading"].contains(&s.active_state.as_str())
            {
                return Err(AppError::new("external_mount","This folder is mounted by another process. Manage it using its original connection tool."));
            }
            guard_disconnect(&s, acknowledge_unknown)?;
        }
        if action == "start" {
            let v = self.validate_path(c.mount_path.to_str().unwrap_or(""))?;
            if !v.valid {
                return Err(AppError::new("mount_path", v.reason));
            }
        }
        if action == "restart" {
            service::action(self.runner.as_ref(), "stop", Some(&c.service)).await?;
            let v = self.validate_path(c.mount_path.to_str().unwrap_or(""))?;
            if !v.valid {
                return Err(AppError::new("mount_path", v.reason));
            }
            service::action(self.runner.as_ref(), "start", Some(&c.service)).await?;
        } else {
            service::action(self.runner.as_ref(), action, Some(&c.service)).await?;
        }
        for _ in 0..20 {
            let mounted = mount::matches(&self.host.mounts()?, &c.mount_path, &c.remote);
            if (action == "stop" && !mounted) || (action != "stop" && mounted) {
                return self.status(id).await;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(AppError::new(
            "mount_state",
            "The requested connection state was not reached. Refresh diagnostics.",
        ))
    }
    pub async fn autostart(&self, id: &str, enabled: bool) -> Result<()> {
        let _op = self.operation.try_lock().map_err(|_| {
            AppError::new("operation_busy", "Another connection operation is running.")
        })?;
        let _lock = self.paths.lock()?;
        let mut c = self.connection(id)?;
        self.check_service_owner(&c)?;
        service::action(
            self.runner.as_ref(),
            if enabled { "enable" } else { "disable" },
            Some(&c.service),
        )
        .await?;
        c.preferences.autostart = enabled;
        self.save_connection(c)
    }
    pub async fn diagnostics(&self, id: &str) -> Result<Diagnostics> {
        let c = self.connection(id)?;
        let s = status::show(self.runner.as_ref(), &c.service).await?;
        let o = self
            .runner
            .run(CommandSpec::new(
                binary("journalctl")?,
                [
                    "--user",
                    "-u",
                    &c.service,
                    "-n",
                    "40",
                    "--no-pager",
                    "--output=cat",
                ],
            ))
            .await?;
        Ok(Diagnostics {
            report: shareable_report(
                s.get("ActiveState")
                    .map(String::as_str)
                    .unwrap_or("unknown"),
                c.managed,
                mount::matches(&self.host.mounts()?, &c.mount_path, &c.remote),
            ),
            details: local_details(
                &o.stdout,
                &self.paths.home.to_string_lossy(),
                &fs::read_to_string("/etc/hostname").unwrap_or_default(),
            ),
        })
    }
    pub async fn open_drive(&self, id: &str) -> Result<()> {
        let c = self.connection(id)?;
        if !mount::matches(&self.host.mounts()?, &c.mount_path, &c.remote) {
            return Err(AppError::new(
                "not_mounted",
                "Connect Drive before opening its folder.",
            ));
        }
        self.runner
            .run(CommandSpec::new(
                binary("xdg-open")?,
                [c.mount_path.to_string_lossy().to_string()],
            ))
            .await?
            .checked()?;
        Ok(())
    }
}
pub fn cache_pending(v: &serde_json::Value) -> Option<u64> {
    Some(
        v.pointer("/diskCache/uploadsQueued")?.as_u64()?
            + v.pointer("/diskCache/uploadsInProgress")?.as_u64()?
            + v.pointer("/diskCache/erroredFiles")?.as_u64()?,
    )
}
pub fn guard_disconnect(s: &ConnectionStatus, acknowledge_unknown: bool) -> Result<()> {
    if s.pending_uploads.is_some_and(|n| n > 0) || s.open_files.is_some_and(|n| n > 0) {
        return Err(AppError::new("pending_writes","Drive has open files or unfinished uploads. Close your files and wait for uploads before disconnecting."));
    }
    if (s.mounted || ["active", "activating", "reloading"].contains(&s.active_state.as_str()))
        && (s.pending_uploads.is_none() || s.open_files.is_none())
        && !acknowledge_unknown
    {
        return Err(AppError::new("pending_unknown","DriveMount cannot confirm that all changes have uploaded. Close your files and confirm you understand that disconnecting may interrupt pending edits."));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_uploads_block_stop_even_with_override() {
        let mut s = ConnectionStatus {
            id: "x".into(),
            state: "connected".into(),
            mounted: true,
            active_state: "active".into(),
            autostart: true,
            pending_uploads: Some(1),
            open_files: Some(0),
            cache_bytes: Some(0),
            detail: "".into(),
        };
        assert_eq!(
            guard_disconnect(&s, true).unwrap_err().code,
            "pending_writes"
        );
        s.pending_uploads = None;
        assert_eq!(
            guard_disconnect(&s, false).unwrap_err().code,
            "pending_unknown"
        );
        assert!(guard_disconnect(&s, true).is_ok());
    }
    #[test]
    fn missing_cache_stats_are_unknown() {
        assert_eq!(
            cache_pending(
                &json!({"diskCache":{"uploadsQueued":2,"uploadsInProgress":1,"erroredFiles":0}})
            ),
            Some(3)
        );
        assert_eq!(cache_pending(&json!({})), None);
    }
}
