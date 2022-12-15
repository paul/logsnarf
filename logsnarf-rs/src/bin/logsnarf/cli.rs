use clap::{crate_name, Parser, Subcommand};

#[derive(Parser)]
#[command(name = crate_name!(), author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract metrics from a single log file
    Parse {
        /// File to parse
        file: String,
    },

    /// Run a server that continuously parses metrics from input
    Server,
}
