use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::BufReader;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

use tracing::{debug_span, field, instrument, trace_span};

use logsnarf::{
    decoder::{self, Decoder},
    error::Result,
    metric::Metric,
    parser::{self, LogData},
    record_stream::RecordStream,
    settings::Settings,
};

pub struct Parser {
    settings: Settings,
}

impl Parser {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    #[instrument(
        name = "Parser::parse",
        level = "debug",
        skip(self),
        fields(bytes, lines, metrics)
    )]
    pub async fn parse(&self, filename: String) -> Result<()> {
        let file = File::open(&filename).await?;
        let data = BufReader::new(file);

        let decoders = decoder::build_decoders(&self.settings.metrics);
        let mut metrics: Vec<Metric> = Vec::new();

        let mut line_cnt: u64 = 0;
        let mut metric_cnt: u64 = 0;
        let bytes = Arc::new(AtomicUsize::new(0));

        let data = RecordStream::new(data, bytes.clone());
        let mut stream = FramedRead::new(data, LinesCodec::new_with_max_length(16 * 1024));

        while let Some(Ok(line)) = stream.next().await {
            let span = trace_span!(
                "extract_metric",
                line,
                message = field::Empty,
                metric = field::Empty
            );
            let _enter = span.enter();

            line_cnt += 1;
            Self::parse_line(line.as_ref())?.and_then(|ld| {
                span.record("message", field::debug(&ld));
                Self::find_decoder(&decoders, &ld).and_then(|decoder| {
                    Self::decode_metric(&decoder, &ld).ok()?.and_then(|metric| {
                        metric_cnt += 1;
                        span.record("metric", field::debug(&metric));
                        metrics.push(metric);
                        Some(())
                    })
                })
            });
        }

        tracing::Span::current().record("bytes", field::debug(bytes));
        tracing::Span::current().record("lines", line_cnt);
        tracing::Span::current().record("metrics", metric_cnt);

        Ok(())
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
