use crate::{
    errors::{AppError, Result},
    models::Connection,
    security::validation,
    system::{
        paths::atomic_write,
        process::{binary, CommandSpec, Runner},
    },
};
use std::{fs, path::Path};
pub fn argument(s: &str) -> Result<String> {
    if s.chars().any(|c| c.is_control()) {
        return Err(AppError::new(
            "unit_argument",
            "A service setting contains an unsupported control character.",
        ));
    }
    Ok(format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('%', "%%")
            .replace('$', "$$")
    ))
}
pub fn marker(id: &str) -> String {
    format!("# DriveMount v1 connection={id}")
}
pub fn generate(c: &Connection, rclone: &Path, fuse: &Path) -> Result<String> {
    validation::remote_name(&c.remote)?;
    validation::unit_name(&c.service)?;
    validation::preferences(&c.preferences)?;
    let mut args = vec![
        rclone.to_string_lossy().to_string(),
        "mount".into(),
        format!("{}:", c.remote),
        c.mount_path.to_string_lossy().into(),
        "--config".into(),
        c.config_path.to_string_lossy().into(),
        "--cache-dir".into(),
        c.cache_path.to_string_lossy().into(),
        "--vfs-cache-mode".into(),
        c.preferences.cache.mode.clone(),
        "--vfs-cache-max-size".into(),
        format!("{}G", c.preferences.cache.size_gib),
        "--vfs-cache-max-age".into(),
        format!("{}h", c.preferences.cache.age_hours),
        "--rc".into(),
        "--rc-addr".into(),
        format!("unix://{}", c.socket_path.display()),
        "--rc-no-auth".into(),
        "--log-level".into(),
        "ERROR".into(),
    ];
    if c.preferences.read_only {
        args.push("--read-only".into());
    }
    let command = args
        .iter()
        .map(|s| argument(s))
        .collect::<Result<Vec<_>>>()?
        .join(" ");
    let stop = [
        fuse.to_string_lossy().to_string(),
        "-u".into(),
        c.mount_path.to_string_lossy().to_string(),
    ]
    .iter()
    .map(|s| argument(s))
    .collect::<Result<Vec<_>>>()?
    .join(" ");
    Ok(format!("{}\n[Unit]\nDescription=DriveMount Google Drive\nStartLimitIntervalSec=0\n\n[Service]\nType=notify\nUMask=0077\nRuntimeDirectory=drivemount\nRuntimeDirectoryMode=0700\nRuntimeDirectoryPreserve=yes\nExecStart={}\nExecStop={}\nRestart=on-failure\nRestartSec=15s\nTimeoutStartSec=90s\nTimeoutStopSec=45s\n\n[Install]\nWantedBy=default.target\n",marker(&c.id),command,stop))
}
pub fn write_owned(path: &Path, c: &Connection, unit: &str) -> Result<()> {
    if path.exists() {
        let old = fs::read_to_string(path)?;
        if old.lines().next() != Some(marker(&c.id).as_str()) {
            return Err(AppError::new(
                "unowned_service",
                "This service belongs to another setup. DriveMount will not replace it.",
            ));
        }
    }
    atomic_write(path, unit.as_bytes(), 0o600)
}
pub async fn action(r: &dyn Runner, verb: &str, unit: Option<&str>) -> Result<()> {
    if ![
        "daemon-reload",
        "enable",
        "disable",
        "start",
        "stop",
        "restart",
    ]
    .contains(&verb)
    {
        return Err(AppError::new(
            "service_action",
            "This service operation is not allowed.",
        ));
    }
    let mut args: Vec<String> = vec!["--user".into(), verb.into()];
    if let Some(unit) = unit {
        validation::unit_name(unit)?;
        args.push(unit.into());
    }
    r.run(CommandSpec::new(binary("systemctl")?, args).seconds(100))
        .await?
        .checked()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_quoting() {
        assert_eq!(
            argument("/home/文/My %Drive $HOME/\"x\"").unwrap(),
            "\"/home/文/My %%Drive $$HOME/\\\"x\\\"\""
        );
        assert!(argument("x\nExecStart=bad").is_err());
    }
    #[test]
    fn preserves_unowned_unit() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("other.service");
        fs::write(&p, "[Service]\nExecStart=keep").unwrap();
        let c = crate::tests::connection(t.path());
        assert!(write_owned(&p, &c, "replacement").is_err());
        assert!(fs::read_to_string(p).unwrap().contains("keep"));
    }
    #[test]
    fn generated_unit_is_explicit_and_secret_free() {
        let t = tempfile::tempdir().unwrap();
        let mut c = crate::tests::connection(t.path());
        c.mount_path = t.path().join("My Drive % 文");
        c.preferences.read_only = true;
        let u = generate(
            &c,
            Path::new("/usr/bin/rclone"),
            Path::new("/usr/bin/fusermount3"),
        )
        .unwrap();
        assert!(u.contains("--config"));
        assert!(u.contains("--read-only"));
        assert!(u.contains("My Drive %% 文"));
        assert!(!u.contains("secret"));
        assert!(!u.contains("network-online"));
    }
}
