use poem::http::StatusCode;
use poem::Route;
use poem::{get, post};
use poem::{Endpoint, EndpointExt};
use poem::{IntoResponse, Response};

use crate::handlers::file_system;
use crate::reply::Error as ReplyError;

pub async fn http_error(e: ReplyError) -> StatusCode {
    match e {
        ReplyError::WorkspaceRoot | ReplyError::IsAbsolute => StatusCode::FORBIDDEN,
        ReplyError::NotFound => StatusCode::NOT_FOUND,
        ReplyError::IsADirectory => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ReplyError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => unreachable!(),
    }
}

pub async fn reply_error(e: ReplyError) -> Response {
    e.into_response()
}

pub fn new() -> Route {
    read_workspace().nest_no_strip("/", write_workspace())
}

fn read_workspace() -> Route {
    Route::new()
        .at(
            "/rfs/*path",
            get(file_system::download)
                .before(file_system::ensure_relative)
                .catch_error(http_error),
        )
        .at(
            "/rdir/*path",
            get(file_system::read_dir)
                .before(file_system::ensure_relative)
                .catch_error(reply_error),
        )
}

/// Write the file system,
/// these handlers cannot operate the workspace root.
#[rustfmt::skip]
fn write_workspace() -> impl Endpoint {
    Route::new()
        .at(
            "/wfs/*path",
            post(file_system::upload)
           .put(file_system::rename)
           .delete(file_system::remove)
                .before(file_system::ensure_relative)
                .before(file_system::ensure_not_root),
        )
        .at(
            "/wdir/*path",
            post(file_system::mkdir)
                .before(file_system::ensure_relative)
                .before(file_system::ensure_not_root),
        )
        .catch_error(reply_error)
}

#[cfg(test)]
mod tests {
    use super::{read_workspace, write_workspace};
    use crate::reply::status::{IS_ABSOLUTE, WORKSPACE_ROOT};
    use crate::utils::tests::*;
    use poem::http::StatusCode;
    use poem::test::TestClient;
    use poem::EndpointExt;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_read_workspace() {
        let (_tmp_dir, wk) = setup_workspace();
        let client = TestClient::new(read_workspace().data(Arc::new(wk)));

        client
            .get("/rfs//")
            .send()
            .await
            .assert_status(StatusCode::FORBIDDEN);

        assert_buss_status(IS_ABSOLUTE, client.get("/rdir//").send().await.json().await);
    }

    #[tokio::test]
    async fn test_write_workspace() {
        let (_tmp_dir, wk) = setup_workspace();
        let client = TestClient::new(write_workspace().data(Arc::new(wk)));

        assert_buss_status(
            WORKSPACE_ROOT,
            client.delete("/wfs/").send().await.json().await,
        );

        assert_buss_status(
            IS_ABSOLUTE,
            client.delete("/wfs//").send().await.json().await,
        );
    }
}
