//! Conservative discovery: never interpret an arbitrary service as a shell script.
use crate::{
    errors::Result,
    models::ExistingService,
    security::validation,
    system::{paths::Paths, process::Runner},
    systemd::status,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
fn words(text: &str) -> Option<Vec<String>> {
    let mut words = vec![];
    let mut word = String::new();
    let mut quote = None;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let escaped = chars.next()?;
                word.push(match escaped {
                    '\\' | '"' | '\'' => escaped,
                    's' => ' ',
                    'x' => {
                        let hex: String = chars.by_ref().take(2).collect();
                        if hex.len() != 2 {
                            return None;
                        }
                        let b = u8::from_str_radix(&hex, 16).ok()?;
                        if !b.is_ascii() || b.is_ascii_control() {
                            return None;
                        }
                        b as char
                    }
                    _ => return None,
                });
            }
            '\'' | '"' => {
                if quote == Some(ch) {
                    quote = None
                } else if quote.is_none() {
                    quote = Some(ch)
                } else {
                    word.push(ch)
                }
            }
            c if c.is_whitespace() && quote.is_none() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
    }
    if quote.is_some() {
        return None;
    }
    if !word.is_empty() {
        words.push(word)
    }
    Some(words)
}
fn expand(value: &str, home: &Path) -> Option<String> {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '%' => match chars.next()? {
                '%' => out.push('%'),
                'h' => out.push_str(home.to_str()?),
                _ => return None,
            },
            '$' => {
                if chars.next()? != '$' {
                    return None;
                }
                out.push('$')
            }
            _ => out.push(c),
        }
    }
    Some(out)
}
pub fn parse_mount(text: &str, home: &Path) -> Option<(String, PathBuf, Option<PathBuf>)> {
    let joined = text.replace("\\\n", " ");
    let commands: Vec<_> = joined
        .lines()
        .filter_map(|l| l.trim().strip_prefix("ExecStart="))
        .collect();
    if commands.len() != 1 {
        return None;
    }
    let args = words(commands[0])?;
    if Path::new(args.first()?).file_name()? != "rclone" {
        return None;
    }
    let i = args.iter().position(|a| a == "mount")?;
    let remote = args.get(i + 1)?.strip_suffix(':')?;
    validation::remote_name(remote).ok()?;
    let mount = PathBuf::from(expand(args.get(i + 2)?, home)?);
    if !mount.is_absolute() {
        return None;
    }
    let config = args
        .iter()
        .position(|a| a == "--config")
        .and_then(|n| args.get(n + 1))
        .and_then(|s| expand(s, home))
        .map(PathBuf::from);
    Some((remote.into(), mount, config))
}
pub async fn discover(r: &dyn Runner, p: &Paths) -> Result<(Vec<ExistingService>, bool)> {
    let mut files = BTreeMap::new();
    // Higher-priority user files override system-provided user units of the same name.
    for dir in [
        PathBuf::from("/usr/lib/systemd/user"),
        PathBuf::from("/etc/systemd/user"),
        p.units.clone(),
    ] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|x| x == "service") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        files.insert(name.to_string(), path);
                    }
                }
            }
        }
    }
    let mut services = vec![];
    let mut ambiguous = false;
    for (name, path) in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if !text.contains("rclone") || !text.contains("mount") {
            continue;
        }
        let Some((remote, mount_path, config_path)) = parse_mount(&text, &p.home) else {
            ambiguous = true;
            continue;
        };
        // Drop-in overrides may change semantics; leave those setups under their original tool.
        if p.units.join(format!("{name}.d")).exists()
            || PathBuf::from("/etc/systemd/user")
                .join(format!("{name}.d"))
                .exists()
        {
            ambiguous = true;
            continue;
        }
        if validation::unit_name(&name).is_err() {
            ambiguous = true;
            continue;
        }
        let Ok(s) = status::show(r, &name).await else {
            continue;
        };
        services.push(ExistingService {
            name,
            remote,
            mount_path,
            config_path,
            autostart: s.get("UnitFileState").is_some_and(|v| v == "enabled"),
            managed: text.starts_with("# DriveMount v1 connection="),
        });
    }
    Ok((services, ambiguous))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_reference_service() {
        let text="[Service]\nExecStart=/usr/bin/rclone mount gdrive: %h/GoogleDrive \\\n --vfs-cache-mode full";
        let (r, p, c) = parse_mount(text, Path::new("/home/test")).unwrap();
        assert_eq!(r, "gdrive");
        assert_eq!(p, PathBuf::from("/home/test/GoogleDrive"));
        assert!(c.is_none());
    }
    #[test]
    fn generated_unit_roundtrip() {
        let t = tempfile::tempdir().unwrap();
        let mut c = crate::tests::connection(t.path());
        c.mount_path = t.path().join("My Drive % $ 文");
        let text = crate::systemd::service::generate(
            &c,
            Path::new("/usr/bin/rclone"),
            Path::new("/usr/bin/fusermount3"),
        )
        .unwrap();
        let (_, p, config) = parse_mount(&text, t.path()).unwrap();
        assert_eq!(p, c.mount_path);
        assert_eq!(config, Some(c.config_path));
    }
    #[test]
    fn never_guesses_shell_or_environment() {
        assert!(parse_mount(
            "ExecStart=/bin/sh -c 'rclone mount x: /home/u/Drive'",
            Path::new("/home/u")
        )
        .is_none());
        assert!(parse_mount(
            "ExecStart=/usr/bin/rclone mount x: ${HOME}/Drive",
            Path::new("/home/u")
        )
        .is_none());
    }
}
