use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Directory {
    pub parent: Option<String>,
    pub entries: Vec<FsEntry>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FsEntry {
    name: String,
    kind: FsEntryKind,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum FsEntryKind {
    Dir,
    File,
}

impl FsEntry {
    #[inline]
    pub fn dir(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: FsEntryKind::Dir,
        }
    }

    #[inline]
    pub fn file(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: FsEntryKind::File,
        }
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

#[cfg(test)]
mod tests {
    use super::FsEntry;

    #[test]
    fn test_entry_ord() {
        assert!(FsEntry::file("/a/b/c") > FsEntry::dir("/e/f/g"));
        assert_eq!(FsEntry::file("/a/b/c"), FsEntry::file("/a/b/c"));
    }
}
