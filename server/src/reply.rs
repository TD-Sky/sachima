use std::io;

use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::web::Json;
use poem::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug)]
pub struct Data<T>(pub T);

// This Error would be converted into Response directly,
// so it doesn't need to display anything
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("try to operate the workspace root")]
    WorkspaceRoot,

    #[error("file has already existed")]
    AlreadyExists,

    #[error("missing parent directory")]
    MissingParent,

    #[error("no such file or directory")]
    NotFound,

    #[error("path isn't a directory")]
    NotADirectory,

    #[error("path is a directory")]
    IsADirectory,

    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Serialize)]
struct ReplyData<T> {
    status: u16,
    data: T,
}

#[derive(Debug, Serialize)]
pub struct ReplyError {
    status: u16,
    msg: &'static str,
}

impl<T: Serialize + Send> IntoResponse for Data<T> {
    fn into_response(self) -> Response {
        Json(ReplyData {
            status: 0,
            data: self.0,
        })
        .into_response()
    }
}

impl TryFrom<Error> for ReplyError {
    type Error = io::Error;

    fn try_from(e: Error) -> Result<Self, Self::Error> {
        Ok(match e {
            Error::WorkspaceRoot => Self {
                status: 1,
                msg: "try to operate the workspace root",
            },
            Error::AlreadyExists => Self {
                status: 2,
                msg: "file has already existed",
            },
            Error::MissingParent => Self {
                status: 3,
                msg: "missing parent directory",
            },
            Error::NotFound => Self {
                status: 4,
                msg: "no such file or directory",
            },
            Error::NotADirectory => Self {
                status: 5,
                msg: "path isn't a directory",
            },
            Error::IsADirectory => Self {
                status: 6,
                msg: "path is a directory",
            },

            Error::Io(e) => return Err(e),
        })
    }
}

impl IntoResponse for ReplyError {
    #[inline]
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl ResponseError for Error {
    #[inline]
    fn status(&self) -> StatusCode {
        match self {
            Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::OK,
        }
    }
}
