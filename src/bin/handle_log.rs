use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::RwLock;

use tracing::info;

use lambda_http::{
    service_fn,
    Request, RequestExt,
    Response, IntoResponse,
    http::StatusCode};

use tokio::signal::unix::{signal, SignalKind};

use logsnarf::utils;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

type Token = String;
type Metric = Vec<u8>;

#[derive(Default)]
struct Buffer {
    pub data: RwLock<HashMap<Token, Vec<Metric>>>,
}

#[tokio::main]
async fn main() -> Result<(), E> {
    utils::setup_tracing();

    let buffer = Buffer::default();
    let buffer_ref = &buffer;

    let mut shutdown = if utils::local_lambda() {
        signal(SignalKind::interrupt())?
    } else {
        signal(SignalKind::terminate())?
    };

    tokio::select! {
        _ = lambda_http::run(service_fn(move |event: Request| handle_event(buffer_ref, event))) => {},
        _ = shutdown.recv() => {
            flush_all(&buffer_ref).await?
        },
    }

    Ok(())
}

async fn handle_event(buffer: &Buffer, req: Request) -> Result<impl IntoResponse, E> {
    let _context = req.lambda_context();
    let (parts, body) = req.into_parts();

    let token = parts.uri.path().split("/").last().unwrap();

    info!("Writing: {} {:?}", token, body);
    // buffer.data.write().unwrap().insert(token.to_string(), body.to_vec());
    let mut data = buffer.data.write().unwrap();
    match data.entry(token.to_string()) {
        Entry::Occupied(mut e) => { e.get_mut().push(body.to_vec()); },
        Entry::Vacant(e) => {e.insert(vec![body.to_vec()]); },
    };

    Ok(Response::builder().status(StatusCode::ACCEPTED).body(()).unwrap())
}

async fn flush_all(buffer: &Buffer) -> Result<(), E> {
    info!("Flushing: {:?}", buffer.data);
    buffer.data.write().unwrap().clear();
    Ok(())
}
