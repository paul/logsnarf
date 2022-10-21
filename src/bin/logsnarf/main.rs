use clap::Parser;

use logsnarf::{settings::Settings, util};

mod cli;
mod parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    util::setup()?;
    let settings = Settings::new()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file } => parser::Parser::new(settings).parse(file).await?,
        Commands::Server => (),
    };

    util::teardown()?;
    Ok(())
}
