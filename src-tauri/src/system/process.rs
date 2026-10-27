use crate::errors::{classify, AppError, Result};
use async_trait::async_trait;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Command};
#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub timeout: Duration,
}
impl CommandSpec {
    pub fn new(
        program: impl Into<PathBuf>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            timeout: Duration::from_secs(20),
        }
    }
    pub fn seconds(mut self, n: u64) -> Self {
        self.timeout = Duration::from_secs(n);
        self
    }
}
#[derive(Debug, Clone)]
pub struct Output {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}
impl Output {
    pub fn checked(self) -> Result<Self> {
        if self.success {
            Ok(self)
        } else {
            Err(classify(&self.stderr))
        }
    }
}
#[async_trait]
pub trait Runner: Send + Sync {
    async fn run(&self, spec: CommandSpec) -> Result<Output>;
}
pub struct SystemRunner;
#[async_trait]
impl Runner for SystemRunner {
    async fn run(&self, s: CommandSpec) -> Result<Output> {
        let mut child = Command::new(&s.program)
            .args(&s.args)
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| {
                AppError::new(
                    "missing_component",
                    "A required system component could not be started. Run the system check again.",
                )
            })?;
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let work = async {
            // Both pipe types implement AsyncRead, but are distinct concrete types.
            let out = async {
                let mut b = Vec::new();
                stdout
                    .take(1024 * 1024)
                    .read_to_end(&mut b)
                    .await
                    .map(|_| b)
            };
            // stderr is drained concurrently to avoid pipe deadlocks.
            let err = async {
                let mut b = Vec::new();
                stderr
                    .take(1024 * 1024)
                    .read_to_end(&mut b)
                    .await
                    .map(|_| b)
            };
            let (out, err, status) = tokio::join!(out, err, child.wait());
            Ok::<_, AppError>(Output {
                success: status?.success(),
                stdout: String::from_utf8_lossy(&out?).into(),
                stderr: String::from_utf8_lossy(&err?).into(),
            })
        };
        match tokio::time::timeout(s.timeout, work).await {
            Ok(r) => r,
            Err(_) => {
                let _ = child.kill().await;
                Err(AppError::new(
                    "timeout",
                    "This operation took too long. Check your connection and try again.",
                ))
            }
        }
    }
}
pub fn binary(name: &str) -> Result<PathBuf> {
    which::which(name).map_err(|_| {
        AppError::new(
            "missing_component",
            format!("The {name} component is not installed."),
        )
    })
}
#[cfg(test)]
pub mod fake {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    #[derive(Default)]
    pub struct FakeRunner {
        pub responses: Mutex<VecDeque<Result<Output>>>,
        pub calls: Mutex<Vec<CommandSpec>>,
    }
    impl FakeRunner {
        pub fn push(&self, success: bool, text: &str) {
            self.responses.lock().unwrap().push_back(Ok(Output {
                success,
                stdout: text.into(),
                stderr: if success { String::new() } else { text.into() },
            }));
        }
    }
    #[async_trait]
    impl Runner for FakeRunner {
        async fn run(&self, s: CommandSpec) -> Result<Output> {
            self.calls.lock().unwrap().push(s);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected system command")
        }
    }
}
