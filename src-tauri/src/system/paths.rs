use crate::{
    errors::{AppError, Result},
    models::Registry,
};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
#[derive(Debug, Clone)]
pub struct Paths {
    pub home: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
    pub units: PathBuf,
    pub runtime: PathBuf,
}
impl Paths {
    pub fn discover() -> Result<Self> {
        let home = PathBuf::from(
            std::env::var_os("HOME")
                .ok_or_else(|| AppError::new("home", "Your home directory could not be found."))?,
        );
        let xdg = |key: &str, fallback: PathBuf| {
            std::env::var_os(key)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .unwrap_or(fallback)
        };
        let config = xdg("XDG_CONFIG_HOME", home.join(".config"));
        let cache = xdg("XDG_CACHE_HOME", home.join(".cache"));
        let runtime = runtime_root(
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .as_deref(),
        )?;
        Ok(Self {
            home,
            config: config.join("drivemount"),
            cache: cache.join("drivemount"),
            units: config.join("systemd/user"),
            runtime: runtime.join("drivemount"),
        })
    }
    pub fn registry(&self) -> PathBuf {
        self.config.join("connections.json")
    }
    pub fn prepare_runtime(&self) -> Result<()> {
        let parent = self.runtime.parent().ok_or_else(runtime_error)?;
        runtime_root(Some(parent))?;
        match fs::DirBuilder::new().mode(0o700).create(&self.runtime) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => return Err(error.into()),
        }
        runtime_root(Some(&self.runtime))?;
        Ok(())
    }
    pub fn load(&self) -> Result<Registry> {
        match fs::read(self.registry()) {
            Ok(b) => {
                let r: Registry = serde_json::from_slice(&b)?;
                if r.version != 1 {
                    return Err(AppError::new(
                        "registry_version",
                        "This connection registry requires a newer version of DriveMount.",
                    ));
                }
                Ok(r)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Registry {
                version: 1,
                connections: vec![],
            }),
            Err(e) => Err(e.into()),
        }
    }
    pub fn lock(&self) -> Result<File> {
        private_dir(&self.config)?;
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.config.join("operation.lock"))?;
        f.try_lock_exclusive().map_err(|_| {
            AppError::new(
                "operation_busy",
                "Another setup or connection operation is running. Wait for it to finish.",
            )
        })?;
        Ok(f)
    }
    pub fn save(&self, registry: &Registry) -> Result<()> {
        private_dir(&self.config)?;
        atomic_write(
            &self.registry(),
            &serde_json::to_vec_pretty(registry)?,
            0o600,
        )
    }
}
fn runtime_error() -> AppError {
    AppError::new("runtime_directory", "A private desktop runtime directory is required. Sign in to a full desktop session and try again.")
}
/// Reject a missing runtime directory and any directory chain another local user can replace.
/// Root-owned sticky directories are safe ancestors of an owner-only directory (for test fixtures).
pub fn runtime_root(path: Option<&Path>) -> Result<PathBuf> {
    let path = path.filter(|p| p.is_absolute()).ok_or_else(runtime_error)?;
    let uid = unsafe { libc::geteuid() };
    let mut current = PathBuf::from("/");
    for part in path.components().skip(1) {
        if !matches!(part, std::path::Component::Normal(_)) {
            return Err(runtime_error());
        }
        current.push(part);
        let meta = fs::symlink_metadata(&current).map_err(|_| runtime_error())?;
        let root_sticky = (meta.uid() == 0 || meta.uid() == uid) && meta.mode() & 0o1000 != 0;
        if !meta.is_dir()
            || meta.file_type().is_symlink()
            || (meta.uid() != 0 && meta.uid() != uid)
            || (meta.mode() & 0o022 != 0 && !root_sticky)
        {
            return Err(runtime_error());
        }
    }
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| runtime_error())?;
    let meta = directory.metadata()?;
    if meta.uid() != uid || meta.mode() & 0o777 != 0o700 {
        return Err(runtime_error());
    }
    Ok(path.to_owned())
}
pub async fn verify_private_socket(path: &Path, expected_pid: Option<u32>) -> Result<()> {
    runtime_root(path.parent())?;
    let meta = fs::symlink_metadata(path).map_err(|_| runtime_error())?;
    if !meta.file_type().is_socket()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
    {
        return Err(runtime_error());
    }
    let stream = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::net::UnixStream::connect(path),
    )
    .await
    .map_err(|_| runtime_error())?
    .map_err(|_| runtime_error())?;
    let peer = stream.peer_cred().map_err(|_| runtime_error())?;
    if peer.uid() != unsafe { libc::geteuid() }
        || expected_pid.is_some_and(|pid| peer.pid().map(|p| p as u32) != Some(pid))
    {
        return Err(runtime_error());
    }
    Ok(())
}
pub fn private_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    let m = fs::symlink_metadata(path)?;
    if m.file_type().is_symlink() || !m.is_dir() || m.uid() != unsafe { libc::geteuid() } {
        return Err(AppError::new("directory_owner","An application folder is not owned by you or is a symbolic link. Choose a private local folder."));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
pub fn create_private(path: &Path) -> Result<()> {
    let _ = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    Ok(())
}
pub fn atomic_write(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| AppError::new("path", "A configuration path has no parent folder."))?;
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(AppError::new(
            "symlink",
            "Refusing to replace a symbolic link.",
        ));
    }
    let temp = parent.join(format!(".drivemount-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp)?;
        f.write_all(data)?;
        f.sync_all()?;
        fs::rename(&temp, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_requires_a_private_trusted_directory() {
        assert!(runtime_root(None).is_err());
        assert!(runtime_root(Some(Path::new("relative"))).is_err());
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        assert!(runtime_root(Some(root.path())).is_ok());
        let unsafe_parent = root.path().join("writable");
        fs::create_dir(&unsafe_parent).unwrap();
        fs::set_permissions(&unsafe_parent, fs::Permissions::from_mode(0o777)).unwrap();
        let leaf = unsafe_parent.join("private");
        fs::DirBuilder::new().mode(0o700).create(&leaf).unwrap();
        assert!(runtime_root(Some(&leaf)).is_err());
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&leaf, &alias).unwrap();
        assert!(runtime_root(Some(&alias)).is_err());
    }
    #[tokio::test]
    async fn socket_peer_and_owner_are_verified() {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let socket = root.path().join("rpc.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(verify_private_socket(&socket, Some(std::process::id()))
            .await
            .is_ok());
        assert!(verify_private_socket(&socket, Some(std::process::id() + 1))
            .await
            .is_err());
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(verify_private_socket(&socket, None).await.is_err());
        drop(listener);
    }
    #[test]
    fn credentials_permissions() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("secret");
        create_private(&p).unwrap();
        assert_eq!(fs::metadata(p).unwrap().permissions().mode() & 0o777, 0o600);
    }
    #[test]
    fn prevents_duplicate_setup() {
        let t = tempfile::tempdir().unwrap();
        let p = Paths {
            home: t.path().into(),
            config: t.path().join("config"),
            cache: t.path().join("cache"),
            units: t.path().join("units"),
            runtime: t.path().join("run"),
        };
        let _l = p.lock().unwrap();
        assert!(p.lock().is_err());
    }
    #[test]
    fn registry_roundtrip_and_interrupted_write() {
        let t = tempfile::tempdir().unwrap();
        let p = Paths {
            home: t.path().into(),
            config: t.path().join("config"),
            cache: t.path().join("cache"),
            units: t.path().join("units"),
            runtime: t.path().join("run"),
        };
        let _l = p.lock().unwrap();
        p.save(&Registry {
            version: 1,
            connections: vec![],
        })
        .unwrap();
        fs::write(p.config.join(".drivemount-interrupted.tmp"), "partial").unwrap();
        assert!(p.load().unwrap().connections.is_empty());
    }
}
