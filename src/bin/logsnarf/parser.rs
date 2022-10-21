use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::{AsyncRead, BufReader};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

use tracing::{debug, info, instrument};

use logsnarf::{
    decoder, error::Result, metric::Metric, parser, record_stream::RecordStream, settings::Settings,
};

pub struct Parser {
    settings: Settings,
}

impl Parser {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
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

        while let Some(line) = stream.next().await {
            line_cnt += 1;
            if let Some(log_data) = parser::parse_line(line.unwrap())? {
                if let Some(decoder) = decoders.iter().find(|dec| dec.matches(&log_data)) {
                    if let Some(metric) = decoder.decode(&log_data)? {
                        metric_cnt += 1;
                        metrics.push(metric);
                    }
                }
            }
        }

        debug!("{:?}", metrics);
        info!(
            "Parsed {:?} bytes in {} lines, resulting in {} metrics",
            bytes, line_cnt, metric_cnt
        );

        Ok(())
    }
}
