use crate::handlers::file_system;
use poem::error::InternalServerError;
use poem::http::StatusCode;
use poem::{get, post, Response};
use poem::{Endpoint, EndpointExt};
use poem::{IntoResponse, Route};

use crate::reply;
use crate::reply::ReplyError;

async fn http_error(e: reply::Error) -> StatusCode {
    match e {
        reply::Error::WorkspaceRoot => StatusCode::FORBIDDEN,
        reply::Error::NotFound => StatusCode::NOT_FOUND,
        reply::Error::IsADirectory => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        reply::Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => unreachable!(),
    }
}

async fn reply_error(e: reply::Error) -> Response {
    match ReplyError::try_from(e) {
        Ok(rp) => rp.into_response(),
        Err(e) => InternalServerError(e).into_response(),
    }
}

pub fn new() -> Route {
    Route::new()
        .at(
            "/rfs/*path",
            get(file_system::download).catch_error(http_error),
        )
        .at(
            "/rdir/*path",
            get(file_system::read).catch_error(reply_error),
        )
        .nest_no_strip("/", workspace_guard())
}

/// Write the file system,
/// these handlers cannot operate the workspace root.
#[rustfmt::skip]
fn workspace_guard() -> impl Endpoint {
    Route::new()
        .at(
            "/wfs/*path",
            post(file_system::upload)
           .put(file_system::rename)
           .delete(file_system::remove)
                .before(file_system::ensure_not_root),
        )
        .at(
            "/wdir/*path",
            post(file_system::make)
                .before(file_system::ensure_not_root),
        )
        .catch_error(reply_error)
}
