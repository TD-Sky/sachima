use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use poem::http::StatusCode;
use poem::test::TestClient;
use poem::Route;
use poem::{delete, get, post, put};
use poem::{Endpoint, EndpointExt};
use tempdir::TempDir;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::reply::status::*;
use crate::router::{http_error, reply_error};
use crate::utils::{self, tests::*};

async fn create_txt(path: PathBuf, content: &str) -> io::Result<()> {
    let mut fd = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .unwrap();
    let mut fd = BufWriter::new(&mut fd);

    if !content.is_empty() {
        fd.write_all(content.as_bytes()).await?;
    }
    fd.flush().await?;

    Ok(())
}

fn setup<E>(path: &str, ep: E) -> (TempDir, TestClient<impl Endpoint>)
where
    E: Endpoint + 'static,
{
    let (tmp_dir, wk) = setup_workspace();
    let app = Route::new()
        .at(path, ep)
        .catch_error(reply_error)
        .data(Arc::new(wk));

    (tmp_dir, TestClient::new(app))
}

#[tokio::test]
async fn test_download() -> io::Result<()> {
    let (tmp_dir, wk) = setup_workspace();
    let root = tmp_dir.path();
    let app = Route::new()
        .at("/file/*path", super::download)
        .catch_error(http_error)
        .data(Arc::new(wk));
    let client = TestClient::new(app);

    client
        .get("/file/")
        .send()
        .await
        .assert_status(StatusCode::FORBIDDEN);

    client
        .get("/file/not-found")
        .send()
        .await
        .assert_status(StatusCode::NOT_FOUND);

    fs::create_dir(root.join("is-a-directory")).await?;
    client
        .get("/file/is-a-directory")
        .send()
        .await
        .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);

    create_txt(root.join("download-file.txt"), "the downloaded content").await?;
    client
        .get("/file/download-file.txt")
        .send()
        .await
        .assert_status_is_ok();

    Ok(())
}

#[tokio::test]
async fn test_upload_file() -> io::Result<()> {
    let (tmp_dir, client) = setup("/file/*path", post(super::upload));
    let content = "the uploaded content";

    assert_buss_status(
        MISSING_PARENT,
        client
            .post("/file/missing-parent/upload.txt")
            .body(content)
            .send()
            .await
            .json()
            .await,
    );

    create_txt(tmp_dir.path().join("already-exists"), "").await?;
    assert_buss_status(
        ALREADY_EXISTS,
        client
            .post("/file/already-exists")
            .body(content)
            .send()
            .await
            .json()
            .await,
    );

    assert_buss_status(
        OK,
        client
            .post("/file/upload.txt")
            .body(content)
            .send()
            .await
            .json()
            .await,
    );

    Ok(())
}

#[tokio::test]
async fn test_rename() -> io::Result<()> {
    let (tmp_dir, client) = setup("/file/*path", put(super::rename));
    let root = tmp_dir.path();

    assert_buss_status(
        NOT_FOUND,
        client
            .put("/file/not-found")
            .query("name", &"rename-not-found")
            .send()
            .await
            .json()
            .await,
    );

    create_txt(root.join("rename-to-already-exists"), "").await?;
    create_txt(root.join("already-exists"), "").await?;
    assert_buss_status(
        ALREADY_EXISTS,
        client
            .put("/file/rename-to-already-exists")
            .query("name", &"already-exists")
            .send()
            .await
            .json()
            .await,
    );

    fs::create_dir(root.join("rename-old-dir")).await?;
    assert_buss_status(
        OK,
        client
            .put("/file/rename-old-dir")
            .query("name", &"rename-new-dir")
            .send()
            .await
            .json()
            .await,
    );

    fs::create_dir(root.join("rename-old-file")).await?;
    assert_buss_status(
        OK,
        client
            .put("/file/rename-old-file")
            .query("name", &"rename-new-file")
            .send()
            .await
            .json()
            .await,
    );

    Ok(())
}

#[tokio::test]
async fn test_remove_entity() -> io::Result<()> {
    let (tmp_dir, client) = setup("/file/*path", delete(super::remove));
    let root = tmp_dir.path();

    assert_buss_status(
        NOT_FOUND,
        client.delete("/file/not-found").send().await.json().await,
    );

    fs::create_dir(root.join("remove-dir")).await?;
    assert_buss_status(
        OK,
        client.delete("/file/remove-dir").send().await.json().await,
    );

    create_txt(root.join("remove-file"), "").await?;
    assert_buss_status(
        OK,
        client.delete("/file/remove-file").send().await.json().await,
    );

    Ok(())
}

#[tokio::test]
async fn test_mkdir() -> io::Result<()> {
    let (tmp_dir, client) = setup("/dir/*path", post(super::mkdir));

    assert_buss_status(
        MISSING_PARENT,
        client
            .post("/dir/missing-parent/make-dir")
            .send()
            .await
            .json()
            .await,
    );

    fs::create_dir(tmp_dir.path().join("already-exists")).await?;
    assert_buss_status(
        ALREADY_EXISTS,
        client.post("/dir/already-exists").send().await.json().await,
    );

    assert_buss_status(OK, client.post("/dir/make-dir").send().await.json().await);

    Ok(())
}
