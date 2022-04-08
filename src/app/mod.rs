
use async_trait::async_trait;

use crate::Error;

mod memory;

pub use memory::MemoryStore;

pub trait App: AppHandle {}

#[async_trait]
pub trait AppHandle: Send + Sync {
    async fn handle(&self, token: &str, body: Vec<u8>) -> Result<(), Error>;
}
