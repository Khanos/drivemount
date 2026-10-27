use crate::{
    errors::Result,
    rclone::mount::{self, Mount},
    system::dependencies,
};
use async_trait::async_trait;
#[async_trait]
pub trait Host: Send + Sync {
    fn mounts(&self) -> Result<Vec<Mount>>;
    async fn online(&self) -> bool;
}
pub struct RealHost;
#[async_trait]
impl Host for RealHost {
    fn mounts(&self) -> Result<Vec<Mount>> {
        mount::mounts()
    }
    async fn online(&self) -> bool {
        dependencies::online().await
    }
}
