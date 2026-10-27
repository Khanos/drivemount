use crate::{
    errors::{AppError, Result},
    models::Remote,
    system::process::{binary, CommandSpec, Runner},
};
use std::path::{Path, PathBuf};
pub fn version_supported(text: &str) -> bool {
    let Some(v) = text.lines().next().and_then(|s| s.strip_prefix("rclone v")) else {
        return false;
    };
    let p: Vec<_> = v.split('.').collect();
    let major = p.first().and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
    let minor = p.get(1).and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
    major > 1 || (major == 1 && minor >= 70)
}
pub async fn config_path(r: &dyn Runner) -> Result<PathBuf> {
    let o = r
        .run(CommandSpec::new(binary("rclone")?, ["config", "file"]))
        .await?
        .checked()?;
    let path = o
        .stdout
        .lines()
        .rev()
        .find(|l| l.starts_with('/'))
        .ok_or_else(|| {
            AppError::new(
                "rclone_config",
                "Rclone did not report an absolute configuration path.",
            )
        })?;
    Ok(PathBuf::from(path))
}
pub fn parse_remotes(text: &str, path: &Path) -> Vec<Remote> {
    text.lines()
        .filter_map(|l| {
            let (name, backend) = l.split_once(':')?;
            let backend = backend.trim();
            if backend != "drive" {
                return None;
            }
            Some(Remote {
                name: name.trim().into(),
                backend: backend.into(),
                config_path: path.into(),
            })
        })
        .collect()
}
pub async fn remotes(r: &dyn Runner, path: &Path) -> Result<Vec<Remote>> {
    let o = r
        .run(CommandSpec::new(
            binary("rclone")?,
            [
                String::from("listremotes"),
                "--long".into(),
                "--config".into(),
                path.to_string_lossy().into(),
            ],
        ))
        .await?
        .checked()?;
    Ok(parse_remotes(&o.stdout, path))
}
pub async fn verify_remote(r: &dyn Runner, path: &Path, name: &str) -> Result<()> {
    crate::security::validation::remote_name(name)?;
    r.run(
        CommandSpec::new(
            binary("rclone")?,
            [
                String::from("about"),
                format!("{name}:"),
                "--json".into(),
                "--config".into(),
                path.to_string_lossy().into(),
                "--contimeout".into(),
                "5s".into(),
                "--timeout".into(),
                "10s".into(),
                "--retries".into(),
                "1".into(),
                "--low-level-retries".into(),
                "1".into(),
            ],
        )
        .seconds(20),
    )
    .await?
    .checked()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_without_reading_tokens() {
        let r = parse_remotes(
            "gdrive: drive\narchive: s3\nother: drive",
            Path::new("/tmp/r.conf"),
        );
        assert_eq!(r.len(), 2);
        assert_eq!(r[1].name, "other");
    }
    #[test]
    fn checks_compatibility() {
        assert!(version_supported("rclone v1.75.2\n-os/version: x"));
        assert!(!version_supported("rclone v1.60.1"));
        assert!(!version_supported("garbage"));
    }
}
