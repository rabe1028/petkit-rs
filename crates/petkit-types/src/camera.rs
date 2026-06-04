use alloc::format;
use alloc::string::String;

use nojson::{JsonParseError, JsonValueKind, RawJsonValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PtzKind {
    Move,
    Stop,
    Custom(i64),
}

impl PtzKind {
    pub const fn wire_value(self) -> i64 {
        match self {
            Self::Move => 0,
            Self::Stop => 1,
            Self::Custom(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PtzDirection {
    Up,
    Down,
    Left,
    Right,
    Stop,
    Custom(i64),
}

impl PtzDirection {
    pub const fn wire_value(self) -> i64 {
        match self {
            Self::Up => 0,
            Self::Down => 1,
            Self::Left => 2,
            Self::Right => 3,
            Self::Stop => 4,
            Self::Custom(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CameraRtmCommand {
    StartLive {
        is_sd: bool,
    },
    StopLive,
    PtzControl {
        kind: PtzKind,
        direction: PtzDirection,
    },
    Heartbeat,
}

impl CameraRtmCommand {
    pub fn payload_json(&self) -> String {
        match self {
            Self::StartLive { is_sd } => {
                format!(r#"{{"cmd":"start_live","payload":{{"is_sd":{is_sd}}}}}"#)
            }
            Self::StopLive => String::from(r#"{"cmd":"stop_live"}"#),
            Self::PtzControl { kind, direction } => format!(
                r#"{{"cmd":"ptz_ctrl","payload":{{"type":{},"ptz_dir":{}}}}}"#,
                kind.wire_value(),
                direction.wire_value()
            ),
            Self::Heartbeat => String::from(r#"{"cmd":"heartbeat"}"#),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgoraRtmResponse {
    pub accepted: bool,
    pub request_id: Option<String>,
    pub message_id: Option<String>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for AgoraRtmResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            accepted: agora_truthy(value)?,
            request_id: optional_string_any(value, &["request_id", "requestId"])?,
            message_id: optional_string_any(value, &["message_id", "messageId"])?,
        })
    }
}

pub fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch < ' ' => {
                escaped.push_str("\\u");
                escaped.push_str(&format!("{:04x}", ch as u32));
            }
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

fn optional_string_any(
    value: RawJsonValue<'_, '_>,
    keys: &[&'static str],
) -> Result<Option<String>, JsonParseError> {
    for key in keys {
        match value.to_member(key)?.optional() {
            Some(value) if value.kind() == JsonValueKind::Null => {}
            Some(value) => return String::try_from(value).map(Some),
            None => {}
        }
    }
    Ok(None)
}

fn agora_truthy(value: RawJsonValue<'_, '_>) -> Result<bool, JsonParseError> {
    match value.kind() {
        JsonValueKind::Boolean => bool::try_from(value),
        JsonValueKind::Integer | JsonValueKind::Float => Ok(i64::try_from(value)? != 0),
        JsonValueKind::String => {
            let raw = String::try_from(value)?;
            Ok(matches!(raw.as_str(), "1" | "true" | "ok" | "success"))
        }
        JsonValueKind::Object => {
            if let Some(value) = value.to_member("code")?.optional() {
                return Ok(i64::try_from(value)? == 0);
            }
            if let Some(value) = value.to_member("success")?.optional() {
                return bool::try_from(value).or_else(|_| i64::try_from(value).map(|v| v != 0));
            }
            Ok(true)
        }
        JsonValueKind::Array | JsonValueKind::Null => Ok(false),
    }
}
