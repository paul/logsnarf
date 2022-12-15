
use anyhow::anyhow;
use dotenv::dotenv;
use std::env;

use sqlx::postgres::PgPoolOptions;
// use sqlx::any::Any;
use sqlx::Connection;
use sqlx::postgres::PgConnection;
use async_std::fs::File;
use async_std::io::BufReader;
// use async_std::prelude::*;
// use async_std::task;

pub mod opt;

// extern crate serde;

// #[macro_use]
// extern crate serde_derive;

pub mod adapter;
// pub mod app;
pub mod credentials;
pub mod decoder;
// pub mod handler;
// pub mod influxdb_v1_adapter;
pub mod log_data;
pub mod parser;
pub mod extractor;
pub mod metric;
// pub mod settings;
// pub mod writer;
// pub mod writer_store;

// pub use decoder::find_decoders;
// pub use log_data::LogData;
// pub use parser::{parse_line, parse_msg, ParseErr};

pub use crate::opt::{ParseOpt,ServerOpt};
// pub use crate::handler::handle;
// pub use crate::influxdb_v1_adapter::InfluxDbV1Adapter;

pub async fn parse(opt: ParseOpt) -> anyhow::Result<()> {
    dotenv().ok();

    let tsdb_url = opt.tsdb_url.expect("Missing tsdb_url arg");
    let file = File::open(opt.file.expect("Missing file arg")).await?;
    let data = BufReader::new(file);

    // let pg = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect_lazy(&database_url).await?;
    // let pg = PgConnection::connect(&database_url).await?;

    // let tsdb = InfluxDbV1Adapter::connect(tsdb_url).await?;

    Ok(())
    // Ok(handle(data, tsdb).await?)
}

// pub async fn serve(opt: ServerOpt) -> anyhow::Result<()> {
//     use log::LevelFilter;
//     use sqlx::{Acquire, ConnectOptions}; // Or sqlx::prelude::*;
//     use sqlx::postgres::{PgConnectOptions, PgPoolOptions, Postgres};

//     use tide_sqlx::SQLxMiddleware;
//     use tide_sqlx::SQLxRequestExt;

//     dotenv().ok();

//     let database_url = match opt.database_url {
//         Some(db_url) => db_url,
//         None => env::var("DATABASE_URL")
//             .map_err(|_| anyhow!("DATABASE_URL env var or --database_url flag must be set"))?,
//     };

//     // let mut connect_opts = PgConnectOptions::new();
//     // connect_opts.log_statements(LevelFilter::Debug);

//     // let pg_pool = PgPoolOptions::new()
//     //     .max_connections(5)
//     //     .connect_with(connect_opts)
//     //     .await?;
//     let pg = PgPoolOptions::new()
//         .max_connections(5)
//         .connect_lazy(&database_url).await?;

//     let mut app = tide::new();
//     app.with(SQLxMiddleware::<Postgres>::new(pg));

//     app.at("/").post(|req: tide::Request<()>| async move {
//         let mut pg_conn = req.sqlx_conn::<Postgres>().await;

//         sqlx::query("SELECT * FROM users")
//             .fetch_optional(pg_conn.acquire().await?)
//             .await;

//         Ok("")
//     });

//     Ok(())
// }
