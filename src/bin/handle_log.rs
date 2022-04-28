
use lambda_http::{
    service_fn,
    Request,
    Response, IntoResponse,
    http::StatusCode};

use logsnarf::{app, utils};

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    utils::setup_tracing();

    let aws_shared_config = aws_config::load_from_env().await;
    let kinesis = aws_sdk_kinesis::Client::new(&aws_shared_config);

    lambda_http::run(service_fn(|event: Request| handle_event(&kinesis, event))).await?;

    Ok(())
}

async fn handle_event(kinesis: &aws_sdk_kinesis::Client, req: Request) -> Result<impl IntoResponse, E> {
    let metrics = app::extract_metrics(req)?;

    let records: Vec<aws_sdk_kinesis::model::PutRecordsRequestEntry> = metrics.metrics.iter().map(|metric| { 
                aws_sdk_kinesis::model::PutRecordsRequestEntry::builder()
                .set_data(Some(metric.into()))
                .set_partition_key(Some(metrics.token.clone()))
                .build()
    }).collect();

    if records.len() > 0 {
        kinesis
            .put_records()
            .stream_name("logsnarf-test-stream")
            .set_records(Some(records))
            .send()
            .await?;
    }

    Ok(Response::builder().status(StatusCode::ACCEPTED).body(()).unwrap())
}
