use logsnarf::{error::Result, settings::Settings};

pub struct Parser {
    settings: Settings,
}

impl Parser {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
    pub fn parse(&self, filename: String) -> Result<()> {
        Ok(())
    }
}
