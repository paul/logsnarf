use std::io::{BufRead, BufReader};
use std::sync::Arc;

use env_logger;

// use logsnarf_rs::handler::Handler;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    use log::LevelFilter;
    use sqlx::{Acquire, ConnectOptions}; // Or sqlx::prelude::*;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions, Postgres};

    use tide_sqlx::SQLxMiddleware;
    use tide_sqlx::SQLxRequestExt;

    let mut connect_opts = PgConnectOptions::new();
    connect_opts.log_statements(LevelFilter::Debug);

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await?;

    let mut app = tide::new();
    // app.with(SQLxMiddleware::from(pg_pool));

    // app.at("/").post(|req: tide::Request<()>| async move {
    //     let mut pg_conn = req.sqlx_conn::<Postgres>().await;

    //     sqlx::query("SELECT * FROM users")
    //         .fetch_optional(pg_conn.acquire().await?)
    //         .await;

    //     Ok("")
    // });
    Ok(())
}


// fn handle_body(handler: &mut Handler, mut body: impl Buf) {
//     // while body.has_remaining() {
//     //     let reader = BufReader::new(body.bytes());
//     //     for line in reader.lines() {
//     //         if let Ok(l) = line {
//     //             handler.handle(l);
//     //         }
//     //     }
//     //     let cnt = body.bytes().len();
//     //     body.advance(cnt);
//     // }
// }

