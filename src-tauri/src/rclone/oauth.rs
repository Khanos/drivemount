use crate::{
    errors::{AppError, Result},
    models::{AuthRequest, AuthStatus, Connection},
    rclone::detection,
    security::validation,
    system::{
        paths::{verify_private_socket, Paths},
        process::{binary, Runner},
    },
};
use serde_json::{json, Value};
use std::{process::Stdio, sync::Arc, time::Duration};
use tokio::{
    process::Command,
    sync::{Mutex, Notify},
};

pub struct Job {
    pub status: Mutex<AuthStatus>,
    pub cancel: Notify,
}
#[derive(Default)]
pub struct OAuthManager {
    pub job: Mutex<Option<Arc<Job>>>,
}
impl OAuthManager {
    pub async fn status(&self) -> Option<AuthStatus> {
        let j = self.job.lock().await.clone();
        match j {
            Some(j) => Some(j.status.lock().await.clone()),
            None => None,
        }
    }
    pub async fn cancel(&self) {
        if let Some(j) = self.job.lock().await.as_ref() {
            j.cancel.notify_one();
        }
    }
    pub async fn start(
        &self,
        paths: Paths,
        runner: Arc<dyn Runner>,
        connection: Connection,
        request: AuthRequest,
        lock: std::fs::File,
    ) -> Result<AuthStatus> {
        let mut current = self.job.lock().await;
        if let Some(j) = current.as_ref() {
            if ["waiting", "completing"].contains(&j.status.lock().await.state.as_str()) {
                return Err(AppError::new(
                    "operation_busy",
                    "Google authorization is already running.",
                ));
            }
        }
        let status = AuthStatus {
            id: connection.id.clone(),
            state: "waiting".into(),
            message: "Waiting for Google authorization in your browser…".into(),
        };
        let job = Arc::new(Job {
            status: Mutex::new(status.clone()),
            cancel: Notify::new(),
        });
        *current = Some(job.clone());
        tokio::spawn(async move {
            let _lock = lock;
            let work = async {
                authorize(&paths, &connection, &request).await?;
                {
                    let mut status = job.status.lock().await;
                    status.state = "completing".into();
                    status.message = "Completing your Google connection…".into();
                }
                detection::verify_remote(
                    runner.as_ref(),
                    &connection.config_path,
                    &connection.remote,
                )
                .await
            };
            let result = tokio::select! {r=tokio::time::timeout(Duration::from_secs(180),work)=>match r{Ok(r)=>r,Err(_)=>Err(AppError::new("oauth_timeout","Google authorization timed out. Try connecting again."))},_ = job.cancel.notified()=>Err(AppError::new("oauth_cancelled","Google account connection was cancelled. You can try again."))};
            let mut status = job.status.lock().await;
            match result {
                Ok(()) => {
                    let saved = (|| {
                        let mut reg = paths.load()?;
                        let c = reg
                            .connections
                            .iter_mut()
                            .find(|c| c.id == connection.id)
                            .ok_or_else(|| {
                                AppError::new(
                                    "connection_missing",
                                    "The saved connection could not be found.",
                                )
                            })?;
                        c.authorized = true;
                        paths.save(&reg)
                    })();
                    match saved {
                        Ok(()) => {
                            status.state = "connected".into();
                            status.message = "Your Google account is connected.".into();
                        }
                        Err(e) => {
                            status.state = "failed".into();
                            status.message = e.message;
                        }
                    }
                }
                Err(e) => {
                    status.state = if e.code == "oauth_cancelled" {
                        "cancelled"
                    } else {
                        "failed"
                    }
                    .into();
                    status.message = e.message;
                }
            }
            let _ = std::fs::remove_file(connection.socket_path.with_extension("oauth.sock"));
        });
        Ok(status)
    }
}
pub fn answer(response: &Value) -> Result<Option<(String, String)>> {
    if response
        .get("Error")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty())
    {
        return Err(AppError::new(
            "oauth_failed",
            "Google authorization failed. Check your OAuth client configuration and try again.",
        ));
    }
    let state = response.get("State").and_then(Value::as_str).unwrap_or("");
    if state.is_empty() {
        return Ok(None);
    }
    let name = response
        .pointer("/Option/Name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let value=match name {"config_is_local"|"config_refresh_token"=>"true","config_change_team_drive"=>"false",_=>return Err(AppError::new("oauth_protocol","This rclone version requested an unsupported authorization step. Update rclone and try again."))};
    Ok(Some((state.into(), value.into())))
}
pub async fn client(socket: &std::path::Path) -> Result<reqwest::Client> {
    private_client(socket, None).await
}
async fn private_client(
    socket: &std::path::Path,
    expected_pid: Option<u32>,
) -> Result<reqwest::Client> {
    verify_private_socket(socket, expected_pid).await?;
    reqwest::Client::builder()
        .unix_socket(socket.to_owned())
        .no_proxy()
        .timeout(Duration::from_secs(170))
        .build()
        .map_err(|_| {
            AppError::new(
                "local_connection",
                "The private authorization connection could not be created.",
            )
        })
}
pub async fn rc(c: &reqwest::Client, endpoint: &str, body: Value) -> Result<Value> {
    let response = c
        .post(format!("http://localhost/{endpoint}"))
        .json(&body)
        .send()
        .await
        .map_err(|_| {
            AppError::new(
                "local_connection",
                "The connection helper could not be reached. Try again.",
            )
        })?;
    // Rclone error bodies echo request parameters. Never return or log them.
    if !response.status().is_success() {
        return Err(AppError::new("oauth_failed","Rclone could not complete this request. Check the Google credentials or renew authorization."));
    }
    response.json().await.map_err(|_| {
        AppError::new(
            "oauth_protocol",
            "The connection helper returned an unreadable response.",
        )
    })
}
async fn authorize(paths: &Paths, c: &Connection, request: &AuthRequest) -> Result<()> {
    let app_client = if request.client_id.is_empty() && !c.authorized {
        super::app_client::load()
    } else {
        None
    };
    let (id, secret) = if let Some(client) = app_client.as_ref() {
        (client.client_id.as_str(), client.client_secret.as_str())
    } else {
        (request.client_id.as_str(), request.client_secret.as_str())
    };
    if !c.authorized {
        validation::oauth(id, secret)?;
    }
    paths.prepare_runtime()?;
    let socket = c.socket_path.with_extension("oauth.sock");
    if socket.as_os_str().len() > 100 {
        return Err(AppError::new(
            "socket_path",
            "Your runtime directory path is too long for a private connection socket.",
        ));
    }
    let mut cmd = Command::new(binary("rclone")?);
    cmd.args(["rcd", "--config"])
        .arg(&c.config_path)
        .args([
            "--rc-addr",
            &format!("unix://{}", socket.display()),
            "--rc-no-auth",
            "--log-level",
            "ERROR",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    unsafe {
        cmd.pre_exec(|| {
            libc::umask(0o077);
            Ok(())
        });
    }
    let mut child = cmd.spawn().map_err(|_| {
        AppError::new(
            "oauth_helper",
            "The Google connection helper could not start.",
        )
    })?;
    for _ in 0..50 {
        if socket.exists() {
            break;
        }
        if child.try_wait()?.is_some() {
            return Err(AppError::new(
                "oauth_helper",
                "The Google connection helper stopped before it was ready.",
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let http = private_client(&socket, child.id()).await?;
    let parameters = json!({"client_id":id,"client_secret":secret,"scope":"drive","config_is_local":"true","config_change_team_drive":"false"});
    let mut response = if c.authorized {
        rc(&http,"config/update",json!({"name":c.remote,"parameters":{"config_refresh_token":"true"},"opt":{"nonInteractive":true}})).await?
    } else {
        rc(&http,"config/create",json!({"name":c.remote,"type":"drive","parameters":parameters,"opt":{"nonInteractive":true,"obscure":true}})).await?
    };
    let mut complete = false;
    for _ in 0..12 {
        let Some((state, result)) = answer(&response)? else {
            complete = true;
            break;
        };
        response=rc(&http,"config/update",json!({"name":c.remote,"parameters":{"config_is_local":"true","config_change_team_drive":"false"},"opt":{"nonInteractive":true,"continue":true,"state":state,"result":result}})).await?;
    }
    if !complete {
        return Err(AppError::new(
            "oauth_protocol",
            "Google authorization could not finish. Update rclone and try again.",
        ));
    }
    child.kill().await?;
    let _ = child.wait().await;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noninteractive_protocol() {
        assert_eq!(
            answer(&json!({"State":"oauth,local","Option":{"Name":"config_is_local"}})).unwrap(),
            Some(("oauth,local".into(), "true".into()))
        );
        assert!(answer(&json!({"State":""})).unwrap().is_none());
        assert!(answer(&json!({"State":"x","Option":{"Name":"client_secret"}})).is_err());
        assert!(answer(&json!({"Error":"sensitive error"})).is_err());
    }
}
