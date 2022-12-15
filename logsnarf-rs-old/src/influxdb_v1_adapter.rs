use std::collections::HashMap;
use std::vec::Vec;

use async_std::task;
use log::{debug,info};
use influxdb::{Client, Query, Timestamp};
use thiserror::Error;

use crate::adapter::Adapter;
// use crate::credentials::InfluxDbCredentials;
use crate::decoder::{Metric,FieldValue};


#[derive(Debug, Error)]
pub enum ClientErr {
}



pub struct InfluxDbV1Adapter {
    client: Client,
}

pub async fn connect(url: &String) -> Result<InfluxDbV1Adapter, ClientErr> {
    let mut client = Client::new(url, "logsnarf");

    Ok(InfluxDbV1Adapter{client: client})

}


// impl InfluxDbV1Adapter {
//     pub fn new(credentials: &InfluxDbCredentials) -> InfluxDbV1Adapter {
//         InfluxDbV1Adapter {
//             client: Client::new(
//                 Url::parse(&credentials.influxdb_url.clone()).unwrap(),
//                 "logsnarf",
//             ),
//         }
//     }
// }

impl Adapter for InfluxDbV1Adapter {

    fn write(&self, metrics: Vec<Metric>) {
        let points: Vec<Point> = metrics.into_iter().map(|m| Point::from(m)).collect();
            let l = points.len();
            let ps = Points::create_new(points);
            // self.client.write_points(ps, Some(Precision::Microseconds), None).await.unwrap();
            client.query(points).await;
            info!("wrote {} points to influxdb", l);
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

impl From<Metric> for WriteQuery {
    fn from(m: Metric) -> Self {
        let mut q = Timestamp::Seconds(m.timestamp.timestamp)
            .into_query(m.name);
        m.tags.into_iter().map(|(k,v)| q.add_tag(k, v));
        m.fields.into_iter().map(|(k,v)| q.add_field(k, v));

        q.build()
    }
}
