use tokio::fs::File;
use tokio::io::BufReader;

use tracing::instrument;

use logsnarf::{app::App, error::Result, settings::Settings};

pub struct Parser {
    app: App,
}

impl Parser {
    pub fn new(settings: Settings) -> Self {
        let app = App::new(settings);
        Self { app }
    }

    #[instrument(name = "Parser::parse", skip(self))]
    pub async fn parse(&self, filename: String) -> Result<()> {
        let file = File::open(&filename).await?;
        let data = BufReader::new(file);

        self.app.extract(data).await
    }
}
