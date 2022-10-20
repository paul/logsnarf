use std::collections::BTreeMap;
// use std::convert::Infallible;
// use std::fmt::{Display, Formatter};
// use std::str::FromStr;

use chrono::{DateTime, Utc};

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
