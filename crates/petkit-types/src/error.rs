use alloc::string::{String, ToString};

use core::fmt;

use nojson::JsonParseError;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PetkitErrorCode {
    ServerBusy,
    SessionExpired,
    AuthenticationFailed,
    UnregisteredEmail,
    Other(i32),
}

impl PetkitErrorCode {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::ServerBusy,
            5 => Self::SessionExpired,
            122 => Self::AuthenticationFailed,
            125 => Self::UnregisteredEmail,
            other => Self::Other(other),
        }
    }

    #[must_use]
    pub const fn raw(self) -> i32 {
        match self {
            Self::ServerBusy => 1,
            Self::SessionExpired => 5,
            Self::AuthenticationFailed => 122,
            Self::UnregisteredEmail => 125,
            Self::Other(value) => value,
        }
    }
}

impl fmt::Display for PetkitErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ServerBusy => f.write_str("server busy"),
            Self::SessionExpired => f.write_str("session expired"),
            Self::AuthenticationFailed => f.write_str("authentication failed"),
            Self::UnregisteredEmail => f.write_str("unregistered email"),
            Self::Other(code) => write!(f, "unknown error code {code}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PetkitError {
    #[error("{code}: {message}")]
    Api {
        code: PetkitErrorCode,
        message: String,
    },
    #[error("unexpected HTTP status {status}")]
    HttpStatus { status: u16 },
    #[error("decode error: {0}")]
    Decode(String),
    #[error("{0}")]
    InvalidResponse(&'static str),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("unsupported command `{command}` for device `{device_type}`")]
    UnsupportedCommand {
        device_type: String,
        command: &'static str,
    },
}

impl PetkitError {
    #[must_use]
    pub fn api(code: i32, message: impl Into<String>) -> Self {
        Self::Api {
            code: PetkitErrorCode::from_raw(code),
            message: message.into(),
        }
    }
}

impl From<JsonParseError> for PetkitError {
    fn from(value: JsonParseError) -> Self {
        Self::Decode(value.to_string())
    }
}
