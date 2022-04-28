use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::instrument;

use crate::metric::Metric;
use crate::parser::{self, KVPairs, LogData};

pub struct Decoder<'a> {
    matcher: &'a dyn Fn(&LogData) -> bool,
    pub name: &'a str,
    pub tag_names: &'a [&'a str],
    pub field_names: &'a [&'a str],
}

const HEROKU: &str = "heroku";
const HEROKU_ROUTER: &str = "router";
const HEROKU_POSTGRES: &str = "heroku-postgres";
const HEROKU_REDIS: &str = "heroku-redis";

const HEROKU_LOAD_AVG: &str = "sample#load_avg_1m";
const HEROKU_MEMORY_TOTAL: &str = "sample#memory_total";

const DYNO_LOAD_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.appname == HEROKU && ld.msg.contains(HEROKU_LOAD_AVG) },
    name: "heroku_dyno_load",
    tag_names: &["source"],
    field_names: &[
        "sample#load_avg_1m",
        "sample#load_avg_5m",
        "sample#load_avg_15m",
    ],
};

const DYNO_MEMORY_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.appname == HEROKU && ld.msg.contains(HEROKU_MEMORY_TOTAL) },
    name: "heroku_dyno_memory",
    tag_names: &["source"],
    field_names: &[
        "sample#memory_total",
        "sample#memory_rss",
        "sample#memory_cache",
        "sample#memory_swap",
        "sample#memory_pgpgin",
        "sample#memory_pgpgout",
        "sample#memory_quota",
    ],
};

const ROUTER_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.appname == HEROKU && ld.procid == HEROKU_ROUTER },
    name: "heroku_router",
    tag_names: &["method", "host", "dyno", "status", "protocol"],
    field_names: &["connect", "service", "bytes"],
};

const POSTGRES_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.procid == HEROKU_POSTGRES },
    name: "heroku_postgres",
    tag_names: &["addon", "source"],
    field_names: &[
        "sample#db_size",
        "sample#tables",
        "sample#active-connections",
        "sample#waiting-connections",
        "sample#index-cache-hit-rate",
        "sample#table-cache-hit-rate",
        "sample#load-avg-1m",
        "sample#load-avg-5m",
        "sample#load-avg-15m",
        "sample#read-iops",
        "sample#write-iops",
        "sample#memory-total",
        "sample#memory-free",
        "sample#memory-cached",
        "sample#memory-postgres",
    ],
};

const REDIS_DECODER: Decoder<'static> = Decoder {
    matcher: &{ |ld: &LogData| ld.procid == HEROKU_REDIS },
    name: "heroku_redis",
    tag_names: &["addon"],
    field_names: &[
        "sample#active-connections",
        "sample#load-avg-1m",
        "sample#load-avg-5m",
        "sample#load-avg-15m",
        "sample#read-iops",
        "sample#write-iops",
        "sample#memory-total",
        "sample#memory-free",
        "sample#memory-cached",
        "sample#memory-redis",
        "sample#hit-rate",
        "sample#evicted-keys",
    ],
};

const DECODERS: &'static [Decoder] = &[
    DYNO_LOAD_DECODER,
    DYNO_MEMORY_DECODER,
    POSTGRES_DECODER,
    REDIS_DECODER,
    ROUTER_DECODER,
];

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

#[instrument(level = "info")]
pub fn decode(line: String) -> Result<Option<Metric>, DecodeError> {
    match parse(line)? {
        Some(ld) => match extract(&ld)? {
            Some(metric) => Ok(Some(metric)),
            None => Ok(None),
        },
        None => Ok(None),
    }
}

#[instrument(level = "info")]
fn parse(line: String) -> Result<Option<LogData>, DecodeError> {
    parser::parse_line(line).map_err(DecodeError::ParseError)
}

#[instrument(level = "info")]
fn extract(ld: &LogData) -> Result<Option<Metric>, DecodeError> {
    if let Some(decoder) = find_decoder(ld) {
        let pairs = parser::parse_msg(&ld.msg)?;
        Ok(Some(Metric::new(
            parse_timestamp(ld)?,
            decoder.name.to_string(),
            extract_keys(decoder.tag_names, &pairs)?,
            extract_keys(decoder.field_names, &pairs)?,
        )))
    } else {
        Ok(None)
    }
}

fn find_decoder(ld: &LogData) -> Option<&Decoder<'_>> {
    for decoder in DECODERS {
        if (decoder.matcher)(ld) {
            return Some(decoder);
        }
    }
    None
}

fn parse_timestamp(ld: &LogData) -> Result<DateTime<Utc>, DecodeError> {
    Ok(DateTime::parse_from_rfc3339(ld.timestamp_str.as_ref())
        .map_err(DecodeError::TimestampParseError)?
        .with_timezone(&Utc))
}

fn extract_keys(keys: &[&str], pairs: &KVPairs) -> Result<KVPairs, DecodeError> {
    let mut out = KVPairs::new();
    for key in keys {
        let key_s = String::from(*key);

        if let Some(val) = pairs.get(&key_s) {
            out.insert(key_s, val.clone());
        }
    }
    Ok(out)
}
