use lambda_http::{service_fn, Request, IntoResponse};
// use products::{entrypoints::lambda::apigateway::get_product, utils::*};

use tracing::{info, instrument};

use logsnarf::utils::*;
use logsnarf::entrypoints::lambda::httpfunction::handle_log_event;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    // Initialize logger
    setup_tracing();

    // Initialize store
    let app = get_app().await;

    // Run the Lambda function
    //
    // This is the entry point for the Lambda function. The `lambda_http`
    // crate will take care of contacting the Lambda runtime API and invoking
    // the `handle_log_event` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass a store to a lambda function.
    //
    // Furthermore, we don't await the result of `handle_log_event` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_http::run(service_fn(|event: Request| handle_log_event(&app, event))).await?;
    Ok(())
}

