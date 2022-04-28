use tracing::info;

/// Setup tracing
pub fn setup_tracing() {
    if local_lambda() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
        info!("Configured logging for local lambda env");
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            // this needs to be set to false, otherwise ANSI color codes will
            // show up in a confusing manner in CloudWatch logs.
            .with_ansi(false)
            // disabling time is handy because CloudWatch will add the ingestion time.
            .without_time()
            .init();
        info!("Configured logging for AWS Lambda Env");
    };
}

pub fn local_lambda() -> bool {
    match std::env::var("TERM") {
        Ok(_) => true,
        Err(_) => false,
    }
}
