use async_trait::async_trait;

use thiserror::Error;

use crate::{metric::Metric, settings::TsdbCredentials};

pub mod influxdb_v1;

#[derive(Debug, Error)]
pub enum WriterError {
    #[error(transparent)]
    InfluxdbV1Error(#[from] influxdb_v1::InfluxdbV1Error),
}

pub fn build(creds: &TsdbCredentials) -> impl MetricWriter {
    match creds {
        TsdbCredentials::InfluxdbV1(creds) => influxdb_v1::InfluxdbV1::new(&creds),
    }
}

#[async_trait]
pub trait MetricWriter {
    fn write(&mut self, metric: Metric);

    async fn flush(&mut self) -> Result<(), WriterError>;
}
