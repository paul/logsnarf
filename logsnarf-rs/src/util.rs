use std::error::Error;

use tracing_forest::ForestLayer;
use tracing_subscriber::{
    layer::SubscriberExt, registry::Registry, util::SubscriberInitExt, EnvFilter,
};

pub fn setup() -> Result<sentry::ClientInitGuard, Box<dyn Error>> {
    // let tracer =
    //     opentelemetry_jaeger::new_pipeline().install_batch(opentelemetry::runtime::Tokio)?;
    // let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let guard = sentry::init((
        "https://5ffc37f324204b7fbda4140a73a9475c@o4504040962916352.ingest.sentry.io/4504040964685824",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            debug: true,
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ));

    Registry::default()
        .with(ForestLayer::default())
        .with(EnvFilter::from_default_env())
        .with(sentry_tracing::layer())
        // .with(sentry::integrations::tracing::layer())
        // .with(telemetry)
        .init();

    // tracing_subscriber::fmt()
    //     .compact()
    //     // enable everything
    //     .with_max_level(tracing::Level::DEBUG)
    //     // sets this to be the default, global collector for this application.
    //     .init();

    Ok(guard)
}

pub fn teardown() -> Result<(), Box<dyn Error>> {
    // opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
