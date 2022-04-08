use tracing::{info, instrument};

use crate::app;

/// Setup tracing
pub fn setup_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}

#[instrument]
pub async fn get_app() -> impl app::App {
    info!("Initializing App");
    app::MemoryStore::default()
}
