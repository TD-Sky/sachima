use std::path::PathBuf;
use std::sync::Arc;

use bytes::Bytes;
use poem::handler;
use poem::web::Data;
use poem::web::Path;
use poem::web::Query;
use poem::Body;
use poem::Request;
use tokio::fs;
use tokio::fs::{File, OpenOptions};
use tokio::io::BufReader;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::{Stream, StreamExt};

use crate::models::fs::{Directory, FsEntry};
use crate::reply::Data as ReplyData;
use crate::reply::Error as ReplyError;
use crate::Config;

/// Protect the workspace root
pub async fn ensure_not_root(req: Request) -> poem::Result<Request> {
    let config: &Arc<Config> = req.extensions().get().unwrap();
    let path: PathBuf = req.path_params()?;

    println!("ensure_not_root");

    let path = config.workspace.join(path);
    if path == *config.workspace {
        return Err(ReplyError::WorkspaceRoot.into());
    }

    Ok(req)
}

/// **Download a file**
/// Because the mime of response is "application/octet-stream",
/// so it uses the HTTP status code to handler errors.
///
/// - Ok: return the bytes of file
/// - Err:
///   - workspace root => 403
///   - file doesn't exist => 404
///   - it's a directory, not a file => 415
///   - I/O error => 503
#[handler]
pub async fn download(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
) -> Result<Body, ReplyError> {
    let path = config.workspace.join(path);

    if path == *config.workspace {
        return Err(ReplyError::WorkspaceRoot);
    }

    if !path.exists() {
        return Err(ReplyError::NotFound);
    }

    if path.is_dir() {
        return Err(ReplyError::IsADirectory);
    }

    let fd = File::open(path).await?;

    Ok(Body::from_async_read(BufReader::new(fd)))
}

/// **Upload a file**
/// - Ok
/// - Err:
///   - parent directory doesn't exist => ReplyError::MissingParent
///   - file has already existed => ReplyError::AlreadyExists
///   - I/O Error => ReplyError::Io
#[handler]
pub async fn upload(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
    mut bytes: Bytes,
) -> Result<ReplyData<()>, ReplyError> {
    let path = config.workspace.join(path);

    if !path.parent().unwrap().exists() {
        return Err(ReplyError::MissingParent);
    }

    if path.exists() {
        return Err(ReplyError::AlreadyExists);
    }

    let mut fd = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await?;

    let mut fd = BufWriter::new(&mut fd);
    fd.write_all_buf(&mut bytes).await?;
    fd.flush().await?;

    Ok(ReplyData(()))
}

/// **Rename a file or directory**
/// - Ok
/// - Err:
///   - file doesn't exist => ReplyError::NotFound
///   - I/O error => ReplyError::Io
#[handler]
pub async fn rename(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
    Query(name): Query<String>,
) -> Result<ReplyData<()>, ReplyError> {
    let src = config.workspace.join(path);

    if !src.exists() {
        return Err(ReplyError::NotFound);
    }

    let dest = src.with_file_name(name);
    fs::rename(src, dest).await?;

    Ok(ReplyData(()))
}

/// **Remove a file or directory**
/// - Ok
/// - Err:
///   - file doesn't exist => ReplyError::NotFound
///   - I/O error => ReplyError::Io
#[handler]
pub async fn remove(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
) -> Result<ReplyData<()>, ReplyError> {
    let path = config.workspace.join(path);

    if !path.exists() {
        return Err(ReplyError::NotFound);
    }

    if path.is_dir() {
        fs::remove_dir_all(path).await?;
    } else {
        fs::remove_file(path).await?;
    }

    Ok(ReplyData(()))
}

/// **Read a directory**
/// - Ok: return the entry array
/// - Err:
///   - directory doesn't exist => ReplyError::NotFound
///   - path isn't directory => ReplyError::NotADirectory
///   - I/O error => ReplyError::Io
#[handler]
pub async fn read(
    Data(config): Data<&Arc<Config>>,
    Path(org): Path<PathBuf>,
) -> Result<ReplyData<Directory>, ReplyError> {
    let path = config.workspace.join(&org);

    if !path.exists() {
        return Err(ReplyError::NotFound);
    }

    if !path.is_dir() {
        return Err(ReplyError::NotADirectory);
    }

    let mut rd = ReadDirStream::new(fs::read_dir(&path).await?);
    let mut entries = Vec::with_capacity(rd.size_hint().0);

    while let Some(entry) = rd.next().await {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let entry = if entry.file_type().await?.is_dir() {
            FsEntry::dir(name)
        } else {
            FsEntry::file(name)
        };
        entries.push(entry);
    }
    entries.sort();

    let parent = (path != *config.workspace).then(|| org.to_string_lossy().into_owned());

    Ok(ReplyData(Directory { parent, entries }))
}

/// **Make a directory**
/// - Ok
/// - Err:
///   - parent directory doesn't exist => ReplyError::MissingParent
///   - directory has already existed => ReplyError::AlreadyExists
///   - I/O error => ReplyError::Io
#[handler]
pub async fn make(
    Data(config): Data<&Arc<Config>>,
    Path(path): Path<PathBuf>,
) -> Result<ReplyData<()>, ReplyError> {
    let path = config.workspace.join(path);

    if !path.parent().unwrap().exists() {
        return Err(ReplyError::MissingParent);
    }

    if path.exists() {
        return Err(ReplyError::AlreadyExists);
    }

    fs::create_dir(path).await?;

    Ok(ReplyData(()))
}
