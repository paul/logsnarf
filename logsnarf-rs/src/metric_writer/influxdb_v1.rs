use std::io::{self, Write};

use async_trait::async_trait;
use bytes::BufMut;
use chrono::{DateTime, Utc};
use futures::prelude::*;
use reqwest;
use serde_derive::Deserialize;
use thiserror::Error;
use tracing::instrument;
use url::Url;

use crate::{
    metric::{self, Metric},
    metric_writer::{MetricWriter, WriterError},
};

#[derive(Debug, Error)]
pub enum InfluxdbV1Error {
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub url: Url,
}

pub struct InfluxdbV1 {
    credentials: Credentials,
    metrics: Vec<Metric>,
    client: reqwest::Client,
    write_url: Url,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

impl InfluxdbV1 {
    pub fn new(creds: &Credentials) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();

        Self {
            credentials: creds.clone(),
            metrics: Vec::with_capacity(100),
            client,
            write_url: creds.url.join("write").expect("bogus influxdb url!"),
        }
    }
}

#[async_trait]
impl MetricWriter for InfluxdbV1 {
    #[instrument(skip(self))]
    fn write(&mut self, metric: Metric) {
        self.metrics.push(metric)
    }

    #[instrument(skip(self), fields(count, response))]
    async fn flush(&mut self) -> Result<(), WriterError> {
        let count = self.metrics.len();
        let buf = stream::iter(self.metrics.clone());
        self.metrics.clear();

        let mut buffer = bytes::BytesMut::new();

        let body = buf.map(move |point| {
            let mut w = (&mut buffer).writer();
            point.write_data_point_to(&mut w)?;
            w.flush()?;
            Ok::<_, io::Error>(buffer.split().freeze())
        });

        let body = reqwest::Body::wrap_stream(body);

        let response = self
            .client
            .post(self.write_url.clone())
            .query(&[("db", "logsnarf"), ("precision", "u")])
            .body(body)
            .send()
            .await
            .map_err(|e| InfluxdbV1Error::HttpError(e))?;

        tracing::Span::current().record("count", count);
        tracing::Span::current().record("response", response.status().as_str());
        Ok(())
    }
}

pub trait WriteDataPoint {
    /// Write this data point as line protocol. The implementor is responsible
    /// for properly escaping the data and ensuring that complete lines
    /// are generated.
    fn write_data_point_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write;
}

impl WriteDataPoint for Metric {
    fn write_data_point_to<W>(&self, mut w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        escape_and_write_value(&self.name, MEASUREMENT_DELIMITERS, &mut w)?;

        for (k, v) in &self.tags {
            w.write_all(b",")?;
            k.write_tag_key_to(&mut w)?;
            w.write_all(b"=")?;
            v.write_tag_value_to(&mut w)?;
        }

        for (i, (k, v)) in self.fields.iter().enumerate() {
            let d = if i == 0 { b" " } else { b"," };

            w.write_all(d)?;
            k.write_field_key_to(&mut w)?;
            w.write_all(b"=")?;
            v.write_field_value_to(&mut w)?;
        }

        w.write_all(b" ")?;
        self.timestamp.write_timestamp_to(&mut w)?;

        w.write_all(b"\n")?;

        Ok(())
    }
}

trait WriteTagKey {
    fn write_tag_key_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write;
}

impl WriteTagKey for str {
    fn write_tag_key_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        escape_and_write_value(self, TAG_KEY_DELIMITERS, w)
    }
}

trait WriteTagValue {
    fn write_tag_value_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write;
}

impl WriteTagValue for str {
    fn write_tag_value_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        escape_and_write_value(self, TAG_VALUE_DELIMITERS, w)
    }
}

trait WriteFieldKey {
    fn write_field_key_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write;
}

impl WriteFieldKey for str {
    fn write_field_key_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        escape_and_write_value(self, FIELD_KEY_DELIMITERS, w)
    }
}

trait WriteFieldValue {
    fn write_field_value_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write;
}

impl WriteFieldValue for metric::FieldValue {
    fn write_field_value_to<W>(&self, mut w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        use metric::FieldValue::*;

        match self {
            Boolean(v) => write!(w, "{}", if *v { "t" } else { "f" }),
            Float(v, _) => write!(w, "{}", v),
            Integer(v, _) => write!(w, "{}i", v),
            Text(v) => {
                w.write_all(br#"""#)?;
                escape_and_write_value(v, FIELD_VALUE_STRING_DELIMITERS, &mut w)?;
                w.write_all(br#"""#)
            }
        }
    }
}

trait WriteTimestamp {
    fn write_timestamp_to<W>(&self, w: W) -> io::Result<()>
    where
        W: io::Write;
}

impl WriteTimestamp for DateTime<Utc> {
    fn write_timestamp_to<W>(&self, mut w: W) -> io::Result<()>
    where
        W: io::Write,
    {
        write!(
            w,
            "{}",
            self.timestamp() * 1000000 + self.timestamp_subsec_micros() as i64
        )
    }
}

const MEASUREMENT_DELIMITERS: &[char] = &[',', ' '];
const TAG_KEY_DELIMITERS: &[char] = &[',', '=', ' '];
const TAG_VALUE_DELIMITERS: &[char] = TAG_KEY_DELIMITERS;
const FIELD_KEY_DELIMITERS: &[char] = TAG_KEY_DELIMITERS;
const FIELD_VALUE_STRING_DELIMITERS: &[char] = &['"'];

fn escape_and_write_value<W>(
    value: &str,
    escaping_specification: &[char],
    mut w: W,
) -> io::Result<()>
where
    W: io::Write,
{
    let mut last = 0;

    for (idx, delim) in value.match_indices(escaping_specification) {
        let s = &value[last..idx];
        write!(w, r#"{}\{}"#, s, delim)?;
        last = idx + delim.len();
    }

    w.write_all(value[last..].as_bytes())
}
