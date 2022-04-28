use std::str;

use lambda_http::{Request, RequestExt};

use tracing::{info, instrument};

use crate::{decoder::decode, Metric};

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Token = String;

pub struct Metrics {
    pub token: Token,
    pub metrics: Vec<Metric>,
}

#[instrument(level = "info", skip(req), fields(token, bytes, lines, metrics))]
pub fn extract_metrics(req: Request) -> Result<Metrics, E> {
    let _context = req.lambda_context();
    let (parts, body) = req.into_parts();
    let token = parts.uri.path().split("/").last().unwrap();

    tracing::Span::current().record("token", &token);
    tracing::Span::current().record("bytes", &body.len());

    let mut stream = body.split(|c| *c == b'\n');

    let mut lines: u16 = 0;
    let mut metrics: Vec<Metric> = Vec::with_capacity(5);

    while let Some(line) = stream.next() {
        lines += 1;
        if let Ok(Some(metric)) = decode(str::from_utf8(line)?.to_string()) {
            metrics.push(metric);
        }
    }

    tracing::Span::current().record("lines", &lines);
    tracing::Span::current().record("metrics", &metrics.len());

    info!("extracted metrics");

    Ok(Metrics {
        token: token.to_string(),
        metrics: metrics,
    })
}
