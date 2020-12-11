use std::collections::HashMap;
use std::vec::Vec;

use log::{debug,info};
use influx_db_client::{Client, Point, Points, Precision, Value};
use reqwest::Url;

use crate::decoder::{Metric, FieldValue};

pub struct Writer {
    buffer: Vec<Metric>,
    influxdb: Client,
}

impl Writer {
    pub fn new() -> Writer {
        Writer {
            buffer: Vec::with_capacity(1000),
            influxdb: Client::new(Url::parse("http://localhost:8086/").unwrap(), "logsnarf"),
        }
    }

    pub fn write(&mut self, metric: Metric) {
        self.buffer.push(metric);

        if self.buffer.len() >= 1000 {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
            // TODO look up the adapter to use

            self.write_influx(self.buffer.clone());
            self.buffer.clear();

    }

    fn write_influx(&self, metrics: Vec<Metric>) {
        let points: Vec<Point> = metrics.into_iter().map(|m| Point::from(m)).collect();
        tokio::runtime::Runtime::new().unwrap().block_on(async move {
            let l = points.len();
            let ps = Points::create_new(points);
            self.influxdb.write_points(ps, Some(Precision::Microseconds), None).await.unwrap();
            info!("wrote {} points to influxdb", l);

        })
    }

}

impl From<FieldValue> for Value {
    fn from(val: FieldValue) -> Self {
        match val {
            FieldValue::Boolean(v) => Value::Boolean(v),
            FieldValue::Float(v) => Value::Float(v),
            FieldValue::Integer(v) => Value::Integer(v),
            FieldValue::Text(v) => Value::String(v),
        }


    }
}

impl From<Metric> for Point {
    fn from(m: Metric) -> Self {
        let ts = Some((m.timestamp.sec as i64 * 1000000) + (m.timestamp.nsec as i64 / 1000));
        let tags = m.tags.into_iter().map(|(k,v)| (k, Value::String(v))).collect::<HashMap<_, _>>();
        let fields = m.fields.into_iter().map(|(k,v)| (k, Value::from(v))).collect::<HashMap<_, _>>();
        Point {
            timestamp: ts,
            measurement: m.name,
            tags,
            fields,

        }

    }
}



