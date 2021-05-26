use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::log_data::StructuredData;

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

pub type Tags = HashMap<TagKey, TagValue>;
pub type Fields = HashMap<FieldKey, FieldValue>;

#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub tags: Tags,
    pub fields: Fields,
}

#[derive(Debug, Error)]
pub enum ExtractErr {
    #[error("Key `{0}` is not found")]
    MissingKey(String),
}

pub fn extract_tags(names: &[&str], sd: &StructuredData) -> Result<Tags, ExtractErr> {
    let mut tags = Tags::new();
    for key in names {
        let key_s = String::from(*key);
        let val = sd
            .get(&key_s)
            .ok_or(ExtractErr::MissingKey(key_s.clone()))?;
        tags.insert(key_s.clone(), val.clone());
    }
    Ok(tags)
}

pub fn extract_fields(names: &[&str], sd: &StructuredData) -> Result<Fields, ExtractErr> {
    let mut fields = Fields::new();
    for key in names {
        let key_s = String::from(*key);
        let val = sd
            .get(&key_s)
            .ok_or(ExtractErr::MissingKey(key_s.clone()))?;
        let (k, v) = extract_unit(key_s, val);
        fields.insert(k.clone(), v.clone());
    }
    Ok(fields)
}

fn extract_unit(key: String, val: &str) -> (String, FieldValue) {
    (key, val.parse::<FieldValue>().unwrap())
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

#[cfg(test)]
mod tests {

    use super::*;

    use chrono::Utc;

    use std::collections::HashMap;
    macro_rules! collection {
        // map-like
        ($($k:expr => $v:expr),* $(,)?) => {
            std::iter::Iterator::collect(std::array::IntoIter::new([$(($k, $v),)*]))
        };
        // set-like
        ($($v:expr),* $(,)?) => {
            std::iter::Iterator::collect(std::array::IntoIter::new([$($v,)*]))
        };
    }

    #[test]
    fn test_decode_field_value_plain_int() {
        let r = "42".parse::<FieldValue>().unwrap();
        assert_eq!(r, FieldValue::Integer(42, None));
    }

    #[test]
    fn test_decode_field_value_int_unit() {
        let r = "7widgets".parse::<FieldValue>().unwrap();
        assert_eq!(r, FieldValue::Integer(7, Some("widgets".to_string())));
    }

    #[test]
    fn test_decode_field_value_plain_float() {
        let r = "42.0".parse::<FieldValue>().unwrap();
        assert_eq!(r, FieldValue::Float(42.0, None));
    }

    #[test]
    fn test_decode_field_value_float_unit() {
        let r = "7.2widgets".parse::<FieldValue>().unwrap();
        assert_eq!(r, FieldValue::Float(7.2, Some("widgets".to_string())));
    }
}
