
use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use super::{App, AppHandle};
use crate::Error;

#[derive(Default)]
pub struct MemoryStore {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Default::default()
    }
}

impl App for MemoryStore {}


#[async_trait]
impl AppHandle for MemoryStore {
    async fn handle(&self, token: &str, bytes: Vec<u8>) -> Result<(), Error> {
        self.data
            .write()
            .unwrap()
            .insert(token.to_string().clone(), bytes.clone());
        Ok(())
    }
}
