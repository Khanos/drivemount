use crate::{
    errors::{AppError, Result},
    models::{ExistingMount, PathValidation},
    security::validation::absolute_path,
    system::paths::Paths,
};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone)]
pub struct Mount {
    pub target: PathBuf,
    pub source: String,
    pub kind: String,
}
fn decode(s: &str) -> String {
    s.replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}
pub fn parse_mountinfo(text: &str) -> Vec<Mount> {
    text.lines()
        .filter_map(|line| {
            let (left, right) = line.split_once(" - ")?;
            let a: Vec<_> = left.split_whitespace().collect();
            let b: Vec<_> = right.split_whitespace().collect();
            Some(Mount {
                target: decode(a.get(4)?).into(),
                kind: b.first()?.to_string(),
                source: decode(b.get(1)?),
            })
        })
        .collect()
}
pub fn mounts() -> Result<Vec<Mount>> {
    Ok(parse_mountinfo(&fs::read_to_string(
        "/proc/self/mountinfo",
    )?))
}
pub fn drives(list: &[Mount]) -> Vec<ExistingMount> {
    list.iter()
        .filter(|m| m.kind == "fuse.rclone")
        .map(|m| ExistingMount {
            target: m.target.clone(),
            source: m.source.clone(),
            service: None,
        })
        .collect()
}
pub fn matches(list: &[Mount], path: &Path, remote: &str) -> bool {
    list.iter()
        .any(|m| m.target == path && m.kind == "fuse.rclone" && m.source == format!("{remote}:"))
}
pub fn validate(value: &str, paths: &Paths, list: &[Mount]) -> Result<PathValidation> {
    let path = absolute_path(value, &paths.home)?;
    let invalid = |reason: &str| {
        Ok(PathValidation {
            path: path.clone(),
            valid: false,
            exists: path.exists(),
            reason: reason.into(),
        })
    };
    // Mounts are inspected before opening the path: a stale FUSE mount can block filesystem calls.
    if list.iter().any(|m| {
        m.target == path
            || m.target.starts_with(&path)
            || (m.kind.starts_with("fuse.") && path.starts_with(&m.target))
    }) {
        return Ok(PathValidation{path,valid:false,exists:true,reason:"This folder overlaps an existing mount. Choose another location or manage that connection.".into()});
    }
    if paths.home.starts_with(&path)
        || path.starts_with(&paths.config)
        || path.starts_with(&paths.cache)
        || paths.config.starts_with(&path)
        || paths.cache.starts_with(&path)
    {
        return invalid(
            "Choose a dedicated Drive folder outside application configuration and cache folders.",
        );
    }
    let mut current = PathBuf::from("/");
    for c in path.components().skip(1) {
        current.push(c);
        if fs::symlink_metadata(&current).is_ok_and(|m| m.file_type().is_symlink()) {
            return invalid("Choose a direct folder path rather than a symbolic link.");
        }
    }
    let exists = path.exists();
    if exists {
        let m = fs::metadata(&path)?;
        if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } {
            return invalid("Choose a folder owned by your user account.");
        }
        if fs::read_dir(&path)?.next().is_some() {
            return invalid("This folder contains files. Choose or create an empty folder to keep them visible and safe.");
        }
    }
    let mut parent = if exists {
        path.as_path()
    } else {
        path.parent().unwrap_or(Path::new("/"))
    };
    while !parent.exists() {
        parent = parent
            .parent()
            .ok_or_else(|| AppError::new("mount_path", "The parent folder could not be found."))?;
    }
    let m = fs::metadata(parent)?;
    if m.uid() != unsafe { libc::geteuid() } {
        return invalid("Choose a location inside a folder owned by your account.");
    }
    use std::os::unix::ffi::OsStrExt;
    let p = std::ffi::CString::new(parent.as_os_str().as_bytes())
        .map_err(|_| AppError::new("mount_path", "This folder path is invalid."))?;
    if unsafe { libc::access(p.as_ptr(), libc::W_OK | libc::X_OK) } != 0 {
        return invalid("You need permission to create and access files in this folder.");
    }
    Ok(PathValidation {
        path,
        valid: true,
        exists,
        reason: "An empty, accessible folder is ready for your Drive.".into(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn paths(t: &Path) -> Paths {
        Paths {
            home: t.into(),
            config: t.join("config"),
            cache: t.join("cache"),
            units: t.join("units"),
            runtime: t.join("run"),
        }
    }
    #[test]
    fn parses_escaped_paths() {
        let m = parse_mountinfo("22 1 0:2 / /home/u/My\\040Drive rw - fuse.rclone gdrive: rw");
        assert_eq!(m[0].target, PathBuf::from("/home/u/My Drive"));
        assert!(matches(&m, Path::new("/home/u/My Drive"), "gdrive"));
    }
    #[test]
    fn never_hides_existing_files() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("Drive");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("keep.txt"), "keep").unwrap();
        assert!(
            !validate(dir.to_str().unwrap(), &paths(t.path()), &[])
                .unwrap()
                .valid
        );
    }
    #[test]
    fn rejects_existing_mount_and_symlink() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("My Drive 文");
        assert!(
            validate(dir.to_str().unwrap(), &paths(t.path()), &[])
                .unwrap()
                .valid
        );
        let list = vec![Mount {
            target: dir.clone(),
            source: "other:".into(),
            kind: "fuse.rclone".into(),
        }];
        assert!(
            !validate(dir.to_str().unwrap(), &paths(t.path()), &list)
                .unwrap()
                .valid
        );
        std::os::unix::fs::symlink(t.path(), t.path().join("link")).unwrap();
        assert!(
            !validate(
                t.path().join("link/Drive").to_str().unwrap(),
                &paths(t.path()),
                &[]
            )
            .unwrap()
            .valid
        );
    }
}
