use super::{
    platform::PackageManager,
    process::{CommandSpec, Runner},
};
use crate::errors::{AppError, Result};
pub fn install_arguments(manager: &PackageManager, packages: &[String]) -> Result<Vec<String>> {
    let allowed = ["rclone", "fuse3"];
    if packages.is_empty() || packages.iter().any(|p| !allowed.contains(&p.as_str())) {
        return Err(AppError::new(
            "packages",
            "Only the missing Drive connection components may be installed.",
        ));
    }
    let mut args = match manager {
        PackageManager::Pacman => vec!["-S", "--needed", "--noconfirm"],
        PackageManager::Apt => vec!["install", "--yes"],
        PackageManager::Dnf => vec!["install", "--assumeyes"],
    }
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    args.extend_from_slice(packages);
    Ok(args)
}
pub async fn install(
    r: &dyn Runner,
    manager: &PackageManager,
    packages: &[String],
    consent: bool,
) -> Result<()> {
    if !consent {
        return Err(AppError::new(
            "consent",
            "Review and approve the listed packages before installation.",
        ));
    }
    let name = match manager {
        PackageManager::Pacman => "pacman",
        PackageManager::Apt => "apt-get",
        PackageManager::Dnf => "dnf",
    };
    // Never elevate an executable resolved from a user-writable PATH directory.
    let executable = std::fs::canonicalize(std::path::Path::new("/usr/bin").join(name))?;
    if !executable.starts_with("/usr/bin/") {
        return Err(AppError::new(
            "package_manager",
            "The system package manager could not be verified.",
        ));
    }
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(&executable)?;
    if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
        return Err(AppError::new(
            "package_manager",
            "The system package manager has unsafe ownership or permissions.",
        ));
    }
    // Only a fixed package-manager executable and backend-produced allowlisted package names reach pkexec.
    let mut args = vec![executable.to_string_lossy().to_string()];
    args.extend(install_arguments(manager, packages)?);
    r.run(CommandSpec::new("/usr/bin/pkexec", args).seconds(900))
        .await?
        .checked()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_arguments_are_restricted() {
        assert!(
            install_arguments(&PackageManager::Apt, &["--allow-unauthenticated".into()]).is_err()
        );
        assert_eq!(
            install_arguments(&PackageManager::Pacman, &["rclone".into()]).unwrap(),
            vec!["-S", "--needed", "--noconfirm", "rclone"]
        );
    }
}
