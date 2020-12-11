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
use crate::writer::Writer;

pub struct App {
    settings: Settings,
    credentials: credentials::Store
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

impl App {
    pub fn new() -> App {
        let settings = Settings::new().unwrap();
        App {
            settings: settings.clone(),
            credentials: credentials::Store::new(settings.clone()),
        }
    }

    pub async fn handle(&self, token: String, body: impl BufRead + std::marker::Unpin) -> Result<(), AppErr> {
        // find credentials from token
        // decode lines into metrics
        // push credentials + metrics to writer
        // let creds = self.credentials.fetch(&token).map_err(|source| AppErr::BadCredentials { source })?.clone();
        let mut writer = Writer::new();
        let mut lines = body.lines();
        while let Some(line) = lines.next().await {
            let real_line = line.map_err(|source| AppErr::ReadError {source })?;
            let log_data = real_line.parse::<LogData>().map_err(|source| AppErr::ParseError { source })?;
            let decoders = find_decoders(&log_data);
            for decoder in decoders {
                let data = log_data.msg.parse::<StructuredData>().map_err(|source| AppErr::ParseError { source })?;
                if let Some(metric) = decoder.try_decode(&log_data, &data) {
                    writer.write(metric);
                }
            }
        }
        writer.flush();
        Ok(())
    }

    pub fn exit(&mut self) {
        // self.writer.flush();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

