
use crate::{
    error::Error,
    app::AppHandle
};

pub async fn handle_log(app: &dyn AppHandle, token: &str, body: Vec<u8>) -> Result<(), Error> {
    app.handle(token, body).await
}
