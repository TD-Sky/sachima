use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use fs_set_times::SystemTimeSpec;
use serde::Deserialize;

use crate::error::FileSysError;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Workspace(PathBuf);

impl Workspace {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Deref for Workspace {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Workspace {
    type Error = FileSysError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl FromStr for Workspace {
    type Err = FileSysError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PathBuf::from(shellexpand::tilde(s).as_ref()).try_into()
    }
}

impl TryFrom<&Path> for Workspace {
    type Error = FileSysError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        path.to_owned().try_into()
    }
}

impl TryFrom<PathBuf> for Workspace {
    type Error = FileSysError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if let Err(e) = path.read_dir() {
            return Err(FileSysError { path, source: e });
        }

        if let Err(e) = fs_set_times::set_atime(&path, SystemTimeSpec::SymbolicNow) {
            return Err(FileSysError { path, source: e });
        }

        Ok(Workspace(path))
    }
}

#[cfg(test)]
mod tests {
    use super::Workspace;
    use once_cell::sync::Lazy;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io;
    use std::path::PathBuf;

    static STATE: Lazy<PathBuf> = Lazy::new(|| {
        dirs::state_dir()
            .unwrap()
            .join("sachima/workspace-unit-tests")
    });

    #[test]
    fn test_valid() {
        let expected = STATE.join("test_valid");
        fs::create_dir_all(&expected).unwrap();

        let wk: Workspace = "~/.local/state/sachima/workspace-unit-tests/test_valid"
            .parse()
            .unwrap();
        assert_eq!(wk.0, expected);

        fs::remove_dir(&expected).unwrap();
    }

    #[test]
    fn test_non_existent() {
        let path = STATE.join("test_non_existent");
        fs::create_dir_all(&path).unwrap();
        fs::remove_dir(&path).unwrap();

        let e = Workspace::try_from(path.as_path()).unwrap_err();
        assert_eq!(e.source.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_not_a_directory() {
        fs::create_dir_all(&*STATE).unwrap();
        let path = STATE.join("test_not_a_directory");
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();

        // io::Error::NotADirectory
        assert!(Workspace::try_from(path.as_path()).is_err());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_unprivileged() {
        let e = "/home".parse::<Workspace>().unwrap_err();
        assert_eq!(e.source.kind(), io::ErrorKind::PermissionDenied);
    }
}
