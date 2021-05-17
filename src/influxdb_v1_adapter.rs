use std::collections::HashMap;
use std::vec::Vec;

use async_std::task;
use log::{debug,info};
use influx_db_client::{Client, Point, Points, Precision, Value};
use reqwest::Url;

use crate::adapter::Adapter;
use crate::credentials::InfluxDbCredentials;
use crate::decoder::{Metric,FieldValue};


pub struct InfluxDbV1Adapter {
    client: Client,
}

impl InfluxDbV1Adapter {
    pub fn new(credentials: &InfluxDbCredentials) -> InfluxDbV1Adapter {
        InfluxDbV1Adapter {
            client: Client::new(
                Url::parse(&credentials.influxdb_url.clone()).unwrap(),
                "logsnarf",
            ),
        }
    }
}

impl Adapter for InfluxDbV1Adapter {

    fn write(&self, metrics: Vec<Metric>) {
        let points: Vec<Point> = metrics.into_iter().map(|m| Point::from(m)).collect();
        task::block_on(async move {
            let l = points.len();
            let ps = Points::create_new(points);
            self.client.write_points(ps, Some(Precision::Microseconds), None).await.unwrap();
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
