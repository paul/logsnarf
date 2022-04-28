use std::str;

use lambda_http::{
    service_fn,
    Request, RequestExt,
    Response, IntoResponse,
    http::StatusCode};

use logsnarf::utils;
use logsnarf::decoder::decode;

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
    let _context = req.lambda_context();
    let (parts, body) = req.into_parts();

    let token = parts.uri.path().split("/").last().unwrap();

    let mut stream = body.split(|c| *c == b'\n');

    let mut records: Vec<aws_sdk_kinesis::model::PutRecordsRequestEntry> = Vec::with_capacity(5);

    while let Some(line) = stream.next() {
        if let Ok(Some(metric)) = decode(str::from_utf8(line)?.to_string()) {
            records.push(
                aws_sdk_kinesis::model::PutRecordsRequestEntry::builder()
                .set_data(Some(metric.into()))
                .set_partition_key(Some(token.to_string()))
                .build()
            )
        }
    }

    kinesis
        .put_records()
        .stream_name("logsnarf-test")
        .set_records(Some(records))
        .send()
        .await?;


    Ok(Response::builder().status(StatusCode::ACCEPTED).body(()).unwrap())
}
