mod workspace;
pub use workspace::Workspace;

pub mod log_level;
pub use log_level::LogLevel;

use bytesize::ByteSize;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// 服务器部署端口
    pub port: u16,
    /// poem的日志等级
    pub poem_level: Option<LogLevel>,
    /// 服务的目录
    pub workspace: Workspace,
    /// 可上传单文件的最大数据量
    pub max_upload: ByteSize,
}

#[cfg(test)]
mod tests {
    use super::Config;
    use super::LogLevel;
    use super::Workspace;
    use bytesize::ByteSize;
    use indoc::indoc;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_valid() {
        fs::create_dir_all(
            PathBuf::from(env::var_os("HOME").unwrap()).join(".local/state/sachima"),
        )
        .unwrap();

        let expected = Config {
            port: 8000,
            poem_level: Some(LogLevel::Info),
            workspace: Workspace::try_from("~/.local/state/sachima".to_owned()).unwrap(),
            max_upload: ByteSize::gb(2),
        };

        let input = indoc! {r#"
            port = 8000
            poem_level = "info"
            workspace = "~/.local/state/sachima"
            max_upload = "2G"
        "#};

        let actual: Config = toml::from_str(input).unwrap();
        assert_eq!(actual, expected);
    }
}
