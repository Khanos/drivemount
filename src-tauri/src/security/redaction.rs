use regex::Regex;
/// Diagnostics deliberately omit entire potentially sensitive lines, including unknown JSON layouts.
pub fn redact(input: &str) -> String {
    let sensitive=Regex::new(r"(?i)(token|secret|authorization|client[_ -]?id|password|bearer|ya29\.|1//|gocspx|accounts\.google\.com|https?://[^\s]*[?&](code|state)=)").unwrap();
    let opaque = Regex::new(r"[A-Za-z0-9_+/=-]{32,}").unwrap();
    input
        .lines()
        .map(|line| {
            if sensitive.is_match(line) || opaque.is_match(line) {
                "[Sensitive diagnostic line removed]".into()
            } else {
                line.chars()
                    .filter(|c| !c.is_control() || *c == '\t')
                    .take(500)
                    .collect::<String>()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
/// Only fixed vocabulary and booleans enter the export; no journal, identifiers or paths.
pub fn shareable_report(active: &str, managed: bool, mounted: bool) -> String {
    let state = match active {
        "active" | "inactive" | "activating" | "deactivating" | "failed" | "reloading" => active,
        _ => "unknown",
    };
    format!("DriveMount {}\nService state: {state}\nManaged by DriveMount: {managed}\nMount detected: {mounted}\nNo account, folder, computer names or service messages are included.", env!("CARGO_PKG_VERSION"))
}
/// Best-effort filtering for local troubleshooting, never included in the shareable export.
pub fn local_details(input: &str, home: &str, hostname: &str) -> String {
    let mut text = redact(input);
    if !home.is_empty() {
        text = text.replace(home, "[home]");
    }
    let hostname = hostname.trim();
    if !hostname.is_empty() {
        text = text.replace(hostname, "[computer]");
    }
    Regex::new(r"[^\s<>]+@[^\s<>]+\.[^\s<>]+")
        .unwrap()
        .replace_all(&text, "[email]")
        .into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_rejects_arbitrary_system_text() {
        let report = shareable_report(
            "workstation /home/person alice@example.test secret",
            true,
            false,
        );
        for private in ["workstation", "/home/person", "alice@", "secret"] {
            assert!(!report.contains(private));
        }
        assert!(report.contains("Service state: unknown"));
        let details = local_details(
            "laptop /home/person alice@example.test\ntoken=hidden",
            "/home/person",
            "laptop\n",
        );
        for private in ["laptop", "/home/person", "alice@", "hidden"] {
            assert!(!details.contains(private));
        }
    }
    #[test]
    fn removes_credentials() {
        let s=redact("healthy\naccess_token: topsecret\nAuthorization: Bearer x\nclient_secret='hidden'\nrefresh_token\n  ya29.hidden");
        assert!(s.starts_with("healthy"));
        assert!(!s.contains("topsecret"));
        assert!(!s.contains("hidden"));
    }
}
