use std::collections::HashMap;

use async_std::io::BufRead;
use async_std::stream::StreamExt;
use async_std::io::prelude::*;
use async_std::sync::{Arc,Mutex};
use async_std::task;
use dynomite::dynamodb::{DynamoDbClient};
use log::{debug,info,warn};
use rusoto_core::Region;
use thiserror::Error;

use crate::{
    credentials,
    credentials::{Credentials, CredentialsError}
};
use crate::settings::Settings;
use crate::parser;
use crate::writer::Writer;
use crate::log_data::{LogData, StructuredData};
use crate::decoder::find_decoders;
use crate::decoder::Metric;

type Token = String;

pub struct Handler {
    credentials_cache: HashMap<Token, Credentials>,
    writers: HashMap<Token, Arc<Mutex<Writer>>>,
    dynamo_db: rusoto_dynamodb::DynamoDbClient,
    settings: Settings,
}

#[derive(Debug, Error)]
pub enum HandlerErr {
    #[error("credentials error")]
    BadCredentials { source: CredentialsError },

    #[error("parse error")]
    ParseError { source: parser::ParseErr },

    #[error("read error")]
    ReadError { source: std::io::Error },
}


impl Handler {
    pub fn new() -> Handler {
        Handler {
            credentials_cache: HashMap::with_capacity(100),
            writers: HashMap::with_capacity(100),
            dynamo_db: DynamoDbClient::new(Region::UsEast2),
            settings: Settings::new().expect("Error getting settings"),
        }
    }

    pub async fn call(&mut self, token: Token, data: impl BufRead + std::marker::Unpin) -> Result<(), HandlerErr> {
        let creds = self.lookup_creds(&token).map_err(|source| HandlerErr::BadCredentials { source })?;
        debug!("{:?}", creds);
        let writer = self.writers.entry(token).or_insert_with(|| Arc::new(Mutex::new(Writer::new(creds))));

        let mut lines = data.lines();
        let metrics: &mut Vec<Metric> = &mut Vec::with_capacity(5);
        while let Some(line) = lines.next().await {
            let real_line = line.map_err(|source| HandlerErr::ReadError {source })?;
            let log_data = real_line.parse::<LogData>().map_err(|source| HandlerErr::ParseError { source })?;
            let decoders = find_decoders(&log_data);
            for decoder in decoders {
                let data = log_data.msg.parse::<StructuredData>().map_err(|source| HandlerErr::ParseError { source })?;
                if let Some(metric) = decoder.try_decode(&log_data, &data) {
                    metrics.push(metric);
                }
            }
        }

        if !metrics.is_empty() {
            task::spawn(writer.lock().await.add_many(metrics));
        }

        Ok(())
    }

    // pub fn finish(&mut self) {
    //     for writer in self.writers.values_mut() {
    //         writer.flush()
    //     }
    // }

    fn lookup_creds(&mut self, token: &Token) -> Result<Credentials, CredentialsError> {
        match self.credentials_cache.get_mut(token) {
            Some(creds) => Ok(creds.clone()),
            None => {
                task::block_on(async {
                    let creds = self.fetch_creds(token).await?;
                    self.credentials_cache.insert(token.clone(), creds.clone());
                    Ok(creds)
                })
            }
        }
    }

    async fn fetch_creds(&self, token: &Token) -> Result<Credentials, CredentialsError> {
        credentials::fetch(&self.dynamo_db, "logsnarf_config".to_string(), token).await
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        warn!("shutting down Handler");
        // self.finish()
    }
}
