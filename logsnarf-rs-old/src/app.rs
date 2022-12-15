use async_std::io::BufRead;

use async_std::io::BufReader;
use async_std::stream::StreamExt;
use async_std::io::prelude::*;
use thiserror::Error;

use crate::decoder::find_decoders;
use crate::log_data::{LogData, StructuredData};
use crate::settings::Settings;
use crate::credentials;
use crate::parser;
use crate::writer_store::WriterStore;

pub struct App<'a> {
    settings: &'a Settings,
    writers: &'a mut WriterStore<'a>,
}

#[derive(Debug, Error)]
pub enum AppErr {
    #[error("credentials error")]
    BadCredentials { source: credentials::CredentialsError },

    #[error("parse error")]
    ParseError { source: parser::ParseErr },

    #[error("read error")]
    ReadError { source: std::io::Error },
}

impl<'a> App<'_> {
    pub fn new() -> App<'a> {
        let settings = Settings::new().unwrap();
        let writers = &mut WriterStore::new(&settings);
        App {
            settings: &settings,
            writers,
        }
    }

    pub async fn handle(&mut self, token: String, body: impl BufRead + std::marker::Unpin) -> Result<(), AppErr> {
        let mut writer = self.writers.get(&token).map_err(|source| AppErr::BadCredentials { source })?;

        let mut lines = body.lines();
        while let Some(line) = lines.next().await {
            let real_line = line.map_err(|source| AppErr::ReadError {source })?;
            let log_data = real_line.parse::<LogData>().map_err(|source| AppErr::ParseError { source })?;
            let decoders = find_decoders(&log_data);
            for decoder in decoders {
                let data = log_data.msg.parse::<StructuredData>().map_err(|source| AppErr::ParseError { source })?;
                if let Some(metric) = decoder.try_decode(&log_data, &data) {
                    writer.add(metric);
                }
            }
        }
        Ok(())
    }

    pub fn exit(&mut self) {
        self.writers.flush_all();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

