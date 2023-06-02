use std::sync::Arc;

use jwt_codec::prelude::Hs256;
use jwt_codec::Codec;
use poem::http::StatusCode;
use poem::Route;
use poem::{get, post};
use poem::{Endpoint, EndpointExt};
use poem::{IntoResponse, Response};

use crate::config::Workspace;
use crate::handlers::*;
use crate::middlewares::JwtVerifier;
use crate::reply::ReplyError;
use crate::Config;

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

pub fn new(config: Config) -> Route {
    let codec = Arc::new(Codec::hs256(config.jwt_secret_key.as_bytes()));

    Route::new()
        .nest("/wk", workspace(config.workspace, codec.clone()))
        .at("/user/register", post(permission::register))
        .at("/user/login", post(permission::login).data(codec.clone()))
        .at(
            "/user/info",
            get(permission::info).with(JwtVerifier::new(codec)),
        )
}

fn workspace(wk: Workspace, codec: Arc<Codec<Hs256>>) -> impl Endpoint {
    Route::new()
        .nest("/read", read_wk())
        .nest("/write", write_wk().with(JwtVerifier::new(codec)))
        .data(Arc::new(wk))
}

fn read_wk() -> impl Endpoint {
    Route::new()
        .at(
            "/file/*path",
            get(file_system::download)
                .before(file_system::ensure_relative)
                .catch_error(http_error),
        )
        .at(
            "/dir/*path",
            get(file_system::read_dir)
                .before(file_system::ensure_relative)
                .catch_error(reply_error),
        )
}

/// Write the file system,
/// these handlers cannot operate the workspace root.
#[rustfmt::skip]
fn write_wk() -> impl Endpoint {
    Route::new()
        .at(
            "/file/*path",
            post(file_system::upload)
           .put(file_system::rename)
           .delete(file_system::remove)
                .before(file_system::ensure_relative)
                .before(file_system::ensure_not_root),
        )
        .at(
            "/dir/*path",
            post(file_system::mkdir)
                .before(file_system::ensure_relative)
                .before(file_system::ensure_not_root),
        )
        .catch_error(reply_error)
}

#[cfg(test)]
mod tests {
    use super::{read_wk, write_wk};
    use crate::reply::status::{IS_ABSOLUTE, WORKSPACE_ROOT};
    use crate::utils::tests::*;
    use poem::http::StatusCode;
    use poem::test::TestClient;
    use poem::EndpointExt;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_read_workspace() {
        let (_tmp_dir, wk) = setup_workspace();
        let client = TestClient::new(read_wk().data(Arc::new(wk)));

        client
            .get("/file//")
            .send()
            .await
            .assert_status(StatusCode::FORBIDDEN);

        assert_buss_status(IS_ABSOLUTE, client.get("/dir//").send().await.json().await);
    }

    #[tokio::test]
    async fn test_write_workspace() {
        let (_tmp_dir, wk) = setup_workspace();
        let client = TestClient::new(write_wk().data(Arc::new(wk)));

        assert_buss_status(
            WORKSPACE_ROOT,
            client.delete("/file/").send().await.json().await,
        );

        assert_buss_status(
            IS_ABSOLUTE,
            client.delete("/file//").send().await.json().await,
        );
    }
}
