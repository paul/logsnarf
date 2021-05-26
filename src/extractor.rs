use std::option::Option;

use async_std::io::BufRead;
use async_std::prelude::*;
use log::{debug, info, warn};
use thiserror::Error;

use crate::decoder::find_decoder;
use crate::metric::{self,Metric};
use crate::parser;
use crate::log_data::{LogData, StructuredData};

#[derive(Debug, Error)]
pub enum ExtractorErr {
    #[error(transparent)]
    ParseError(#[from] parser::ParseErr),

    #[error("read error")]
    ReadError { source: std::io::Error },

    #[error(transparent)]
    ExtractKeyError(#[from] metric::ExtractErr),
    
}

type Metrics = Vec<Metric>;

pub async fn extract(data: impl BufRead + std::marker::Unpin) -> Result<Metrics, ExtractorErr> {
    // Ok(Metrics::new())
    let mut metrics = Metrics::with_capacity(5);

    let mut lines = data.lines();
    while let Some(line) = lines.next().await {
        let real_line = line.map_err(|source| ExtractorErr::ReadError {source })?;
        let ld = real_line
            .parse::<LogData>()?;

        if let Some(decoder) = find_decoder(&ld) {
        let sd = ld.msg.parse::<StructuredData>()?;
        let tags = metric::extract_tags(decoder.tag_names, &sd)?;
        let fields = metric::extract_fields(decoder.field_names, &sd)?;

        let metric = Metric {
            timestamp: ld.timestamp,
            name: String::from(decoder.name),
            tags,
            fields,
        };

        metrics.push(metric);
            
        }
    }

    Ok(metrics)
}


#[cfg(test)]
mod tests {

    use async_std::fs::File;
    use async_std::io::BufReader;
    // use std::fs::File;
    // use std::io::BufReader;

    use chrono::prelude::*;
    use chrono::Utc;

    use super::*;
    use crate::metric::FieldValue;

    use std::collections::HashMap;
    macro_rules! collection {
        // map-like
        ($($k:expr => $v:expr),* $(,)?) => {
            std::iter::Iterator::collect(std::array::IntoIter::new([$(($k, $v),)*]))
        };
        // set-like
        ($($v:expr),* $(,)?) => {
            std::iter::Iterator::collect(std::array::IntoIter::new([$($v,)*]))
        };
    }

    async fn read_sample(filename: &str) -> std::io::Result<impl BufRead> {
        let f = File::open(format!("{}{}", "./samples/", filename)).await?;
        Ok(BufReader::new(f))
    }

    #[async_std::test]
    async fn test_none() {
        let b = read_sample("none.log").await.unwrap();
        let r = extract(b).await.unwrap();
        let empty = Metrics::new();
        assert_eq!(r, empty);
    }

    #[async_std::test]
    async fn test_dyno_mem() {
        let b = read_sample("dyno_mem.log").await.unwrap();
        let r = &extract(b).await.unwrap()[1];
        // let mut metrics = Metrics::new();
        // metrics.push(Metric {
        //     timestamp: Utc.ymd(2019, 11, 25).and_hms_micro(18, 28, 15, 490955),
        //     name: "dyno_mem".to_string(),
        //     tags: decoder::Tags::new(),
        //     fields: decoder::Fields::new(),
        // });

        assert_eq!(r.timestamp, Utc.ymd(2019, 11, 25).and_hms_micro(18, 28, 15, 490955));
        assert_eq!(r.name, "heroku_dyno_memory".to_string());
        assert_eq!(r.tags, collection! {"source".to_string() => "sqs_background_worker.1".to_string()});
        // assert_eq!(r.fields, collection! {
        //     "sample#memory_total".to_string()   => "324.41MB".to_string().parse::<FieldValue>().unwrap(),
        //     "sample#memory_rss".to_string()     => "317.93MB".to_string().parse::<FieldValue>().unwrap(),
        //     "sample#memory_pgpgout".to_string() => "62370pages".to_string().parse::<FieldValue>().unwrap(),
        //     "sample#memory_cache".to_string()   => "6.48MB".to_string().parse::<FieldValue>().unwrap(),
        //     "sample#memory_quota".to_string()   => "512.00MB".to_string().parse::<FieldValue>().unwrap(),
        //     "sample#memory_swap".to_string()    => "0.00MB".to_string().parse::<FieldValue>().unwrap(),
        //     "sample#memory_pgpgin".to_string()  => "145418pages".to_string().parse::<FieldValue>().unwrap(),
        // });
        assert_eq!(r.fields, collection! {
            "memory_total".to_string()   => "324.31MB".parse::<FieldValue>().unwrap(),
        });
    }
}
