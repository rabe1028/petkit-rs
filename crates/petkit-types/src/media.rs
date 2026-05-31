use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use nojson::{JsonParseError, JsonValueKind, RawJsonValue};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MediaType {
    Image,
    Video,
    Unknown(String),
}

impl MediaType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Image => "jpg",
            Self::Video => "mp4",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl From<String> for MediaType {
    fn from(value: String) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" | "image" | "snapshot" => Self::Image,
            "mp4" | "video" | "playback" | "highlight" => Self::Video,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MediaEventType {
    Eat,
    Feed,
    Move,
    Pet,
    Toileting,
    WasteCheck,
    DishBefore,
    DishAfter,
    Unknown(String),
}

impl MediaEventType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Eat => "eat",
            Self::Feed => "feed",
            Self::Move => "move",
            Self::Pet => "pet",
            Self::Toileting => "toileting",
            Self::WasteCheck => "waste_check",
            Self::DishBefore => "dish_before",
            Self::DishAfter => "dish_after",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl From<String> for MediaEventType {
    fn from(value: String) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "eat" => Self::Eat,
            "feed" => Self::Feed,
            "move" => Self::Move,
            "pet" | "pet_detect" => Self::Pet,
            "toileting" | "toilet" | "toilet_detection" => Self::Toileting,
            "waste_check" | "waste-check" => Self::WasteCheck,
            "dish_before" | "dish-before" => Self::DishBefore,
            "dish_after" | "dish-after" => Self::DishAfter,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaMetadata {
    pub event_id: Option<String>,
    pub event_type: Option<MediaEventType>,
    pub media_type: Option<MediaType>,
    pub device_id: Option<u64>,
    pub user_id: Option<u64>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub media_api: Option<String>,
    pub aes_key: Option<String>,
    pub timestamp: Option<u64>,
}

impl MediaMetadata {
    pub fn has_image(&self) -> bool {
        self.image_url
            .as_ref()
            .is_some_and(|value| !value.is_empty())
    }

    pub fn has_video(&self) -> bool {
        self.video_url
            .as_ref()
            .is_some_and(|value| !value.is_empty())
            || self
                .media_api
                .as_ref()
                .is_some_and(|value| !value.is_empty())
    }
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for MediaMetadata {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            event_id: optional_text_any(value, &["eventId", "event_id", "id", "picId"])?,
            event_type: optional_event_type(value)?,
            media_type: optional_text_any(value, &["mediaType", "media_type", "type"])?
                .map(MediaType::from),
            device_id: optional_u64_any(value, &["deviceId", "device_id"])?,
            user_id: optional_u64_any(value, &["userId", "user_id"])?,
            image_url: optional_text_any(
                value,
                &[
                    "image",
                    "img",
                    "preview",
                    "preview1",
                    "preview2",
                    "shitPicture",
                    "url",
                ],
            )?,
            video_url: optional_text_any(value, &["video", "videoUrl", "video_url"])?,
            media_api: optional_text_any(value, &["mediaApi", "media_api"])?,
            aes_key: optional_text_any(value, &["aesKey", "aes_key", "shitAesKey"])?,
            timestamp: optional_u64_any(
                value,
                &[
                    "timestamp",
                    "completedAt",
                    "eatStartTime",
                    "eatEndTime",
                    "startTime",
                    "endTime",
                    "time",
                    "createdAt",
                ],
            )?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaListResponse {
    pub items: Vec<MediaMetadata>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for MediaListResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let items = match value.kind() {
            JsonValueKind::Array => parse_media_array(value)?,
            JsonValueKind::Object => {
                if let Some(array) = first_array_member(value, &["items", "list", "records"])? {
                    parse_media_array(array)?
                } else {
                    Vec::from([MediaMetadata::try_from(value)?])
                }
            }
            _ => Vec::new(),
        };
        Ok(Self { items })
    }
}

impl From<MediaListResponse> for Vec<MediaMetadata> {
    fn from(value: MediaListResponse) -> Self {
        value.items
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudVideoResponse {
    pub media_api: Option<String>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudVideoResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            media_api: optional_text_any(value, &["mediaApi", "media_api"])?,
        })
    }
}

pub fn latest_image_metadata(items: &[MediaMetadata]) -> Option<&MediaMetadata> {
    items
        .iter()
        .filter(|item| item.has_image())
        .max_by_key(|item| item.timestamp.unwrap_or(0))
}

fn parse_media_array(value: RawJsonValue<'_, '_>) -> Result<Vec<MediaMetadata>, JsonParseError> {
    value
        .to_array()?
        .map(MediaMetadata::try_from)
        .collect::<Result<Vec<_>, _>>()
}

fn first_array_member<'text, 'raw>(
    value: RawJsonValue<'text, 'raw>,
    names: &[&str],
) -> Result<Option<RawJsonValue<'text, 'raw>>, JsonParseError> {
    for name in names {
        if let Some(member) = value.to_member(name)?.optional() {
            if member.kind() == JsonValueKind::Array {
                return Ok(Some(member));
            }
        }
    }
    Ok(None)
}

fn optional_event_type(
    value: RawJsonValue<'_, '_>,
) -> Result<Option<MediaEventType>, JsonParseError> {
    if let Some(event_type) = optional_text_any(value, &["enumEventType", "eventTypeName"])? {
        return Ok(Some(MediaEventType::from(event_type)));
    }
    optional_text_any(value, &["eventType", "event_type"])
        .map(|value| value.map(MediaEventType::from))
}

fn optional_text_any(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
) -> Result<Option<String>, JsonParseError> {
    for name in names {
        match value.to_member(name)?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => return Ok(None),
            Some(member) => return text_value(member).map(Some),
            None => {}
        }
    }
    Ok(None)
}

fn optional_u64_any(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
) -> Result<Option<u64>, JsonParseError> {
    for name in names {
        match value.to_member(name)?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => return Ok(None),
            Some(member) if member.kind() == JsonValueKind::String => {
                let raw: String = member.try_into()?;
                return raw
                    .parse::<u64>()
                    .map(Some)
                    .map_err(|error| member.invalid(error));
            }
            Some(member) => return u64::try_from(member).map(Some),
            None => {}
        }
    }
    Ok(None)
}

