use std::path::PathBuf;
use std::sync::Arc;

use bytes::Bytes;
use poem::handler;
use poem::web::Data;
use poem::web::Path;
use poem::web::Query;
use poem::Body;
use poem::IntoResponse;
use tokio::fs;
use tokio::fs::{File, OpenOptions};
use tokio::io::BufReader;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::Config;

#[handler]
pub async fn download(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
) -> impl IntoResponse {
    let path = config.workspace.join(path);
    assert!(path.exists());

    let fd = File::open(path).await.unwrap();

    Body::from_async_read(BufReader::new(fd))
}

#[handler]
pub async fn upload(Data(config): Data<&Arc<Config>>, Path(path): Path<PathBuf>, mut bytes: Bytes) {
    let path = config.workspace.join(path);
    assert!(!path.exists());

    let mut fd = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .unwrap();

    let mut fd = BufWriter::new(&mut fd);
    fd.write_all_buf(&mut bytes).await.unwrap();
    fd.flush().await.unwrap();
}

#[handler]
pub async fn rename(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
    Query(name): Query<String>,
) {
    let src = config.workspace.join(&path);
    assert!(src.exists());
    let dest = src.with_file_name(name);

    fs::rename(src, dest).await.unwrap();
}

#[handler]
pub async fn remove(Data(config): Data<&Arc<Config>>, Path(path): Path<PathBuf>) {
    let path = config.workspace.join(path);
    assert!(path.exists());

    if path.is_dir() {
        fs::remove_dir_all(path).await.unwrap();
    } else {
        fs::remove_file(path).await.unwrap();
    }
}
