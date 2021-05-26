use clap::{crate_version, FromArgMatches, IntoApp};
use console::style;

use logsnarf_rs::opt::ParseOpt;

#[macro_use]
extern crate log;
use env_logger;

#[async_std::main]
async fn main() {
    env_logger::builder().format_timestamp_micros().init();
    let matches = ParseOpt::into_app().version(crate_version!()).get_matches();

    if let Err(error) = logsnarf_rs::parse(ParseOpt::from_arg_matches(&matches)).await {
        println!("{} {}", style("error:").bold().red(), error);
        std::process::exit(1);
    }
}
