use std::borrow::Cow;
use std::io;

use bytesize::ByteSize;
use poem::error::{ParseMultipartError, ResponseError};
use poem::http::StatusCode;
use poem::web::Json;
use poem::{IntoResponse, Response};
use sea_orm::DbErr;
use serde::Serialize;

use status::*;

pub type InternalError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct ReplyData<T>(pub T);

// This Error would be converted into Response directly,
// so it doesn't need to display anything
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReplyError {
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

    #[error("no such user in registry")]
    UserNotFound,

    #[error("input password is incorrect")]
    IncorrectPassword,

    #[error("a file is expected in request")]
    FileExpected,

    #[error("missing file name in Content-Disposition")]
    MissingFileName,

    #[error("uploaded resource is larger than the upper limit {0}")]
    ResourceTooLarge(ByteSize),

    #[error(transparent)]
    Internal(InternalError),
}

#[derive(Debug, Serialize)]
struct ReplyDataObject<T> {
    status: u16,
    data: T,
}

impl<T: Serialize + Send> IntoResponse for ReplyData<T> {
    fn into_response(self) -> Response {
        Json(ReplyDataObject {
            status: OK,
            data: self.0,
        })
        .into_response()
    }
}

#[derive(Debug, Serialize)]
struct ReplyErrorObject {
    status: u16,
    msg: Cow<'static, str>,
}

impl TryFrom<ReplyError> for ReplyErrorObject {
    type Error = InternalError;

    fn try_from(e: ReplyError) -> Result<Self, Self::Error> {
        Ok(match e {
            ReplyError::WorkspaceRoot => Self {
                status: WORKSPACE_ROOT,
                msg: "try to operate the workspace root".into(),
            },
            ReplyError::IsAbsolute => Self {
                status: IS_ABSOLUTE,
                msg: "path is absolute".into(),
            },
            ReplyError::AlreadyExists => Self {
                status: ALREADY_EXISTS,
                msg: "file has already existed".into(),
            },
            ReplyError::MissingParent => Self {
                status: MISSING_PARENT,
                msg: "missing parent directory".into(),
            },
            ReplyError::NotFound => Self {
                status: NOT_FOUND,
                msg: "no such file or directory".into(),
            },
            ReplyError::NotADirectory => Self {
                status: NOT_A_DIRECTORY,
                msg: "path isn't a directory".into(),
            },
            ReplyError::IsADirectory => Self {
                status: IS_A_DIRECTORY,
                msg: "path is a directory".into(),
            },
            ReplyError::UserNotFound => Self {
                status: USER_NOT_FOUND,
                msg: "no such user in registry".into(),
            },
            ReplyError::IncorrectPassword => Self {
                status: INCORRECT_PASSWORD,
                msg: "input password is incorrect".into(),
            },
            ReplyError::FileExpected => Self {
                status: FILE_EXPECTED,
                msg: "A file is expected in request".into(),
            },
            ReplyError::MissingFileName => Self {
                status: MISSING_FILE_NAME,
                msg: "missing file name in Content-Disposition".into(),
            },
            e @ ReplyError::ResourceTooLarge(_) => Self {
                status: RESOURCE_TOO_LARGE,
                msg: e.to_string().into(),
            },

            ReplyError::Internal(e) => return Err(e),
        })
    }
}

impl IntoResponse for ReplyError {
    fn into_response(self) -> Response {
        match ReplyErrorObject::try_from(self) {
            Ok(rp) => rp.into_response(),
            Err(e) => format!("{e:?}")
                .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                .into_response(),
        }
    }
}

impl IntoResponse for ReplyErrorObject {
    #[inline]
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl ResponseError for ReplyError {
    #[inline]
    fn status(&self) -> StatusCode {
        match self {
            ReplyError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
        USER_NOT_FOUND = 8,
        INCORRECT_PASSWORD = 9,
        FILE_EXPECTED = 10,
        MISSING_FILE_NAME = 11,
        RESOURCE_TOO_LARGE = 12,
    }
}

impl From<io::Error> for ReplyError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::Internal(Box::new(e))
    }
}

impl From<ParseMultipartError> for ReplyError {
    #[inline]
    fn from(e: ParseMultipartError) -> Self {
        Self::Internal(Box::new(e))
    }
}

impl From<DbErr> for ReplyError {
    #[inline]
    fn from(e: DbErr) -> Self {
        Self::Internal(Box::new(e))
    }
}
