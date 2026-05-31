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

fn media_payload<'text, 'raw>(
    value: RawJsonValue<'text, 'raw>,
) -> Result<RawJsonValue<'text, 'raw>, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return Ok(value);
    }
    for name in [
        "data",
        "media",
        "video",
        "cloudVideo",
        "cloud_video",
        "record",
    ] {
        if let Some(member) = value.to_member(name)?.optional() {
            if matches!(member.kind(), JsonValueKind::Array | JsonValueKind::Object) {
                return Ok(member);
            }
        }
    }
    Ok(value)
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
        let media_type =
            optional_text_any(value, &["mediaType", "media_type", "type"])?.map(MediaType::from);
        let generic_url = optional_text_any(value, &["url"])?;
        let mut image_url = optional_text_any(
            value,
            &[
                "image",
                "imageUrl",
                "image_url",
                "img",
                "imgUrl",
                "img_url",
                "preview",
                "preview1",
                "preview2",
                "thumbnail",
                "thumbUrl",
                "shitPicture",
                "picUrl",
            ],
        )?;
        let mut video_url = optional_text_any(value, &["video", "videoUrl", "video_url"])?;
        match media_type.as_ref() {
            Some(MediaType::Video) => {
                video_url = video_url.or(generic_url);
            }
            Some(MediaType::Image) => {
                image_url = image_url.or(generic_url);
            }
            _ => {
                image_url = image_url.or(generic_url);
            }
        }

        Ok(Self {
            event_id: optional_text_any(value, &["eventId", "event_id", "id", "picId"])?,
            event_type: optional_event_type(value)?,
            media_type,
            device_id: optional_u64_any(value, &["deviceId", "device_id"])?,
            user_id: optional_u64_any(value, &["userId", "user_id"])?,
            image_url,
            video_url,
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
                    "eventTime",
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

impl MediaListResponse {
    pub fn latest_image(&self) -> Option<&MediaMetadata> {
        latest_image_metadata(&self.items)
    }

    pub fn latest_video(&self) -> Option<&MediaMetadata> {
        latest_video_metadata(&self.items)
    }
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for MediaListResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let payload = media_payload(value)?;
        let items = match payload.kind() {
            JsonValueKind::Array => parse_media_array(payload)?,
            JsonValueKind::Object => {
                if let Some(array) = first_array_member(payload, &["items", "list", "records"])? {
                    parse_media_array(array)?
                } else {
                    Vec::from([MediaMetadata::try_from(payload)?])
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
    pub video_url: Option<String>,
    pub image_url: Option<String>,
    pub aes_key: Option<String>,
    pub timestamp: Option<u64>,
    pub expires_at: Option<u64>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub duration: Option<u64>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudVideoResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let payload = media_payload(value)?;
        let metadata = MediaMetadata::try_from(payload)?;
        Ok(Self {
            media_api: metadata.media_api,
            video_url: metadata.video_url,
            image_url: metadata.image_url,
            aes_key: metadata.aes_key,
            timestamp: metadata.timestamp,
            expires_at: optional_u64_any(payload, &["expiresAt", "expires_at", "expireTime"])?,
            start_time: optional_u64_any(payload, &["startTime", "start_time"])?,
            end_time: optional_u64_any(payload, &["endTime", "end_time"])?,
            duration: optional_u64_any(payload, &["duration", "durationSeconds"])?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M3u8Response {
    pub url: Option<String>,
    pub media_api: Option<String>,
    pub download_url: Option<String>,
    pub aes_key: Option<String>,
    pub expires_at: Option<u64>,
}

impl M3u8Response {
    pub fn primary_url(&self) -> Option<&str> {
        self.url
            .as_deref()
            .or(self.media_api.as_deref())
            .or(self.download_url.as_deref())
    }
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for M3u8Response {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let payload = media_payload(value)?;
        Ok(Self {
            url: optional_text_any(
                payload,
                &["m3u8", "m3u8Url", "m3u8_url", "playUrl", "play_url", "url"],
            )?,
            media_api: optional_text_any(payload, &["mediaApi", "media_api"])?,
            download_url: optional_text_any(
                payload,
                &[
                    "downloadUrl",
                    "download_url",
                    "downloadM3u8",
                    "download_m3u8",
                ],
            )?,
            aes_key: optional_text_any(payload, &["aesKey", "aes_key"])?,
            expires_at: optional_u64_any(payload, &["expiresAt", "expires_at", "expireTime"])?,
        })
    }
}

pub type GetM3u8Response = M3u8Response;
pub type GetDownloadM3u8Response = M3u8Response;

pub fn latest_image_metadata(items: &[MediaMetadata]) -> Option<&MediaMetadata> {
    items
        .iter()
        .filter(|item| item.has_image())
        .max_by_key(|item| item.timestamp.unwrap_or(0))
}

pub fn latest_video_metadata(items: &[MediaMetadata]) -> Option<&MediaMetadata> {
    items
        .iter()
        .filter(|item| item.has_video())
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
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
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
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
    for name in names {
        match value.to_member(name)?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => {}
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
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
    for name in names {
        match value.to_member(name)?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => {}
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

    const CLOUD_VIDEO_FIXTURE: &str = r#"{"result":{"data":{"mediaApi":"https://media.example/redacted/cloud.m3u8","aesKey":"aes-redacted","expiresAt":1893456000,"startTime":1770000000,"endTime":1770000060,"duration":60}}}"#;
    const MEDIA_LIST_FIXTURE: &str = r#"{"result":{"data":{"records":[{"eventId":"evt-old","enumEventType":"feed","deviceId":"7","imageUrl":"https://media.example/redacted/old.jpg","aesKey":"aes-old-redacted","eventTime":1770000000},{"eventId":"evt-new","enumEventType":"eat","deviceId":7,"preview":"https://media.example/redacted/new.jpg","videoUrl":"https://media.example/redacted/new.mp4","aesKey":"aes-new-redacted","eventTime":1770000100}]}}}"#;
    const M3U8_FIXTURE: &str = r#"{"result":{"data":{"m3u8Url":"https://media.example/redacted/live.m3u8","aesKey":"aes-live-redacted","expiresAt":"1893456000"}}}"#;
    const DOWNLOAD_M3U8_FIXTURE: &str = r#"{"result":{"data":{"downloadUrl":"https://media.example/redacted/download.m3u8","aesKey":"aes-download-redacted","expireTime":1893456060}}}"#;

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
    fn media_list_response_parses_sanitized_petkit_fixture() {
        let response = with_result(MEDIA_LIST_FIXTURE, |value| {
            MediaListResponse::try_from(value).expect("media list should parse")
        });
        assert_eq!(response.items.len(), 2);

        let latest = response.latest_image().expect("latest image should exist");
        assert_eq!(latest.event_id.as_deref(), Some("evt-new"));
        assert_eq!(latest.event_type, Some(MediaEventType::Eat));
        assert_eq!(
            latest.image_url.as_deref(),
            Some("https://media.example/redacted/new.jpg")
        );
        assert_eq!(
            latest.video_url.as_deref(),
            Some("https://media.example/redacted/new.mp4")
        );
        assert_eq!(latest.aes_key.as_deref(), Some("aes-new-redacted"));
        assert_eq!(latest.timestamp, Some(1_770_000_100));

        let latest_video = response.latest_video().expect("latest video should exist");
        assert_eq!(latest_video.event_id.as_deref(), Some("evt-new"));
        assert_eq!(
            latest_video.video_url.as_deref(),
            Some("https://media.example/redacted/new.mp4")
        );
    }

    #[test]
    fn media_aliases_skip_null_values() {
        let response = with_result(
            r#"{"result":{"items":[{"eventId":"evt-null","preview1":null,"preview2":"https://media.example/redacted/fallback.jpg","completedAt":null,"eventTime":"1770000200"}]}}"#,
            |value| MediaListResponse::try_from(value).expect("media list should parse"),
        );

        let latest = response
            .latest_image()
            .expect("fallback image should exist");
        assert_eq!(
            latest.image_url.as_deref(),
            Some("https://media.example/redacted/fallback.jpg")
        );
        assert_eq!(latest.timestamp, Some(1_770_000_200));
    }

    #[test]
    fn media_list_response_unwraps_array_data_payload() {
        let response = with_result(
            r#"{"result":{"data":[{"eventId":"evt-array","imageUrl":"https://media.example/redacted/array.jpg","eventTime":1770000250}]}}"#,
            |value| MediaListResponse::try_from(value).expect("media list should parse"),
        );

        assert_eq!(response.items.len(), 1);
        let latest = response.latest_image().expect("array image should exist");
        assert_eq!(
            latest.image_url.as_deref(),
            Some("https://media.example/redacted/array.jpg")
        );
    }

    #[test]
    fn generic_video_url_is_not_classified_as_image() {
        let response = with_result(
            r#"{"result":{"items":[{"eventId":"evt-video","mediaType":"mp4","url":"https://media.example/redacted/video.mp4","eventTime":1770000300}]}}"#,
            |value| MediaListResponse::try_from(value).expect("media list should parse"),
        );

        assert!(response.latest_image().is_none());
        let latest = response.latest_video().expect("video should exist");
        assert_eq!(
            latest.video_url.as_deref(),
            Some("https://media.example/redacted/video.mp4")
        );
        assert_eq!(latest.media_type, Some(MediaType::Video));
    }

    #[test]
    fn cloud_video_response_parses_media_api() {
        let response = with_result(CLOUD_VIDEO_FIXTURE, |value| {
            CloudVideoResponse::try_from(value).expect("cloud video should parse")
        });
        assert_eq!(
            response.media_api.as_deref(),
            Some("https://media.example/redacted/cloud.m3u8")
        );
        assert_eq!(response.aes_key.as_deref(), Some("aes-redacted"));
        assert_eq!(response.expires_at, Some(1_893_456_000));
        assert_eq!(response.start_time, Some(1_770_000_000));
        assert_eq!(response.end_time, Some(1_770_000_060));
        assert_eq!(response.duration, Some(60));
    }

    #[test]
    fn m3u8_responses_parse_live_and_download_fixtures() {
        let response = with_result(M3U8_FIXTURE, |value| {
            GetM3u8Response::try_from(value).expect("m3u8 response should parse")
        });
        assert_eq!(
            response.primary_url(),
            Some("https://media.example/redacted/live.m3u8")
        );
        assert_eq!(response.aes_key.as_deref(), Some("aes-live-redacted"));
        assert_eq!(response.expires_at, Some(1_893_456_000));

        let response = with_result(DOWNLOAD_M3U8_FIXTURE, |value| {
            GetDownloadM3u8Response::try_from(value).expect("download m3u8 response should parse")
        });
        assert_eq!(
            response.primary_url(),
            Some("https://media.example/redacted/download.m3u8")
        );
        assert_eq!(
            response.download_url.as_deref(),
            Some("https://media.example/redacted/download.m3u8")
        );
    }
}
