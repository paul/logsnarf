// pub mod app;
// pub mod metric_store;

mod error;
pub use error::{Error, Result};

mod shutdown;
use shutdown::Shutdown;
