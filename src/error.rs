use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("{path:?}: {source}")]
pub struct FileSysError {
    pub path: PathBuf,
    #[source]
    pub source: io::Error,
}
