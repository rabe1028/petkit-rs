use alloc::string::{String, ToString};

use core::fmt;

use nojson::JsonParseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetkitErrorCode {
    ServerBusy,
    SessionExpired,
    AuthenticationFailed,
    UnregisteredEmail,
    Other(i32),
}

impl PetkitErrorCode {
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::ServerBusy,
            5 => Self::SessionExpired,
            122 => Self::AuthenticationFailed,
            125 => Self::UnregisteredEmail,
            other => Self::Other(other),
        }
    }

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PetkitError {
    Api {
        code: PetkitErrorCode,
        message: String,
    },
    HttpStatus {
        status: u16,
    },
    Decode(String),
    InvalidResponse(&'static str),
    InvalidArgument(String),
    UnsupportedCommand {
        device_type: String,
        command: &'static str,
    },
}

impl PetkitError {
    pub fn api(code: i32, message: impl Into<String>) -> Self {
        Self::Api {
            code: PetkitErrorCode::from_raw(code),
            message: message.into(),
        }
    }
}

impl fmt::Display for PetkitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api { code, message } => write!(f, "{code}: {message}"),
            Self::HttpStatus { status } => write!(f, "unexpected HTTP status {status}"),
            Self::Decode(message) => write!(f, "decode error: {message}"),
            Self::InvalidResponse(message) => f.write_str(message),
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::UnsupportedCommand {
                device_type,
                command,
            } => write!(
                f,
                "unsupported command `{command}` for device `{device_type}`"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PetkitError {}

impl From<JsonParseError> for PetkitError {
    fn from(value: JsonParseError) -> Self {
        Self::Decode(value.to_string())
    }
}
