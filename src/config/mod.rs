mod workspace;
pub use workspace::Workspace;

pub mod log_level;
pub use log_level::LogLevel;

use bytesize::ByteSize;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The deploying port
    pub port: u16,

    /// The database url
    pub database_url: String,

    /// Log level of poem
    pub poem_log_level: Option<LogLevel>,

    /// The serving directory
    pub workspace: Workspace,

    /// The max data size of single file
    pub max_upload: ByteSize,

    /// The hash secret key for JWT
    pub jwt_secret_key: String,

    /// The hash salt of user password
    pub password_salt: String,
}