fn text_value(value: RawJsonValue<'_, '_>) -> Result<String, JsonParseError> {
    match value.kind() {
        JsonValueKind::String => value.try_into(),
        JsonValueKind::Integer | JsonValueKind::Float => Ok(value.as_number_str()?.to_owned()),
        JsonValueKind::Boolean => Ok(bool::try_from(value)?.to_string()),
        JsonValueKind::Null => Ok(String::new()),
        JsonValueKind::Array | JsonValueKind::Object => Ok(value.as_raw_str().to_owned()),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use nojson::RawJson;

    use super::*;

    fn with_result<T>(text: &str, parse: impl FnOnce(RawJsonValue<'_, '_>) -> T) -> T {
        let raw = RawJson::parse(text).expect("json should parse");
        let value = raw
            .value()
            .to_member("result")
            .expect("result lookup should parse")
            .required()
            .expect("result should exist");
        parse(value)
    }

    #[test]
    fn media_list_response_parses_latest_image_metadata() {
        let response = with_result(
            r#"{"result":{"items":[{"eventId":"older","enumEventType":"feed","deviceId":7,"preview":"https://example/old.jpg","aesKey":"k","timestamp":100},{"eventId":"newer","enumEventType":"eat","deviceId":7,"preview":"https://example/new.jpg","aesKey":"k","timestamp":200}]}}"#,
            |value| MediaListResponse::try_from(value).expect("media list should parse"),
        );

        let latest = latest_image_metadata(&response.items).expect("latest image should exist");
        assert_eq!(latest.event_id.as_deref(), Some("newer"));
        assert_eq!(latest.event_type, Some(MediaEventType::Eat));
    }

    #[test]
    fn cloud_video_response_parses_media_api() {
        let response = with_result(
            r#"{"result":{"mediaApi":"https://example/video.m3u8"}}"#,
            |value| CloudVideoResponse::try_from(value).expect("cloud video should parse"),
        );

        assert_eq!(
            response.media_api.as_deref(),
            Some("https://example/video.m3u8")
        );
    }
}
