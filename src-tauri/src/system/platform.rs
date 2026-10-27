use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Pacman,
    Apt,
    Dnf,
}
#[derive(Debug, Clone)]
pub struct Platform {
    pub name: String,
    pub manager: Option<PackageManager>,
}
pub fn detect(text: &str) -> Platform {
    let field = |key: &str| {
        text.lines()
            .find_map(|l| {
                l.strip_prefix(&format!("{key}="))
                    .map(|v| v.trim_matches('"').trim_matches('\'').to_string())
            })
            .unwrap_or_default()
    };
    let id = field("ID");
    let family = field("ID_LIKE");
    let ids = format!("{id} {family}");
    let ids: Vec<_> = ids.split_whitespace().collect();
    let manager = if ids.contains(&"arch") {
        Some(PackageManager::Pacman)
    } else if ids.iter().any(|x| ["debian", "ubuntu"].contains(x)) {
        Some(PackageManager::Apt)
    } else if ids.iter().any(|x| ["fedora", "rhel"].contains(x)) {
        Some(PackageManager::Dnf)
    } else {
        None
    };
    Platform {
        name: field("PRETTY_NAME"),
        manager,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distro_families() {
        for (s, m) in [
            ("ID=omarchy\nID_LIKE=arch", Some(PackageManager::Pacman)),
            (
                "ID=linuxmint\nID_LIKE=\"ubuntu debian\"",
                Some(PackageManager::Apt),
            ),
            ("ID=fedora", Some(PackageManager::Dnf)),
            ("ID=alpine", None),
        ] {
            assert_eq!(detect(s).manager, m);
        }
    }
}
