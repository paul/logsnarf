use tracing::debug;

use clap::Parser;

use logsnarf::{settings::Settings, util};

mod cli;
mod parser;

use cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    util::setup()?;
    let settings = Settings::new()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file } => parser::Parser::new(settings).parse(file),
        Commands::Server => Ok(()),
    };

    util::teardown()?;
    Ok(())
}
