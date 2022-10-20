use std::error::Error;

use tracing_subscriber::{
    layer::SubscriberExt, registry::Registry, util::SubscriberInitExt, EnvFilter,
};
use tracing_tree::HierarchicalLayer;

pub fn setup() -> Result<(), Box<dyn Error>> {
    // let tracer =
    //     opentelemetry_jaeger::new_pipeline().install_batch(opentelemetry::runtime::Tokio)?;
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let layer = HierarchicalLayer::default()
        .with_ansi(true)
        .with_targets(true)
        .with_bracketed_fields(true);

    Registry::default()
        .with(layer)
        .with(EnvFilter::from_default_env())
        // .with(telemetry)
        .init();

    Ok(())
}

pub fn teardown() -> Result<(), Box<dyn Error>> {
    // opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
