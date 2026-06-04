use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use core::time::Duration;

use nojson::{JsonParseError, JsonValueKind, RawJsonValue};

use crate::{DeviceDetailResponse, DeviceSummary, PetkitError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudBleMetadata {
    pub device_type: String,
    pub mac: String,
    pub group_id: Option<String>,
    pub ble_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloudBleRelayOptions {
    pub poll_interval: Duration,
    pub max_polls: u8,
}

impl CloudBleRelayOptions {
    pub const fn new(poll_interval: Duration, max_polls: u8) -> Self {
        Self {
            poll_interval,
            max_polls,
        }
    }
}

impl Default for CloudBleRelayOptions {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(4),
            max_polls: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudBleDevice {
    pub id: String,
    pub mac: String,
    pub name: Option<String>,
    pub sn: Option<String>,
    pub pim: Option<i64>,
    pub type_id: Option<u64>,
    pub low_version: Option<u64>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudBleDevice {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: required_string_any(value, &["id", "bleId", "ble_id"])?,
            mac: required_string_any(value, &["mac", "btMac", "bt_mac"])?,
            name: optional_string_any(value, &["name", "deviceName"])?,
            sn: optional_string_any(value, &["sn"])?,
            pim: optional_i64_any(value, &["pim"])?,
            type_id: optional_u64_any(value, &["typeId", "type", "type_id"])?,
            low_version: optional_u64_any(value, &["lowVersion", "low_version"])?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudBleDevicesResponse {
    pub devices: Vec<CloudBleDevice>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudBleDevicesResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let devices = match value.kind() {
            JsonValueKind::Array => value
                .to_array()?
                .map(CloudBleDevice::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            JsonValueKind::Object => {
                let devices = value
                    .to_member("devices")?
                    .optional()
                    .or_else(|| value.to_member("list").ok()?.optional());
                match devices {
                    Some(devices) => devices
                        .to_array()?
                        .map(CloudBleDevice::try_from)
                        .collect::<Result<Vec<_>, _>>()?,
                    None => Vec::new(),
                }
            }
            _ => Vec::new(),
        };
        Ok(Self { devices })
    }
}

impl From<CloudBleDevicesResponse> for Vec<CloudBleDevice> {
    fn from(value: CloudBleDevicesResponse) -> Self {
        value.devices
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudBleConnectRequest {
    pub ble_id: String,
    pub device_type: String,
    pub mac: String,
    pub group_id: Option<String>,
}

impl CloudBleConnectRequest {
    pub fn new(
        ble_id: impl Into<String>,
        device_type: impl Into<String>,
        mac: impl Into<String>,
    ) -> Self {
        Self {
            ble_id: ble_id.into(),
            device_type: device_type.into(),
            mac: mac.into(),
            group_id: None,
        }
    }

    pub fn with_group_id(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = Some(group_id.into());
        self
    }

    pub fn from_metadata(
        metadata: &CloudBleMetadata,
        fallback_ble_id: impl Into<String>,
    ) -> Result<Self, PetkitError> {
        let ble_id = metadata
            .ble_id
            .clone()
            .unwrap_or_else(|| fallback_ble_id.into());
        if ble_id.trim().is_empty() {
            return Err(PetkitError::InvalidArgument(String::from(
                "cloud BLE request requires ble_id",
            )));
        }
        if metadata.device_type.trim().is_empty() {
            return Err(PetkitError::InvalidArgument(String::from(
                "cloud BLE request requires device_type",
            )));
        }
        if metadata.mac.trim().is_empty() {
            return Err(PetkitError::InvalidArgument(String::from(
                "cloud BLE request requires mac",
            )));
        }
        Ok(Self {
            ble_id,
            device_type: metadata.device_type.clone(),
            mac: metadata.mac.clone(),
            group_id: metadata.group_id.clone(),
        })
    }
}

pub type CloudBlePollRequest = CloudBleConnectRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudBleControlRequest {
    pub device_id: String,
    pub ble_id: Option<String>,
    pub device_type: String,
    pub mac: String,
    pub cmd: String,
    pub data: String,
    pub group_id: Option<String>,
}

impl CloudBleControlRequest {
    pub fn new(
        device_id: impl Into<String>,
        device_type: impl Into<String>,
        mac: impl Into<String>,
        cmd: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            ble_id: None,
            device_type: device_type.into(),
            mac: mac.into(),
            cmd: cmd.into(),
            data: data.into(),
            group_id: None,
        }
    }

    pub fn with_ble_id(mut self, ble_id: impl Into<String>) -> Self {
        self.ble_id = Some(ble_id.into());
        self
    }

    pub fn with_group_id(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = Some(group_id.into());
        self
    }

    pub fn from_metadata(
        metadata: &CloudBleMetadata,
        fallback_device_id: impl Into<String>,
        cmd: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            device_id: fallback_device_id.into(),
            ble_id: metadata.ble_id.clone(),
            device_type: metadata.device_type.clone(),
            mac: metadata.mac.clone(),
            cmd: cmd.into(),
            data: data.into(),
            group_id: metadata.group_id.clone(),
        }
    }

    pub fn ble_id_or_device_id(&self) -> &str {
        self.ble_id.as_deref().unwrap_or(&self.device_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudBlePollState {
    Connecting,
    Connected,
    Failed,
    NotConnected,
    Unknown(i64),
}

impl CloudBlePollState {
    pub const fn from_code(code: i64) -> Self {
        match code {
            0 => Self::Connecting,
            1 => Self::Connected,
            -1 => Self::Failed,
            2 => Self::NotConnected,
            other => Self::Unknown(other),
        }
    }
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudBlePollState {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let code = match value.kind() {
            JsonValueKind::Object => optional_i64_any(value, &["state", "status", "result"])?
                .unwrap_or_else(|| i64::from(truthy(value).unwrap_or(false))),
            JsonValueKind::Boolean => i64::from(bool::try_from(value)?),
            JsonValueKind::String => {
                let raw = String::try_from(value)?;
                raw.parse::<i64>().map_err(|error| value.invalid(error))?
            }
            _ => i64::try_from(value)?,
        };
        Ok(Self::from_code(code))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudBleConnection {
    pub accepted: bool,
    pub state: Option<CloudBlePollState>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudBleConnection {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let state = CloudBlePollState::try_from(value).ok();
        Ok(Self {
            accepted: truthy(value).unwrap_or(matches!(state, Some(CloudBlePollState::Connected))),
            state,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloudBleControlResponse {
    pub accepted: bool,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CloudBleControlResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            accepted: truthy(value)?,
        })
    }
}

impl DeviceSummary {
    pub fn cloud_ble_metadata(&self) -> Option<CloudBleMetadata> {
        let mac = non_empty_string(self.mac.clone())?;
        Some(CloudBleMetadata {
            device_type: self.device_type_id.or(self.type_code).map_or_else(
                || self.device_type.as_str().to_string(),
                |value| value.to_string(),
            ),
            mac,
            group_id: Some(self.group_id.to_string()),
            ble_id: self.ble_id.clone(),
        })
    }
}

impl DeviceDetailResponse {
    pub fn cloud_ble_metadata(&self) -> Option<CloudBleMetadata> {
        let device_type = non_empty_string(self.device_type.clone())?;
        let mac = non_empty_string(self.mac.clone())?;
        Some(CloudBleMetadata {
            device_type,
            mac,
            group_id: self.group_id.map(|group_id| group_id.to_string()),
            ble_id: self
                .ble_id
                .clone()
                .or_else(|| self.id.map(|id| id.to_string())),
        })
    }
}

fn non_empty_string(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn required_string_any(
    value: RawJsonValue<'_, '_>,
    keys: &[&'static str],
) -> Result<String, JsonParseError> {
    for key in keys {
        if let Some(value) = optional_string_member(value.to_member(key)?)? {
            return Ok(value);
        }
    }
    Err(value.invalid(format!("missing required field `{}`", keys[0])))
}

fn optional_string_any(
    value: RawJsonValue<'_, '_>,
    keys: &[&'static str],
) -> Result<Option<String>, JsonParseError> {
    for key in keys {
        if let Some(value) = optional_string_member(value.to_member(key)?)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn optional_string_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<String>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) => {
            Ok(Some(String::try_from(value).or_else(|_| {
                u64::try_from(value).map(|value| value.to_string())
            })?))
        }
        None => Ok(None),
    }
}

fn optional_u64_any(
    value: RawJsonValue<'_, '_>,
    keys: &[&'static str],
) -> Result<Option<u64>, JsonParseError> {
    for key in keys {
        if let Some(value) = optional_u64_member(value.to_member(key)?)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn optional_u64_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<u64>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) if value.kind() == JsonValueKind::String => {
            let raw = String::try_from(value)?;
            raw.parse::<u64>()
                .map(Some)
                .map_err(|error| value.invalid(error))
        }
        Some(value) => u64::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn optional_i64_any(
    value: RawJsonValue<'_, '_>,
    keys: &[&'static str],
) -> Result<Option<i64>, JsonParseError> {
    for key in keys {
        match value.to_member(key)?.optional() {
            Some(value) if value.kind() == JsonValueKind::Null => {}
            Some(value) if value.kind() == JsonValueKind::String => {
                let raw = String::try_from(value)?;
                return raw
                    .parse::<i64>()
                    .map(Some)
                    .map_err(|error| value.invalid(error));
            }
            Some(value) => return i64::try_from(value).map(Some),
            None => {}
        }
    }
    Ok(None)
}

fn truthy(value: RawJsonValue<'_, '_>) -> Result<bool, JsonParseError> {
    match value.kind() {
        JsonValueKind::Boolean => bool::try_from(value),
        JsonValueKind::Integer | JsonValueKind::Float => Ok(i64::try_from(value)? != 0),
        JsonValueKind::String => {
            let raw = String::try_from(value)?;
            Ok(matches!(raw.as_str(), "1" | "true" | "ok" | "success"))
        }
        JsonValueKind::Object => {
            if let Some(value) = optional_i64_any(value, &["success", "accepted", "result"])? {
                return Ok(value != 0);
            }
            Ok(true)
        }
        JsonValueKind::Array | JsonValueKind::Null => Ok(false),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use nojson::RawJson;

    use crate::{DeviceType, FamilyListResponse};

    use super::*;

    #[test]
    fn family_list_device_summary_exposes_cloud_ble_metadata() {
        let raw = RawJson::parse(
            r#"{"result":[{"deviceList":[{"deviceId":42,"deviceName":"fountain","deviceType":"w5","groupId":7,"uniqueId":"w5-42","type":14,"mac":"aa:bb","bleId":"ble-42"}],"groupId":7,"petList":[]}]}"#,
        )
        .expect("fixture should parse");
        let response = FamilyListResponse::try_from(
            raw.value()
                .to_member("result")
                .expect("result")
                .required()
                .expect("result"),
        )
        .expect("family list should parse");
        let device = &response.groups[0].device_list[0];
        assert_eq!(device.device_type, DeviceType::W5);
        let metadata = device
            .cloud_ble_metadata()
            .expect("metadata should include mac");
        assert_eq!(metadata.device_type, "14");
        assert_eq!(metadata.mac, "aa:bb");
        assert_eq!(metadata.group_id.as_deref(), Some("7"));
        assert_eq!(metadata.ble_id.as_deref(), Some("ble-42"));
    }

    #[test]
    fn cloud_ble_devices_response_parses_relay_list() {
        let raw = RawJson::parse(
            r#"{"list":[{"id":1,"lowVersion":0,"mac":"11:22","name":"relay","pim":1,"sn":"sn","typeId":14}]}"#,
        )
        .expect("fixture should parse");
        let response =
            CloudBleDevicesResponse::try_from(raw.value()).expect("relay list should parse");
        assert_eq!(response.devices.len(), 1);
        assert_eq!(response.devices[0].id, "1");
        assert_eq!(response.devices[0].mac, "11:22");
    }

    #[test]
    fn device_detail_exposes_cloud_ble_metadata() {
        let raw = RawJson::parse(
            r#"{"id":42,"deviceType":"W5","groupId":7,"mac":"aa:bb","settings":{},"state":{}}"#,
        )
        .expect("fixture should parse");
        let detail = DeviceDetailResponse::try_from(raw.value()).expect("detail should parse");
        let metadata = detail
            .cloud_ble_metadata()
            .expect("metadata should include mac");

        assert_eq!(metadata.device_type, "W5");
        assert_eq!(metadata.mac, "aa:bb");
        assert_eq!(metadata.group_id.as_deref(), Some("7"));
        assert_eq!(metadata.ble_id.as_deref(), Some("42"));
    }

    #[test]
    fn cloud_ble_control_request_keeps_device_id_separate_from_ble_id() {
        let metadata = CloudBleMetadata {
            device_type: String::from("W5"),
            mac: String::from("aa:bb"),
            group_id: Some(String::from("7")),
            ble_id: Some(String::from("ble-42")),
        };

        let request = CloudBleControlRequest::from_metadata(&metadata, "device-42", "1", "abcd");

        assert_eq!(request.device_id, "device-42");
        assert_eq!(request.ble_id.as_deref(), Some("ble-42"));
        assert_eq!(request.ble_id_or_device_id(), "ble-42");
    }
}
