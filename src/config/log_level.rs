use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unsupported level {0}")]
pub struct InvalidLevel(String);

impl TryFrom<String> for LogLevel {
    type Error = InvalidLevel;

    fn try_from(s: String) -> Result<Self, InvalidLevel> {
        Ok(match s.as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warn" => Self::Warn,
            "error" => Self::Error,
            _ => return Err(InvalidLevel(s)),
        })
    }
}
