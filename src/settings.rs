use std::collections::BTreeMap;

use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;
use xdg;

use crate::metric;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Daemon {
    http_port: Option<u16>,
    syslog_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Logging {
    level: String,
    output: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Tsdb {
    #[serde(rename(deserialize = "type"))]
    type_: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
#[serde(untagged)]
enum Matcher {
    String(String),
    List(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Rule {
    name: metric::Name,
    tag_names: Vec<metric::TagKey>,
    field_names: Vec<metric::FieldKey>,
    matcher: BTreeMap<String, Matcher>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    daemon: Daemon,
    logging: Logging,
    tsdb: Tsdb,
    rules: Vec<Rule>,
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
