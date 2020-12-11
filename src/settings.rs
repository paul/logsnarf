use config::{Config, ConfigError, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub credentials_table: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.set("credentials_table", "logsnarf_config")?;

        s.merge(Environment::with_prefix("logsnarf"))?;

        s.try_into()
    }
}
