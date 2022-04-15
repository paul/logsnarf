use tracing_subscriber::{prelude::*, Layer};

/// Setup tracing
pub fn setup_tracing() {
    let env_log = tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());
    let subscriber = tracing_subscriber::Registry::default().with(env_log);

    let host_subscriber = if local_lambda() {
        None
    } else {
        Some(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .without_time(),
        )
    };

    let subscriber = subscriber.with(host_subscriber);

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}

pub fn local_lambda() -> bool {
    match std::env::var("TERM") {
        Ok(_) => true,
        Err(_) => false,
    }
}
