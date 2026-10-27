use serde::Serialize;
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
pub type Result<T> = std::result::Result<T, AppError>;
impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable: true,
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(_: std::io::Error) -> Self {
        Self::new(
            "filesystem",
            "A local file could not be accessed. Check folder ownership and available disk space.",
        )
    }
}
impl From<serde_json::Error> for AppError {
    fn from(_: serde_json::Error) -> Self {
        Self::new("invalid_data","Saved data or a system response could not be read. Open diagnostics for recovery information.")
    }
}
pub fn classify(text: &str) -> AppError {
    let text = text.to_lowercase();
    if [
        "invalid_grant",
        "invalid credentials",
        "token has been expired",
        "unauthorized",
        "invalid_client",
    ]
    .iter()
    .any(|x| text.contains(x))
    {
        AppError::new(
            "authentication",
            "Your Google connection needs to be renewed. Connect your account again.",
        )
    } else if [
        "network is unreachable",
        "no such host",
        "temporary failure in name",
        "connection timed out",
    ]
    .iter()
    .any(|x| text.contains(x))
    {
        AppError::new(
            "offline",
            "You're offline or Google cannot be reached. Check your connection and try again.",
        )
    } else if text.contains("not authorized") || text.contains("authentication dismissed") {
        AppError::new(
            "installation_cancelled",
            "Installation was not authorized. You can approve it again when you are ready.",
        )
    } else if text.contains("authentication agent") {
        AppError::new("authentication_agent", "Your desktop authentication service is unavailable. Sign in to a full desktop session and try again.")
    } else if text.contains("permission denied") {
        AppError::new(
            "permission",
            "Access was denied. Check the folder permissions or approve the installation request.",
        )
    } else if text.contains("busy") {
        AppError::new(
            "busy_mount",
            "The Drive folder is in use. Close files and file manager windows, then try again.",
        )
    } else {
        AppError::new(
            "system_operation",
            "The connection operation failed. Refresh diagnostics or try again.",
        )
    }
}
