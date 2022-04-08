use lambda_http::{http::StatusCode, IntoResponse, Request, Response};
use tracing::{error, info, instrument, warn};

use crate::{app, domain};

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

#[instrument(skip(app))]
pub async fn handle_log_event(
    app: &dyn app::AppHandle,
    event: Request,
) -> Result<impl IntoResponse, E> {
    let (parts, body) = event.into_parts();

    let token = match parts.uri.path().split("/").last() {
        Some(token) => token,
        None => {
            warn!("Missing 'token' parameter!");
            return Ok(response(StatusCode::NOT_FOUND));
        }
    };


    let res = domain::handle_log(app, token, body.to_vec()).await;

    Ok(match res {
        Ok(_) => {
            info!("Got it");
            response(StatusCode::ACCEPTED)
        },
        Err(err) => {
            error!("Failed to read data! {}", err);
            response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    })
}

fn response(status_code: StatusCode) -> Response<String> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "text/plain")
        .body("".to_string())
        .unwrap()
}
