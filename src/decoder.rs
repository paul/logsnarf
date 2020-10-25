use std::collections::BTreeMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use thiserror::Error;
use time;

use crate::log_data::{LogData, StructuredData};

pub type TagKey = String;
pub type TagValue = String;
pub type FieldKey = String;

#[derive(Clone, Debug)]
pub enum FieldValue {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    Text(String),
}

macro_rules! from_impl {
        ( $variant:ident => $( $typ:ident ),+ ) => (
                $(
                    impl From<$typ> for FieldValue {
                        fn from(b: $typ) -> Self {
                            FieldValue::$variant(b.into())
                        }
                    }
                )+
        )
}
from_impl! {Boolean => bool}
from_impl! {Float => f32, f64}
from_impl! {Integer => i8, i16, i32, i64}
from_impl! {Text => String}
impl From<&str> for FieldValue {
    fn from(b: &str) -> Self {
        FieldValue::Text(b.into())
    }
}
impl<T> From<&T> for FieldValue
where
    T: Copy + Into<FieldValue>,
{
    fn from(t: &T) -> Self {
        (*t).into()
    }
}

#[derive(Debug, Error)]
pub enum ParseFieldValueErr {
    #[error("failed to parse string into a field")]
    FieldParseErr,
}

impl FromStr for FieldValue {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use FieldValue::*;
        s.parse()
            .map(Boolean)
            .or_else(|_| s.parse().map(Float))
            .or_else(|_| s.parse().map(Integer))
            .or_else(|_| Ok(Text(s.into())))
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use FieldValue::*;

        match self {
            Boolean(x) => write!(f, "{}", x),
            Float(x) => write!(f, "{}", x),
            Integer(x) => write!(f, "{}", x),
            Text(text) => write!(f, "{text}", text = text),
        }
    }
}

pub type Tags = BTreeMap<TagKey, TagValue>;
pub type Fields = BTreeMap<FieldKey, FieldValue>;

#[derive(Clone, Debug)]
pub struct Metric {
    pub timestamp: time::Timespec,
    pub name: String,
    pub tags: Tags,
    pub fields: Fields,
}

#[derive(Clone, Copy, Debug)]
pub struct Decoder<'a> {
    match_field: (&'a str, &'a str),
    name: &'a str,
    tag_names: &'a [&'a str],
    field_names: &'a [&'a str],
}

impl Decoder<'_> {
    pub fn try_decode(self, log_data: &LogData) -> Option<Metric> {
        if self.check(log_data) {
            match log_data.msg.parse::<StructuredData>() {
                Ok(data) => self.extract_tags(&data).map_or(None, |tags| {
                    self.extract_fields(&data).map_or(None, |fields| {
                        Some(Metric {
                            timestamp: log_data.timestamp,
                            name: String::from(self.name),
                            tags: tags,
                            fields: fields,
                        })
                    })
                }),
                Err(err) => {
                    println!("Failed to parse data: {:?}", err);
                    None
                }
            }
        } else {
            None
        }
    }

    fn check(self, data: &LogData) -> bool {
        let (k, v) = self.match_field;
        match k {
            "appname" => data.appname == v,
            "procid" => data.procid == v,
            _ => panic!("can't get here"),
        }
    }

    fn extract_tags(self, data: &StructuredData) -> Option<Tags> {
        let mut tags = Tags::new();
        for key in self.tag_names {
            let key_s = String::from(*key);
            match data.get(&key_s) {
                Some(val) => tags.insert(key_s.clone(), val.clone()),
                None => return None,
            };
        }
        Some(tags)
    }

    fn extract_fields(self, data: &StructuredData) -> Option<Fields> {
        let mut fields = Fields::new();
        for key in self.field_names {
            let key_s = String::from(*key);
            match data.get(&key_s) {
                Some(val) => {
                    let (key_s, value) = self.extract_unit(key_s, val);
                    fields.insert(key_s.clone(), value.clone())
                }
                None => return None,
            };
        }
        Some(fields)
    }

    fn extract_unit(self, key: String, val: &str) -> (String, FieldValue) {
        (key, val.parse::<FieldValue>().unwrap())
    }
}

const DYNO_MEMORY_DECODER: Decoder<'static> = Decoder {
    match_field: ("appname", "heroku"),
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

const DYNO_LOAD_DECODER: Decoder<'static> = Decoder {
    match_field: ("appname", "heroku"),
    name: "heroku_dyno_load",
    tag_names: &["source"],
    field_names: &[
        "sample#load_avg_1m",
        "sample#load_avg_5m",
        "sample#load_avg_15m",
    ],
};

const POSTGRES_DECODER: Decoder<'static> = Decoder {
    match_field: ("procid", "heroku-postgres"),
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
    match_field: ("procid", "heroku-redis"),
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
    DYNO_MEMORY_DECODER,
    DYNO_LOAD_DECODER,
    POSTGRES_DECODER,
    REDIS_DECODER,
];

pub fn decode(data: &LogData) -> Option<Metric> {
    DECODERS.iter().find_map(|dec| dec.try_decode(data))
}
