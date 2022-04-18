use std::collections::HashMap;
use std::sync::RwLock;

use tracing::info;

// use crate::Result;
use crate::metric::Metric;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Token = String;

#[derive(Debug, Default)]
pub struct MetricStore {
    data: RwLock<HashMap<Token, Vec<Metric>>>,
}

impl MetricStore {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, token: Token, metrics: Vec<Metric>) -> Result<(), E> {
        let mut data = self.data.write().unwrap();
        let entry = data.entry(token.to_owned()).or_default();

        info!("Wrote: {} {:?}", token, metrics);

        entry.extend(metrics);

        Ok(())
    }

    pub fn flush_all(&mut self) -> Result<(), E> {
        info!("Flushing: {:?}", self.data);
        self.data.write().unwrap().clear();
        Ok(())
    }
}
