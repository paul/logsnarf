use std::error::Error;

use tracing_forest::ForestLayer;
use tracing_subscriber::{
    layer::SubscriberExt, registry::Registry, util::SubscriberInitExt, EnvFilter,
};

pub fn setup() -> Result<(), Box<dyn Error>> {
    // let tracer =
    //     opentelemetry_jaeger::new_pipeline().install_batch(opentelemetry::runtime::Tokio)?;
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(ForestLayer::default())
        .with(EnvFilter::from_default_env())
        // .with(telemetry)
        .init();

    // tracing_subscriber::fmt()
    //     .compact()
    //     // enable everything
    //     .with_max_level(tracing::Level::DEBUG)
    //     // sets this to be the default, global collector for this application.
    //     .init();

    Ok(())
}

pub fn teardown() -> Result<(), Box<dyn Error>> {
    // opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
