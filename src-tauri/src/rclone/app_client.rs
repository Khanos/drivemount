use crate::security::validation;
use serde::Deserialize;
use std::{fs, path::PathBuf};
#[derive(Deserialize, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct DesktopClient {
    pub client_id: String,
    pub client_secret: String,
}
/// Optional distributor-supplied public Desktop client registration; never a web/server client.
/// Google Desktop client secrets are public-client parameters, not a server-side security boundary.
pub fn load() -> Option<DesktopClient> {
    let path = std::env::var_os("DRIVEMOUNT_OAUTH_CLIENT_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/drivemount/oauth-client.json"));
    let bytes = zeroize::Zeroizing::new(fs::read(path).ok()?);
    if bytes.len() > 16 * 1024 {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let installed = value.get("installed")?;
    let client: DesktopClient = serde_json::from_value(installed.clone()).ok()?;
    validation::oauth(&client.client_id, &client.client_secret).ok()?;
    Some(client)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_registration_validation() {
        assert!(
            validation::oauth("fixture.apps.googleusercontent.com", "not-a-real-secret").is_ok()
        );
        assert!(validation::oauth("web.example.com", "not-a-real-secret").is_err());
        let v:serde_json::Value=serde_json::from_str(r#"{"web":{"client_id":"fixture.apps.googleusercontent.com","client_secret":"not-a-real-secret"}}"#).unwrap();
        assert!(v.get("installed").is_none());
    }
}
