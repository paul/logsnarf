use std::sync::{
    atomic::{self, AtomicUsize},
    Arc,
};

use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{debug, instrument};

use crate::{
    decoder::{self, Decoder},
    error::Result,
    metric::Metric,
    parser::{self, LogData},
    record_stream::RecordStream,
    settings::Settings,
};

pub struct App {
    settings: Settings,
    decoders: Vec<Decoder>,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let decoders = decoder::build_decoders(&settings.metrics);

        Self { settings, decoders }
    }

    #[instrument(skip(self, data), fields(bytes, lines, metrics))]
    pub async fn extract(&self, data: impl AsyncRead + std::marker::Unpin) -> Result<()> {
        let mut metrics: Vec<Metric> = Vec::new();

        let mut line_cnt: u64 = 0;
        let mut metric_cnt: u64 = 0;
        let bytes = Arc::new(AtomicUsize::new(0));

        let data = RecordStream::new(data, bytes.clone());
        let mut stream = FramedRead::new(data, LinesCodec::new_with_max_length(16 * 1024));

        while let Some(Ok(line)) = stream.next().await {
            line_cnt += 1;
            match self.metric_from_line(line.as_ref()) {
                Ok(Some(metric)) => {
                    metric_cnt += 1;
                    metrics.push(metric);
                }
                Ok(None) => {}
                Err(_e) => {
                    // tracing::error!("Problem parsing line: {}\n{}", e, line);
                    // sentry::capture_error(&e);
                }
            }
        }

        tracing::Span::current().record("bytes", bytes.load(atomic::Ordering::Relaxed));
        tracing::Span::current().record("lines", line_cnt);
        tracing::Span::current().record("metrics", metric_cnt);

        debug!(
            "Consumed {:?} bytes, in {} lines, extracted {} metrics",
            bytes, line_cnt, metric_cnt
        );
        Ok(())
    }

    #[instrument(skip(self))]
    fn metric_from_line(&self, line: &str) -> Result<Option<Metric>> {
        Ok(Self::parse_line(line)?.and_then(|ld| {
            Self::find_decoder(&self.decoders, &ld).and_then(|decoder| {
                Self::decode_metric(&decoder, &ld)
                    .ok()?
                    .and_then(|metric| Some(metric))
            })
        }))
    }

    #[instrument]
    fn parse_line(line: &str) -> Result<Option<LogData>> {
        Ok(parser::parse_line(line).map_err(|e| {
            tracing::error!("Problem parsing line: {}", e);
            e
        })?)
    }

    #[instrument]
    fn find_decoder<'a>(decoders: &'a Vec<Decoder>, ld: &'a LogData) -> Option<&'a Decoder> {
        decoders.iter().find(|decoder| decoder.matches(&ld))
    }

    #[instrument]
    fn decode_metric(decoder: &Decoder, ld: &LogData) -> Result<Option<Metric>> {
        Ok(decoder.decode(&ld).map_err(|e| {
            tracing::error!("Problem decoding log message: {}", e);
            e
        })?)
    }
}
