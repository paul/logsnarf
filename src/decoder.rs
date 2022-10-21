use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::instrument;

use crate::metric::Metric;
use crate::{
    parser::{self, KVPairs, LogData},
    settings::{Matcher, MetricDecoder, MetricDecoders},
};

#[derive(Debug)]
pub struct Decoder {
    metric_decoder: MetricDecoder,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error(transparent)]
    ParseError(#[from] parser::ParseError),

    #[error(transparent)]
    TimestampParseError(#[from] chrono::ParseError),

    #[error("Tag Key `{0}` was not found in {1:?}")]
    MissingTagKey(String, KVPairs),

    #[error("Field Key `{0}` was not found in {1:?}")]
    MissingFieldKey(String, KVPairs),
}

impl Decoder {
    pub fn new(metric_decoder: MetricDecoder) -> Self {
        Self { metric_decoder }
    }

    pub fn matches(&self, log_data: &LogData) -> bool {
        self.metric_decoder
            .matcher
            .iter()
            .all(|(attr, condition)| match attr.as_str() {
                "hostname" => match_value(&log_data.hostname, &condition),
                "appname" => match_value(&log_data.appname, &condition),
                "procid" => match_value(&log_data.procid, &condition),
                "msgid" => match_value(&log_data.msgid.as_ref().unwrap(), &condition),
                "msg" => match_value(&log_data.msg, &condition),
                _ => panic!("unknown field {}", attr),
            })
    }

    pub fn decode(&self, log_data: &LogData) -> Result<Option<Metric>, DecodeError> {
        let pairs = parser::parse_msg(&log_data.msg)?;
        Ok(Some(Metric::new(
            parse_timestamp(log_data)?,
            self.metric_decoder.name.to_string(),
            extract_keys(&self.metric_decoder.tag_names, &pairs)?,
            extract_keys(&self.metric_decoder.field_names, &pairs)?,
        )))
    }
}

#[instrument(name = "configure_decoders", level = "trace")]
pub fn build_decoders(metric_decoders: &MetricDecoders) -> Vec<Decoder> {
    metric_decoders
        .iter()
        .map(|dec| Decoder::new(dec.clone()))
        .collect()
}

fn match_value(data_value: &String, matcher_value: &Matcher) -> bool {
    match matcher_value {
        Matcher::String(str) => str == data_value,
        Matcher::Condition(conditions) => {
            conditions.iter().any(|(rule, value)| match rule.as_str() {
                "contains" => data_value.contains(value),
                _ => panic!("unknown condition {}", rule),
            })
        }
    }
}

fn parse_timestamp(ld: &LogData) -> Result<DateTime<Utc>, DecodeError> {
    Ok(DateTime::parse_from_rfc3339(ld.timestamp_str.as_ref())
        .map_err(DecodeError::TimestampParseError)?
        .with_timezone(&Utc))
}

fn extract_keys(keys: &Vec<String>, pairs: &KVPairs) -> Result<KVPairs, DecodeError> {
    let mut out = KVPairs::new();
    for key in keys {
        if let Some(val) = pairs.get(key) {
            out.insert(key.clone(), val.clone());
        }
    }
    Ok(out)
}
