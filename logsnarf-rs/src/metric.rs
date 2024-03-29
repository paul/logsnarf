use std::collections::BTreeMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::parser::KVPairs;

pub type Name = String;
pub type TagKey = String;
pub type TagValue = String;
pub type FieldKey = String;

#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    Boolean(bool),
    Float(f64, Option<String>),
    Integer(i64, Option<String>),
    Text(String),
}

pub type Tags = BTreeMap<TagKey, TagValue>;
pub type Fields = BTreeMap<FieldKey, FieldValue>;

#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub tags: Tags,
    pub fields: Fields,
}

impl Default for Metric {
    fn default() -> Self {
        let timestamp = Utc::now();
        let mut tags = Tags::new();
        tags.insert("my-tag".into(), "value".into());
        let mut fields = Fields::new();
        fields.insert("my-field".into(), FieldValue::Float(0.0, None));

        Self {
            timestamp,
            name: "test-metric".into(),
            tags,
            fields,
        }
    }
}
impl Metric {
    pub fn new(timestamp: DateTime<Utc>, name: String, tags: KVPairs, fields: KVPairs) -> Self {
        Metric {
            timestamp,
            name,
            tags: to_tags(tags),
            fields: to_fields(fields),
        }
    }
}

fn to_tags(t: KVPairs) -> Tags {
    t.clone()
}
fn to_fields(f: KVPairs) -> Fields {
    let mut fields = Fields::new();
    for (k, v) in f {
        let key = k.replace("sample#", "");
        let val = extract_unit(v);
        fields.insert(key, val);
    }
    fields
}

impl FromStr for FieldValue {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use FieldValue::*;
        s.parse()
            .map(Boolean)
            .or_else(|_| parse_int_unit(s).map(|(v, u)| Integer(v, u)))
            .or_else(|_| parse_float_unit(s).map(|(v, u)| Float(v, u)))
            .or_else(|_| Ok(FieldValue::Text(s.into())))
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use FieldValue::*;

        match self {
            Boolean(x) => write!(f, "{}", x),
            Float(v, u) => write!(f, "{} {:?}", v, u),
            Integer(v, u) => write!(f, "{} {:?}", v, u),
            Text(text) => write!(f, "{text}", text = text),
        }
    }
}

fn extract_unit(val: String) -> FieldValue {
    val.parse::<FieldValue>().unwrap()
}

fn parse_float_unit(s: &str) -> Result<(f64, Option<String>), std::num::ParseFloatError> {
    if let Some(i) = s.find(char::is_alphabetic) {
        let (v, u) = s.split_at(i);
        Ok((v.parse::<f64>()?, Some(u.to_string())))
    } else {
        Ok((s.parse::<f64>()?, None))
    }
}

fn parse_int_unit(s: &str) -> Result<(i64, Option<String>), std::num::ParseIntError> {
    if let Some(i) = s.find(char::is_alphabetic) {
        let (v, u) = s.split_at(i);
        Ok((v.parse::<i64>()?, Some(u.to_string())))
    } else {
        Ok((s.parse::<i64>()?, None))
    }
}
