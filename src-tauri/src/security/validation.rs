use crate::{
    errors::{AppError, Result},
    models::Preferences,
};
use std::path::{Component, Path, PathBuf};
pub fn remote_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name.as_bytes()[0].is_ascii_alphabetic()
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err(AppError::new("remote_name","Use a name starting with a letter, containing only letters, numbers, underscores or hyphens."));
    }
    Ok(())
}
pub fn unit_name(name: &str) -> Result<()> {
    if !name.ends_with(".service")
        || name.len() > 240
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.@\\".contains(&c))
    {
        return Err(AppError::new(
            "service_name",
            "This service name cannot be managed safely.",
        ));
    }
    Ok(())
}
pub fn absolute_path(value: &str, home: &Path) -> Result<PathBuf> {
    let p = if value == "~" {
        home.to_owned()
    } else if let Some(rest) = value.strip_prefix("~/") {
        home.join(rest)
    } else {
        PathBuf::from(value)
    };
    if !p.is_absolute()
        || value.chars().any(|c| c.is_control())
        || p.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err(AppError::new(
            "mount_path",
            "Choose an absolute folder path without '..' or control characters.",
        ));
    }
    Ok(p)
}
pub fn preferences(p: &Preferences) -> Result<()> {
    if !["full", "writes", "minimal"].contains(&p.cache.mode.as_str())
        || !(1..=1024).contains(&p.cache.size_gib)
        || !(1..=8760).contains(&p.cache.age_hours)
    {
        return Err(AppError::new(
            "preferences",
            "Choose a cache size from 1–1024 GiB and an age from 1–8760 hours.",
        ));
    }
    Ok(())
}
pub fn oauth(id: &str, secret: &str) -> Result<()> {
    if !id.ends_with(".apps.googleusercontent.com")
        || id.len() > 256
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-._".contains(&c))
        || secret.trim().len() < 8
        || secret.len() > 512
        || secret.chars().any(|c| c.is_control())
    {
        return Err(AppError::new(
            "oauth_credentials",
            "Enter the client ID and client secret from your Google Desktop app credentials.",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_are_not_arguments() {
        for n in ["", "--config", "a:b", "a\nx", "a/b", "a.b"] {
            assert!(remote_name(n).is_err());
        }
        assert!(remote_name("gdrive_2").is_ok());
    }
    #[test]
    fn paths_allow_unicode_and_spaces() {
        assert_eq!(
            absolute_path("~/My Drive/文", Path::new("/home/test")).unwrap(),
            PathBuf::from("/home/test/My Drive/文")
        );
        assert!(absolute_path("/tmp/../home", Path::new("/x")).is_err());
    }
    #[test]
    fn cache_is_bounded() {
        let mut p = Preferences::default();
        assert!(preferences(&p).is_ok());
        p.cache.size_gib = 0;
        assert!(preferences(&p).is_err());
    }
}
