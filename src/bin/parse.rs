// use async_channel;
use clap;

use anyhow::{Context, Result};
use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::task;
use tokio::runtime::Runtime;

#[macro_use]
extern crate log;
use env_logger;

use logsnarf_rs::handler::Handler;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_micros().init();
    let config = clap::App::new("logsnarf-parser")
        .version("1.0")
        .args_from_usage(
            "-t, --token=<TOKEN> 'Config token'
             -f, --file=<FILE> 'Log file to parse'",
        )
        .get_matches();

    let filename = config.value_of("file").expect("missing filename arg");
    let token = config.value_of("token").expect("missing token arg").to_string();
    let mut handler = Handler::new();

    async {
        let file = File::open(filename).await
            .with_context(|| format!("Failed to read file!"))?;
        let reader = BufReader::new(file);

        handler.call(token, reader).await
            .with_context(|| format!("Failed to parse data!"))?;

        Ok(())
    }.await
}
