use std::collections::BTreeMap;

use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;
use xdg;

use crate::{metric, metric_writer};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Daemon {
    http_port: Option<u16>,
    syslog_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Logging {
    level: String,
    output: String,
}

// #[derive(Debug, Deserialize)]
// #[allow(unused)]
// pub struct Tsdb {
//     #[serde(rename(deserialize = "type"))]
//     pub type_: String,
//     pub url: String,
// }

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum TsdbCredentials {
    InfluxdbV1(metric_writer::influxdb_v1::Credentials),
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
#[serde(untagged)]
pub enum Matcher {
    String(String),
    Condition(BTreeMap<String, String>),
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct MetricDecoder {
    pub name: metric::Name,
    pub tag_names: Vec<metric::TagKey>,
    pub field_names: Vec<metric::FieldKey>,
    pub matcher: BTreeMap<String, Matcher>,
}

pub type MetricDecoders = Vec<MetricDecoder>;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub daemon: Daemon,
    pub logging: Logging,
    pub tsdb: TsdbCredentials,
    pub metrics: MetricDecoders,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // config files ~/.config/logsnarf/logsnarf.toml, /etc/logsnarf/logsnarf.toml, etc...
        let xdg_dirs = xdg::BaseDirectories::with_prefix("logsnarf").unwrap();

        let mut builder = Config::builder()
            // defaults
            .set_default("logging.level", "info")?
            .set_default("logging.output", "STDOUT")?;

        builder = builder.add_source(File::with_name("logsnarf"));

        while let Some(config_file_path) = xdg_dirs.find_config_files("logsnarf.toml").next() {
            builder = builder
                .add_source(File::with_name(config_file_path.to_str().unwrap()).required(false))
        }
        let s = builder
            .add_source(Environment::with_prefix("logsnarf"))
            .build()?;

        s.try_deserialize()
    }
}
