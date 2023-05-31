use std::io;

use poem::error::{InternalServerError, ResponseError};
use poem::http::StatusCode;
use poem::web::Json;
use poem::{IntoResponse, Response};
use serde::Serialize;

use status::*;

#[derive(Debug)]
pub struct Data<T>(pub T);

// This Error would be converted into Response directly,
// so it doesn't need to display anything
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("try to operate the workspace root")]
    WorkspaceRoot,

    #[error("path is absolute")]
    IsAbsolute,

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

impl<T: Serialize + Send> IntoResponse for Data<T> {
    fn into_response(self) -> Response {
        Json(ReplyData {
            status: OK,
            data: self.0,
        })
        .into_response()
    }
}

#[derive(Debug, Serialize)]
struct ReplyError {
    status: u16,
    msg: &'static str,
}

impl TryFrom<Error> for ReplyError {
    type Error = io::Error;

    fn try_from(e: Error) -> Result<Self, Self::Error> {
        Ok(match e {
            Error::WorkspaceRoot => Self {
                status: WORKSPACE_ROOT,
                msg: "try to operate the workspace root",
            },
            Error::IsAbsolute => Self {
                status: IS_ABSOLUTE,
                msg: "path is absolute",
            },
            Error::AlreadyExists => Self {
                status: ALREADY_EXISTS,
                msg: "file has already existed",
            },
            Error::MissingParent => Self {
                status: MISSING_PARENT,
                msg: "missing parent directory",
            },
            Error::NotFound => Self {
                status: NOT_FOUND,
                msg: "no such file or directory",
            },
            Error::NotADirectory => Self {
                status: NOT_A_DIRECTORY,
                msg: "path isn't a directory",
            },
            Error::IsADirectory => Self {
                status: IS_A_DIRECTORY,
                msg: "path is a directory",
            },

            Error::Io(e) => return Err(e),
        })
    }
}

impl IntoResponse for Error {
    #[inline]
    fn into_response(self) -> Response {
        match ReplyError::try_from(self) {
            Ok(rp) => rp.into_response(),
            Err(e) => InternalServerError(e).into_response(),
        }
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

pub mod status {
    macro_rules! reply_status_code {
    ($($status: ident = $code: expr),*,) => {
        $(pub const $status: u16 = $code;)*
    };
}

    reply_status_code! {
        OK = 0,
        WORKSPACE_ROOT = 1,
        IS_ABSOLUTE = 2,
        ALREADY_EXISTS = 3,
        MISSING_PARENT = 4,
        NOT_FOUND = 5,
        NOT_A_DIRECTORY = 6,
        IS_A_DIRECTORY = 7,
    }
}
