use std::collections::HashMap;
use std::sync::RwLock;
use std::str;

use tracing::info;

use lambda_http::{
    service_fn,
    Request, RequestExt,
    Response, IntoResponse,
    http::StatusCode};

use tokio::signal::unix::{signal, SignalKind};

use logsnarf::utils;
use logsnarf::decoder::decode;
use logsnarf::metric::Metric;
use logsnarf::metric_store::MetricStore;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Default)]
struct StoreWrapper {
    pub store: RwLock<MetricStore>,
}

#[tokio::main]
async fn main() -> Result<(), E> {
    utils::setup_tracing();


    let store = StoreWrapper::default();
    let store_ref = &store;

    let mut shutdown = if utils::local_lambda() {
        signal(SignalKind::interrupt())?
    } else {
        signal(SignalKind::terminate())?
    };

    tokio::select! {
        _ = lambda_http::run(service_fn(move |event: Request| handle_event(store_ref, event))) => {},
        _ = shutdown.recv() => {
            flush_all(&store_ref).await?
        },
    }

    Ok(())
}

async fn handle_event(store: &StoreWrapper, req: Request) -> Result<impl IntoResponse, E> {
    let _context = req.lambda_context();
    let (parts, body) = req.into_parts();

    let token = parts.uri.path().split("/").last().unwrap();

    let mut stream = body.split(|c| *c == b'\n');

    let mut metrics: Vec<Metric> = Vec::with_capacity(5);

    while let Some(line) = stream.next() {
        if let Ok(Some(metric)) = decode(str::from_utf8(line)?.to_string()) {
            metrics.push(metric);
        }
    }

    let mut data = store.store.write().unwrap();
    data.push(token.to_owned(), metrics)?;

    Ok(Response::builder().status(StatusCode::ACCEPTED).body(()).unwrap())
}

async fn flush_all(store: &StoreWrapper) -> Result<(), E> {
    info!("Flushing: {:?}", store.store);
    let mut data = store.store.write().unwrap();
    data.flush_all()?;
    Ok(())
}
