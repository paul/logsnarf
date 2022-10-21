use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{
    debug, debug_span,
    field::{self, Empty},
    instrument, trace, trace_span, warn,
};

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

    #[instrument(level = "debug", skip(self, data))]
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
                Err(e) => warn!("Problem parsing line: {}\n{}", e, line),
            }
        }

        tracing::Span::current().record("bytes", field::debug(&bytes));
        tracing::Span::current().record("lines", line_cnt);
        tracing::Span::current().record("metrics", metric_cnt);

        debug!(
            "Consumed {:?} bytes, in {} lines, extracted {} metrics",
            bytes, line_cnt, metric_cnt
        );
        Ok(())
    }

    #[instrument(level = "debug", skip(self), fields(message, metric))]
    fn metric_from_line(&self, line: &str) -> Result<Option<Metric>> {
        Ok(Self::parse_line(line)?.and_then(|ld| {
            tracing::Span::current().record("message", field::debug(&ld));
            trace!("message = {:?}", &ld);
            Self::find_decoder(&self.decoders, &ld).and_then(|decoder| {
                trace!("decoder = {}", decoder.name());
                Self::decode_metric(&decoder, &ld).ok()?.and_then(|metric| {
                    tracing::Span::current().record("metric", field::debug(&metric));
                    trace!("metric = {:?}", &metric);
                    Some(metric)
                })
            })
        }))
    }

    #[instrument(name = "parse", level = "trace")]
    fn parse_line(line: &str) -> Result<Option<LogData>> {
        Ok(parser::parse_line(line)?)
    }

    #[instrument(level = "trace")]
    fn find_decoder<'a>(decoders: &'a Vec<Decoder>, ld: &'a LogData) -> Option<&'a Decoder> {
        decoders.iter().find(|decoder| decoder.matches(&ld))
    }

    #[instrument(level = "trace")]
    fn decode_metric(decoder: &Decoder, ld: &LogData) -> Result<Option<Metric>> {
        Ok(decoder.decode(&ld)?)
    }
}
