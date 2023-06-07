use bytesize::ByteSize;
use serde::Serialize;
use std::cmp::Ordering;
use std::io;
use tokio::fs::DirEntry;

use crate::utils::time::to_unix_timestamp;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Directory {
    pub parent: Option<String>,
    pub entries: Vec<FsEntry>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FsEntry {
    kind: FsEntryKind,
    name: String,
    size: Option<ByteSize>,
    modified: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum FsEntryKind {
    Dir,
    File,
}

impl FsEntry {
    pub async fn dir(entry: &DirEntry) -> io::Result<Self> {
        Ok(Self {
            kind: FsEntryKind::Dir,
            name: entry.file_name().to_string_lossy().into_owned(),
            size: None,
            modified: to_unix_timestamp(entry.metadata().await?.modified()?).to_string(),
        })
    }

    pub async fn file(entry: &DirEntry) -> io::Result<Self> {
        let md = entry.metadata().await?;

        Ok(Self {
            kind: FsEntryKind::File,
            name: entry.file_name().to_string_lossy().into_owned(),
            size: Some(ByteSize(md.len())),
            modified: to_unix_timestamp(md.modified()?).to_string(),
        })
    }
}

impl Eq for FsEntry {}

impl PartialOrd for FsEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.kind
            .partial_cmp(&other.kind)
            .or_else(|| self.name.partial_cmp(&other.name))
    }
}

impl Ord for FsEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for FsEntryKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            return None;
        }

        match (self, other) {
            (Self::Dir, Self::File) => Some(Ordering::Less),
            (Self::File, Self::Dir) => Some(Ordering::Greater),
            _ => unreachable!(),
        }
    }
}
