use std::collections::HashMap;
use std::sync::RwLock;

use tracing::info;

use lambda_http::{
    service_fn, Request,
    Response,
    http::StatusCode};

use tokio::signal::unix::{signal, SignalKind};

use logsnarf::utils;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

type Token = String;
type Entry = Vec<u8>;

#[derive(Default)]
struct Buffer {
    pub data: RwLock<HashMap<Token, Entry>>,
}

#[tokio::main]
async fn main() -> Result<(), E> {
    utils::setup_tracing();

    let buffer = Buffer::default();
    let buffer_ref = &buffer;

    let mut shutdown = signal(SignalKind::terminate())?;

    let event_func = move |event: Request| async move {
        let (parts, body) = event.into_parts();

        let token = parts.uri.path().split("/").last().unwrap();

        info!("Writing: {} {:?}", token, body);
        buffer_ref.data.write().unwrap().insert(token.to_string(), body.to_vec());

        Ok(Response::builder().status(StatusCode::ACCEPTED).body(()).unwrap())
    };

    // lambda_http::run(service_fn(event_func)).await?;
    tokio::select! {
        _ = lambda_http::run(service_fn(event_func)) => {},
        _ = shutdown.recv() => {
            info!("Flushing: {:?}", buffer_ref.data);
            buffer_ref.data.write().unwrap().clear();
        },
    }

    Ok(())
}

