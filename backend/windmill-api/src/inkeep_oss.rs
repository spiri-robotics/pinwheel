
use axum::{routing::post, Router};

use windmill_common::error::Error;

pub fn global_service() -> Router {
    Router::new().route("/", post(inkeep_not_available))
}

async fn inkeep_not_available() -> windmill_common::error::Result<()> {
    Err(Error::Generic(
        http::StatusCode::FORBIDDEN,
        "Inkeep AI documentation assistant is only available in Windmill Enterprise Edition"
            .to_string(),
    ))
}
