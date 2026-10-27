use crate::{
    errors::Result,
    models::{Check, Inspection},
    rclone::{detection, mount},
    system::{
        paths::Paths,
        platform,
        process::{binary, CommandSpec, Runner},
    },
    systemd,
};
use std::{fs, fs::OpenOptions, path::PathBuf};
pub async fn online() -> bool {
    let Ok(c) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    else {
        return false;
    };
    c.head("https://www.googleapis.com/").send().await.is_ok()
}
fn check(id: &str, label: &str, state: &str, detail: impl Into<String>) -> Check {
    Check {
        id: id.into(),
        label: label.into(),
        state: state.into(),
        detail: detail.into(),
    }
}
pub async fn inspect(r: &dyn Runner, p: &Paths) -> Result<Inspection> {
    let platform = platform::detect(&fs::read_to_string("/etc/os-release").unwrap_or_default());
    let mut checks = vec![];
    let mut packages = vec![];
    checks.push(check(
        "platform",
        "Linux distribution",
        if platform.manager.is_some() {
            "ready"
        } else {
            "unsupported"
        },
        &platform.name,
    ));
    let mut remotes = vec![];
    match binary("rclone") {
        Ok(b) => {
            let o = r.run(CommandSpec::new(b, ["version"])).await;
            match o {
                Ok(o) if o.success => {
                    checks.push(check(
                        "rclone",
                        "Google Drive connector",
                        if detection::version_supported(&o.stdout) {
                            "ready"
                        } else {
                            "attention"
                        },
                        o.stdout.lines().next().unwrap_or("Unknown version"),
                    ));
                    if let Ok(path) = detection::config_path(r).await {
                        match detection::remotes(r, &path).await {
                            Ok(found) => remotes = found,
                            Err(e) => checks.push(check(
                                "existing_config",
                                "Existing account configuration",
                                "attention",
                                e.message,
                            )),
                        }
                    }
                }
                _ => checks.push(check(
                    "rclone",
                    "Google Drive connector",
                    "attention",
                    "Rclone could not be started.",
                )),
            }
        }
        Err(_) => {
            packages.push("rclone".into());
            checks.push(check(
                "rclone",
                "Google Drive connector",
                "install",
                "Install rclone to connect your account.",
            ));
        }
    }
    let fuse = binary("fusermount3");
    if fuse.is_err() {
        packages.push("fuse3".into());
    }
    checks.push(check(
        "fuse",
        "Filesystem support",
        if fuse.is_ok() { "ready" } else { "install" },
        if fuse.is_ok() {
            "FUSE3 is installed."
        } else {
            "Install FUSE3 to show Drive as a folder."
        },
    ));
    let device = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/fuse")
        .is_ok();
    checks.push(check("fuse_device","Filesystem device access",if device{"ready"}else{"attention"},if device{"Your account can access the FUSE device."}else{"The FUSE device is unavailable. Your administrator may need to enable FUSE or grant access."}));
    let systemd = match binary("systemctl") {
        Ok(b) => r
            .run(CommandSpec::new(
                b,
                ["--user", "show", "--property=Version"],
            ))
            .await
            .is_ok_and(|o| o.success),
        Err(_) => false,
    };
    checks.push(check(
        "systemd",
        "Automatic connection support",
        if systemd { "ready" } else { "attention" },
        if systemd {
            "Your desktop service manager is available."
        } else {
            "Sign in to a systemd-based desktop session to manage connections."
        },
    ));
    let net = online().await;
    checks.push(check(
        "internet",
        "Internet connection",
        if net { "ready" } else { "attention" },
        if net {
            "Google's servers can be reached."
        } else {
            "Google cannot be reached. You can review settings while offline."
        },
    ));
    let mut mounts = mount::drives(&mount::mounts()?);
    // Associate only services whose actual running process arguments match both remote and mount path.
    if systemd {
        if let Ok(b) = binary("systemctl") {
            if let Ok(out) = r
                .run(CommandSpec::new(
                    b,
                    [
                        "--user",
                        "list-units",
                        "--type=service",
                        "--all",
                        "--plain",
                        "--no-legend",
                        "--no-pager",
                    ],
                ))
                .await
            {
                for line in out.stdout.lines() {
                    let Some(name) = line.split_whitespace().next() else {
                        continue;
                    };
                    if crate::security::validation::unit_name(name).is_err() {
                        continue;
                    }
                    if let Ok(s) = systemd::status::show(r, name).await {
                        let pid = s
                            .get("MainPID")
                            .and_then(|v| v.parse::<u32>().ok())
                            .unwrap_or(0);
                        if pid == 0 {
                            continue;
                        }
                        if let Ok(args) = fs::read(format!("/proc/{pid}/cmdline")) {
                            let args: Vec<_> = args
                                .split(|b| *b == 0)
                                .filter_map(|a| std::str::from_utf8(a).ok())
                                .collect();
                            if PathBuf::from(args.first().unwrap_or(&""))
                                .file_name()
                                .is_some_and(|n| n == "rclone")
                            {
                                if let Some(i) = args.iter().position(|a| *a == "mount") {
                                    if let (Some(remote), Some(path)) =
                                        (args.get(i + 1), args.get(i + 2))
                                    {
                                        for m in &mut mounts {
                                            if m.source == *remote
                                                && m.target == PathBuf::from(path)
                                            {
                                                m.service = Some(name.into());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let (services, ambiguous) = if systemd {
        systemd::discovery::discover(r, p).await?
    } else {
        (vec![], false)
    };
    if ambiguous {
        checks.push(check("existing_services", "Existing connection service", "attention", "An existing rclone service uses custom shell commands or overrides. Review it in its original tool before adding a connection."));
    }
    let mut free_path = p.cache.as_path();
    while !free_path.exists() {
        free_path = free_path.parent().unwrap_or(std::path::Path::new("/"));
    }
    let free_cache_bytes = fs2::available_space(free_path).unwrap_or(0);
    let mut connections = p.load()?.connections;
    // Inspection stays read-only. A unit written before an interrupted startup remains manageable.
    for c in &mut connections {
        if !c.configured
            && c.managed
            && fs::read_to_string(p.units.join(&c.service)).is_ok_and(|text| {
                text.lines().next() == Some(systemd::service::marker(&c.id).as_str())
            })
        {
            c.configured = true;
        }
    }
    Ok(Inspection {
        distro: platform.name,
        package_manager: platform.manager.map(|m| {
            match m {
                platform::PackageManager::Pacman => "pacman",
                platform::PackageManager::Apt => "apt",
                platform::PackageManager::Dnf => "dnf",
            }
            .into()
        }),
        packages,
        checks,
        remotes,
        mounts,
        services,
        connections,
        default_mount: p.home.join("GoogleDrive"),
        app_oauth_available: crate::rclone::app_client::load().is_some(),
        free_cache_bytes,
    })
}
