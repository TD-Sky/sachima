use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use poem::http::StatusCode;
use poem::test::{TestClient, TestForm, TestFormField};
use poem::{delete, post, put};
use poem::{get, Route};
use poem::{Endpoint, EndpointExt};
use tempdir::TempDir;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::reply::status::*;
use crate::router::{http_error, reply_error};
use crate::utils::tests::*;

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
        .at("/*path", super::download)
        .catch_error(http_error)
        .data(Arc::new(wk));
    let client = TestClient::new(app);

    client
        .get("/")
        .send()
        .await
        .assert_status(StatusCode::FORBIDDEN);

    client
        .get("/not-found")
        .send()
        .await
        .assert_status(StatusCode::NOT_FOUND);

    fs::create_dir(root.join("is-a-directory")).await?;
    client
        .get("/is-a-directory")
        .send()
        .await
        .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);

    create_txt(root.join("download-file.txt"), "the downloaded content").await?;
    client
        .get("/download-file.txt")
        .send()
        .await
        .assert_status_is_ok();

    Ok(())
}

#[tokio::test]
async fn test_upload_file() -> io::Result<()> {
    fn file_form(filename: &str) -> TestForm {
        let file = TestFormField::bytes("the uploaded content");

        if !filename.is_empty() {
            TestForm::new().field(file.filename(filename))
        } else {
            TestForm::new().field(file)
        }
    }

    let (tmp_dir, client) = setup("/*path", post(super::upload));

    assert_buss_status(
        MISSING_PARENT,
        client
            .post("/missing-parent")
            .multipart(file_form("upload.txt"))
            .send()
            .await
            .json()
            .await,
    );

    create_txt(tmp_dir.path().join("already-exists"), "").await?;
    assert_buss_status(
        ALREADY_EXISTS,
        client
            .post("/")
            .multipart(file_form("already-exists"))
            .send()
            .await
            .json()
            .await,
    );

    assert_buss_status(
        FILE_EXPECTED,
        client
            .post("/")
            .multipart(TestForm::new())
            .send()
            .await
            .json()
            .await,
    );

    assert_buss_status(
        MISSING_FILE_NAME,
        client
            .post("/")
            .multipart(file_form(""))
            .send()
            .await
            .json()
            .await,
    );

    assert_buss_status(
        OK,
        client
            .post("/")
            .multipart(file_form("upload.txt"))
            .send()
            .await
            .json()
            .await,
    );

    Ok(())
}

#[tokio::test]
async fn test_rename() -> io::Result<()> {
    let (tmp_dir, client) = setup("/*path", put(super::rename));
    let root = tmp_dir.path();

    assert_buss_status(
        NOT_FOUND,
        client
            .put("/not-found")
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
            .put("/rename-to-already-exists")
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
            .put("/rename-old-dir")
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
            .put("/rename-old-file")
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
    let (tmp_dir, client) = setup("/*path", delete(super::remove));
    let root = tmp_dir.path();

    assert_buss_status(
        NOT_FOUND,
        client.delete("/not-found").send().await.json().await,
    );

    fs::create_dir(root.join("remove-dir")).await?;
    assert_buss_status(OK, client.delete("/remove-dir").send().await.json().await);

    create_txt(root.join("remove-file"), "").await?;
    assert_buss_status(OK, client.delete("/remove-file").send().await.json().await);

    Ok(())
}

#[tokio::test]
async fn test_read_dir() -> io::Result<()> {
    let (tmp_dir, client) = setup("/*path", get(super::read_dir));

    assert_buss_status(
        NOT_FOUND,
        client.get("/not-found").send().await.json().await,
    );

    create_txt(tmp_dir.path().join("not-a-directory"), "").await?;
    assert_buss_status(
        NOT_A_DIRECTORY,
        client.get("/not-a-directory").send().await.json().await,
    );

    let read_dir = tmp_dir.path().join("read-dir");
    fs::create_dir(&read_dir).await?;
    for i in 0..5 {
        create_txt(
            read_dir.join(format!("{i}.txt")),
            &format!("the content of text {i}"),
        )
        .await?;
    }
    for i in 5..10 {
        fs::create_dir(read_dir.join(i.to_string())).await?;
    }
    assert_buss_status(OK, client.get("/read-dir").send().await.json().await);

    Ok(())
}

#[tokio::test]
async fn test_mkdir() -> io::Result<()> {
    let (tmp_dir, client) = setup("/*path", post(super::mkdir));

    assert_buss_status(
        MISSING_PARENT,
        client
            .post("/missing-parent/make-dir")
            .send()
            .await
            .json()
            .await,
    );

    fs::create_dir(tmp_dir.path().join("already-exists")).await?;
    assert_buss_status(
        ALREADY_EXISTS,
        client.post("/already-exists").send().await.json().await,
    );

    assert_buss_status(OK, client.post("/make-dir").send().await.json().await);

    Ok(())
}
