use std::collections::BTreeMap;
use std::ops;
use std::str::FromStr;

use chrono::{DateTime, Utc};

#[allow(non_camel_case_types)]
pub type procid_t = String;

pub type SDParamIDType = String;
pub type SDParamValueType = String;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Container for the `StructuredData` component of a syslog message.
///
/// This is a map from `SD_ID` to pairs of `SD_ParamID`, `SD_ParamValue`
///
/// The spec does not forbid repeated keys. However, for convenience, we *do* forbid repeated keys.
/// That is to say, if you have a message like
///
/// [foo bar="baz" bar="bing"]
///
/// There's no way to retrieve the original "baz" mapping.
pub struct StructuredData {
    elements: BTreeMap<SDParamIDType, SDParamValueType>,
}

impl ops::Deref for StructuredData {
    type Target = BTreeMap<SDParamIDType, SDParamValueType>;
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

#[cfg(feature = "serde-serialize")]
impl Serialize for StructuredData {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.elements.serialize(ser)
    }
}

impl StructuredData {
    pub fn new_empty() -> Self {
        StructuredData {
            elements: BTreeMap::new(),
        }
    }

    /// Insert a new (sd_id, sd_param_id) -> sd_value mapping into the StructuredData
    pub fn insert_tuple<SPI, SPV>(&mut self, sd_param_id: SPI, sd_param_value: SPV) -> ()
    where
        SPI: Into<SDParamIDType>,
        SPV: Into<SDParamValueType>,
    {
        self.elements
            .insert(sd_param_id.into(), sd_param_value.into());
    }

    /// The number of distinct SD_IDs
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether or not this is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogData {
    pub timestamp: DateTime<Utc>,
    pub appname: String,
    pub procid: procid_t,
    pub msg: String,
}

impl FromStr for LogData {
    type Err = crate::parser::ParseErr;

    /// Parse a string into a `LogData`
    ///
    /// Just calls `parser::parse_line`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parser::parse_line(s)
    }
}

impl FromStr for StructuredData {
    type Err = crate::parser::ParseErr;

    /// Parse a LogData msg into a `StructuredData`
    ///
    /// Just calls `parser::parse_msg`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parser::parse_msg(s)
    }
}
