use std::vec::Vec;
use std::future::Future;

use log::{debug,info};

use crate::adapter::Adapter;
use crate::credentials::{Credentials, Secrets};
use crate::decoder::Metric;
use crate::influxdb_v1_adapter::InfluxDbV1Adapter;

pub struct Writer {
    credentials: Credentials,
    adapter: Box<dyn Adapter>,
    buffer: Vec<Metric>,
}

impl Writer {
    pub fn new(credentials: Credentials) -> Writer {
        let adapter = match credentials.credentials {
            Secrets::InfluxDbCredentials(ref creds) => InfluxDbV1Adapter::new(creds),
            // _ => panic!("no adapter!"),
        };
        Writer {
            credentials: credentials.clone(),
            adapter: Box::new(adapter),
            buffer: Vec::with_capacity(1000),
        }
    }

    pub async fn add(&mut self, metric: Metric) -> impl Future + '_ {
        self.buffer.push(metric);

        self.maybe_flush()
    }

    pub async fn add_many(&mut self, metrics: &mut Vec<Metric>) -> impl Future + '_ {
        debug!("added {} metrics", metrics.len());
        self.buffer.append(metrics);

        self.maybe_flush()
    }

    async fn maybe_flush(&mut self) {
        println!("maybe_flush");
        let l = self.buffer.len();
        debug!("buffer: {}", l);
        if l >= 100 {
            info!("buffer full; flushing");
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        info!("flushing buffer: {} points", self.buffer.len());
        if !self.buffer.is_empty() {
            self.adapter.write(self.buffer.clone());
            self.buffer.clear();
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        info!(target: &self.credentials.name.clone(), "[{}] Shutting down...", self.credentials.name);
        self.flush()
    }
}
