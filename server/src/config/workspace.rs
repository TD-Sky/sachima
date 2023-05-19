use crate::error::FileSysError;
use fs_set_times::SystemTimeSpec;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Workspace(PathBuf);

impl TryFrom<String> for Workspace {
    type Error = FileSysError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let path = PathBuf::from(shellexpand::tilde(&s).as_ref());

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
    use std::env;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io;
    use std::path::PathBuf;

    static STATE: Lazy<PathBuf> =
        Lazy::new(|| PathBuf::from(env::var_os("HOME").unwrap()).join(".local/state/sachima"));

    #[test]
    fn test_valid() {
        let expected = STATE.join("test_valid");
        fs::create_dir_all(&expected).unwrap();

        let wk = Workspace::try_from("~/.local/state/sachima/test_valid".to_owned()).unwrap();
        assert_eq!(wk, Workspace(expected.clone()));

        fs::remove_dir(&expected).unwrap();
    }

    #[test]
    fn test_non_existent() {
        let path = STATE.join("test_non_existent");
        fs::create_dir_all(&path).unwrap();
        fs::remove_dir(&path).unwrap();

        let e = Workspace::try_from(path.to_string_lossy().into_owned()).unwrap_err();
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
        assert!(Workspace::try_from(path.to_string_lossy().into_owned()).is_err());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_unprivileged() {
        let e = Workspace::try_from("/home".to_owned()).unwrap_err();
        assert_eq!(e.source.kind(), io::ErrorKind::PermissionDenied);
    }
}
