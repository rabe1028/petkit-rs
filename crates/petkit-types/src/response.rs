use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use nojson::{JsonParseError, JsonValueKind, RawJsonOwned, RawJsonValue};

use crate::{AccountGroup, IotConfigSet, PETKIT_AGORA_APP_ID, RegionServersPayload, Session};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionServersResponse {
    pub payload: RegionServersPayload,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for RegionServersResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: value.try_into()?,
        })
    }
}

impl From<RegionServersResponse> for RegionServersPayload {
    fn from(value: RegionServersResponse) -> Self {
        value.payload
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestLoginCodeResponse {
    pub sent: bool,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for RequestLoginCodeResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            sent: truthy(value)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginResponse {
    pub session: Session,
    pub api_servers: Vec<String>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for LoginResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            session: value.to_member("session")?.required()?.try_into()?,
            api_servers: optional_string_array(value.to_member("apiServers")?)?,
        })
    }
}

impl From<LoginResponse> for Session {
    fn from(value: LoginResponse) -> Self {
        value.session
    }
}

pub type RefreshSessionResponse = LoginResponse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyListResponse {
    pub groups: Vec<AccountGroup>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for FamilyListResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            groups: value
                .to_array()?
                .map(AccountGroup::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<FamilyListResponse> for Vec<AccountGroup> {
    fn from(value: FamilyListResponse) -> Self {
        value.groups
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IotDeviceInfoResponse {
    pub config: IotConfigSet,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for IotDeviceInfoResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            config: value.try_into()?,
        })
    }
}

impl From<IotDeviceInfoResponse> for IotConfigSet {
    fn from(value: IotDeviceInfoResponse) -> Self {
        value.config
    }
}

pub type IotDeviceInfoV1Response = IotDeviceInfoResponse;
pub type IotDeviceInfoV2Response = IotDeviceInfoResponse;

/// Broad per-device detail payload shared across feeder, litter, fountain, and
/// purifier detail reads.
///
/// PETKIT returns large, model-specific `settings` and `state` objects that
/// vary across device families and firmware revisions, so those subtrees are
/// preserved as raw JSON while the most stable top-level identifiers are parsed
/// eagerly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceDetailResponse {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub device_type: Option<String>,
    pub group_id: Option<u64>,
    pub mac: Option<String>,
    pub ble_id: Option<String>,
    pub sn: Option<String>,
    pub firmware: Option<String>,
    pub settings: Option<RawJsonOwned>,
    pub state: Option<RawJsonOwned>,
}

impl DeviceDetailResponse {
    pub fn settings_member(&self, key: &str) -> Result<Option<RawJsonOwned>, JsonParseError> {
        raw_json_member(self.settings.as_ref(), key)
    }

    pub fn state_member(&self, key: &str) -> Result<Option<RawJsonOwned>, JsonParseError> {
        raw_json_member(self.state.as_ref(), key)
    }
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for DeviceDetailResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let name = match value.to_member("name")?.optional() {
            Some(name) => String::try_from(name).map(Some)?,
            None => optional_string_member(value.to_member("deviceName")?)?,
        };

        Ok(Self {
            id: optional_u64_member(value.to_member("id")?)?,
            name,
            device_type: optional_string_any(value, &["deviceType", "typeName", "type_name"])?,
            group_id: optional_u64_member(value.to_member("groupId")?)?,
            mac: optional_string_any_deep(
                value,
                &[
                    "mac",
                    "btMac",
                    "bt_mac",
                    "deviceMac",
                    "bleMac",
                    "ble_mac",
                    "bluetoothMac",
                    "bluetooth_mac",
                    "macAddress",
                    "mac_address",
                ],
            )?,
            ble_id: optional_string_any_deep(
                value,
                &["bleId", "ble_id", "bleDeviceId", "ble_device_id"],
            )?,
            sn: optional_string_member(value.to_member("sn")?)?,
            firmware: optional_text_member(value.to_member("firmware")?)?,
            settings: optional_raw_member(value.to_member("settings")?)?,
            state: optional_raw_member(value.to_member("state")?)?,
        })
    }
}

pub type FeederDeviceDetailResponse = DeviceDetailResponse;
pub type LitterDeviceDetailResponse = DeviceDetailResponse;
pub type FountainDeviceDetailResponse = DeviceDetailResponse;
pub type PurifierDeviceDetailResponse = DeviceDetailResponse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualFeedResponse {
    pub accepted: bool,
    pub feed_id: Option<u64>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for ManualFeedResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let feed_id = if value.kind() == JsonValueKind::Object {
            optional_u64_member(value.to_member("id")?)?
        } else {
            None
        };
        Ok(Self {
            accepted: truthy(value)?,
            feed_id,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandResponse {
    pub accepted: bool,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for CommandResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            accepted: truthy(value)?,
        })
    }
}

pub type UpdateSettingResponse = CommandResponse;
pub type ControlDeviceResponse = CommandResponse;
pub type CancelManualFeedResponse = CommandResponse;
pub type RemoveDailyFeedResponse = CommandResponse;
pub type RestoreDailyFeedResponse = CommandResponse;
pub type SaveFeedResponse = CommandResponse;
pub type SuspendFeedResponse = CommandResponse;
pub type RestoreFeedResponse = CommandResponse;
pub type SaveRepeatsResponse = CommandResponse;
pub type ResetDesiccantResponse = CommandResponse;
pub type FoodReplenishedResponse = CommandResponse;
pub type CalibrationResponse = CommandResponse;
pub type CallPetResponse = CommandResponse;
pub type PlaySoundResponse = CommandResponse;
pub type ResetN50DeodorizerResponse = CommandResponse;
pub type ScheduleSaveResponse = CommandResponse;
pub type ScheduleRemoveResponse = CommandResponse;
pub type ScheduleCompleteResponse = CommandResponse;

pub type FeederUpdateSettingResponse = UpdateSettingResponse;
pub type FeederManualFeedResponse = ManualFeedResponse;
pub type FeederCancelManualFeedResponse = CancelManualFeedResponse;
pub type FeederRemoveDailyFeedResponse = RemoveDailyFeedResponse;
pub type FeederRestoreDailyFeedResponse = RestoreDailyFeedResponse;
pub type FeederSaveFeedResponse = SaveFeedResponse;
pub type FeederSuspendFeedResponse = SuspendFeedResponse;
pub type FeederRestoreFeedResponse = RestoreFeedResponse;
pub type FeederSaveRepeatsResponse = SaveRepeatsResponse;
pub type FeederResetDesiccantResponse = ResetDesiccantResponse;
pub type FeederFoodReplenishedResponse = FoodReplenishedResponse;
pub type FeederCalibrationResponse = CalibrationResponse;
pub type FeederCallPetResponse = CallPetResponse;
pub type FeederPlaySoundResponse = PlaySoundResponse;
pub type FeederScheduleSaveResponse = ScheduleSaveResponse;
pub type FeederScheduleRemoveResponse = ScheduleRemoveResponse;
pub type FeederScheduleCompleteResponse = ScheduleCompleteResponse;
pub type FeederSettingsReadResponse = FeederDeviceDetailResponse;

pub type LitterUpdateSettingResponse = UpdateSettingResponse;
pub type LitterControlDeviceResponse = ControlDeviceResponse;
pub type LitterResetN50DeodorizerResponse = ResetN50DeodorizerResponse;
pub type LitterScheduleSaveResponse = ScheduleSaveResponse;
pub type LitterScheduleRemoveResponse = ScheduleRemoveResponse;
pub type LitterScheduleCompleteResponse = ScheduleCompleteResponse;
pub type PuraMaxControlDeviceResponse = LitterControlDeviceResponse;
pub type PuraMaxResetDeodorizerResponse = LitterResetN50DeodorizerResponse;
pub type LitterSettingsReadResponse = LitterDeviceDetailResponse;

pub type FountainUpdateSettingResponse = UpdateSettingResponse;
pub type FountainSettingsReadResponse = FountainDeviceDetailResponse;

pub type PurifierUpdateSettingResponse = UpdateSettingResponse;
pub type PurifierControlDeviceResponse = ControlDeviceResponse;
pub type PurifierSettingsReadResponse = PurifierDeviceDetailResponse;

pub type PetUpdateSettingResponse = UpdateSettingResponse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveFeedResponse {
    pub accepted: bool,
    pub whep_url: Option<String>,
    pub app_id: Option<String>,
    pub channel_id: Option<String>,
    pub rtc_token: Option<String>,
    pub rtm_token: Option<String>,
    pub uid: Option<u32>,
    pub app_rtm_user_id: Option<String>,
    pub dev_rtm_user_id: Option<String>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for LiveFeedResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let payload = live_feed_payload(value)?;
        let whep_url = optional_string_any(payload, &["whepUrl", "whep_url", "whep"])?;
        let parsed_app_id = optional_string_any(payload, &["appId", "app_id"])?;
        let channel_id = optional_string_any(payload, &["channelId", "channel_id"])?;
        let rtc_token = optional_string_any(payload, &["rtcToken", "rtc_token"])?;
        let rtm_token = optional_string_any(payload, &["rtmToken", "rtm_token"])?;
        let app_rtm_user_id = optional_string_any(payload, &["appRtmUserId", "app_rtm_user_id"])?;
        let dev_rtm_user_id = optional_string_any(payload, &["devRtmUserId", "dev_rtm_user_id"])?;
        let uid = optional_u32_any(payload, &["uid", "userId", "user_id"])?
            .or_else(|| uid_from_rtm_user_id(app_rtm_user_id.as_deref()));
        let has_live_payload = channel_id.is_some()
            || whep_url.is_some()
            || parsed_app_id.is_some()
            || rtc_token.is_some()
            || rtm_token.is_some()
            || uid.is_some()
            || app_rtm_user_id.is_some()
            || dev_rtm_user_id.is_some();
        let app_id = parsed_app_id.or_else(|| Some(PETKIT_AGORA_APP_ID.to_string()));
        Ok(Self {
            accepted: live_feed_accepted(value, payload, has_live_payload)?,
            whep_url,
            app_id,
            channel_id,
            rtc_token,
            rtm_token,
            uid,
            app_rtm_user_id,
            dev_rtm_user_id,
        })
    }
}

pub type StartLiveResponse = LiveFeedResponse;
pub type CameraLiveFeed = LiveFeedResponse;
pub type FeederStartLiveResponse = StartLiveResponse;
pub type LitterStartLiveResponse = StartLiveResponse;
pub type OpenCameraResponse = LiveFeedResponse;
pub type FeederOpenCameraResponse = OpenCameraResponse;
pub type LitterOpenCameraResponse = OpenCameraResponse;

fn optional_u64_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<u64>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) if value.kind() == JsonValueKind::String => {
            let raw: String = value.try_into()?;
            raw.parse::<u64>()
                .map(Some)
                .map_err(|error| value.invalid(error))
        }
        Some(value) => u64::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn optional_string_array<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Vec<String>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(Vec::new()),
        Some(value) => value
            .to_array()?
            .map(String::try_from)
            .collect::<Result<Vec<_>, _>>(),
        None => Ok(Vec::new()),
    }
}

fn optional_u32_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<u32>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) if value.kind() == JsonValueKind::String => {
            let raw: String = value.try_into()?;
            raw.parse::<u32>()
                .map(Some)
                .map_err(|error| value.invalid(error))
        }
        Some(value) => u32::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn optional_u32_any(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
) -> Result<Option<u32>, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
    for name in names {
        if let Some(value) = optional_u32_member(value.to_member(name)?)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn optional_bool_any(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
) -> Result<Option<bool>, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
    for name in names {
        match value.to_member(name)?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => {}
            Some(member) if member.kind() == JsonValueKind::String => {
                let raw: String = member.try_into()?;
                return Ok(Some(matches!(
                    raw.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "on" | "yes"
                )));
            }
            Some(member) => return truthy(member).map(Some),
            None => {}
        }
    }
    Ok(None)
}

fn optional_string_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<String>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) => String::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn optional_string_any(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
) -> Result<Option<String>, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
    for name in names {
        match value.to_member(name)?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => {}
            Some(member) => return scalar_to_string(member).map(Some),
            None => {}
        }
    }
    Ok(None)
}

fn scalar_to_string(value: RawJsonValue<'_, '_>) -> Result<String, JsonParseError> {
    match value.kind() {
        JsonValueKind::String => String::try_from(value),
        JsonValueKind::Integer | JsonValueKind::Float => Ok(String::from(value.as_number_str()?)),
        JsonValueKind::Boolean => Ok(if bool::try_from(value)? {
            String::from("true")
        } else {
            String::from("false")
        }),
        JsonValueKind::Array | JsonValueKind::Object | JsonValueKind::Null => {
            Err(value.invalid("expected scalar string-compatible value"))
        }
    }
}

fn optional_string_any_deep(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
) -> Result<Option<String>, JsonParseError> {
    optional_string_any_deep_inner(value, names, 0)
}

fn optional_string_any_deep_inner(
    value: RawJsonValue<'_, '_>,
    names: &[&str],
    depth: usize,
) -> Result<Option<String>, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return Ok(None);
    }
    if let Some(found) = optional_string_any(value, names)? {
        return Ok(Some(found));
    }
    if depth >= 4 {
        return Ok(None);
    }
    for (_, member) in value.to_object()? {
        match member.kind() {
            JsonValueKind::Object => {
                if let Some(found) = optional_string_any_deep_inner(member, names, depth + 1)? {
                    return Ok(Some(found));
                }
            }
            JsonValueKind::Array => {
                for item in member.to_array()? {
                    if item.kind() == JsonValueKind::Object
                        && let Some(found) = optional_string_any_deep_inner(item, names, depth + 1)?
                    {
                        return Ok(Some(found));
                    }
                }
            }
            JsonValueKind::Null
            | JsonValueKind::Boolean
            | JsonValueKind::Integer
            | JsonValueKind::Float
            | JsonValueKind::String => {}
        }
    }
    Ok(None)
}

fn live_feed_payload<'text, 'raw>(
    value: RawJsonValue<'text, 'raw>,
) -> Result<RawJsonValue<'text, 'raw>, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return Ok(value);
    }
    for name in ["data", "live", "liveFeed", "live_feed"] {
        if let Some(member) = value.to_member(name)?.optional()
            && member.kind() == JsonValueKind::Object
        {
            return Ok(member);
        }
    }
    Ok(value)
}

fn live_feed_accepted(
    value: RawJsonValue<'_, '_>,
    payload: RawJsonValue<'_, '_>,
    has_live_payload: bool,
) -> Result<bool, JsonParseError> {
    if value.kind() != JsonValueKind::Object {
        return truthy(value);
    }
    if let Some(accepted) = optional_bool_any(value, &["accepted", "success", "ok"])? {
        return Ok(accepted && has_live_payload);
    }
    if let Some(accepted) = optional_bool_any(payload, &["accepted", "success", "ok"])? {
        return Ok(accepted && has_live_payload);
    }
    Ok(has_live_payload)
}

fn uid_from_rtm_user_id(value: Option<&str>) -> Option<u32> {
    value?.split('_').find_map(|part| part.parse::<u32>().ok())
}

fn optional_text_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<String>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) => match value.kind() {
            JsonValueKind::String => String::try_from(value).map(Some),
            JsonValueKind::Integer | JsonValueKind::Float => {
                Ok(Some(value.as_number_str()?.to_owned()))
            }
            JsonValueKind::Boolean => Ok(Some(bool::try_from(value)?.to_string())),
            JsonValueKind::Null => Ok(None),
            JsonValueKind::Array | JsonValueKind::Object => Ok(Some(value.as_raw_str().to_owned())),
        },
        None => Ok(None),
    }
}

fn optional_raw_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<RawJsonOwned>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) => RawJsonOwned::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn raw_json_member(
    value: Option<&RawJsonOwned>,
    key: &str,
) -> Result<Option<RawJsonOwned>, JsonParseError> {
    let Some(value) = value else {
        return Ok(None);
    };
    match value.value().to_member(key)?.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) => RawJsonOwned::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn truthy(value: RawJsonValue<'_, '_>) -> Result<bool, JsonParseError> {
    let truthy = match value.kind() {
        JsonValueKind::Null => false,
        JsonValueKind::Boolean => bool::try_from(value)?,
        JsonValueKind::Integer | JsonValueKind::Float => {
            let raw = value.as_number_str()?;
            if let Ok(n) = raw.parse::<i64>() {
                n != 0
            } else if let Ok(n) = raw.parse::<u64>() {
                n != 0
            } else {
                raw.parse::<f64>().map(|n| n != 0.0).unwrap_or(false)
            }
        }
        JsonValueKind::String => !value.to_unquoted_string_str()?.is_empty(),
        JsonValueKind::Array => value.to_array()?.next().is_some(),
        JsonValueKind::Object => value.to_object()?.next().is_some(),
    };
    Ok(truthy)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use alloc::string::String;

    use nojson::RawJson;
    use secrecy::ExposeSecret;

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

    const FEEDER_OPEN_CAMERA_FIXTURE: &str = r#"{"result":{"data":{"channelId":"feeder-open-redacted","rtcToken":"rtc-token-redacted","rtmToken":"rtm-token-redacted","appRtmUserId":"app_1001","devRtmUserId":"dev_2001","uid":"1001"}}}"#;
    const FEEDER_START_LIVE_FIXTURE: &str = r#"{"result":{"live":{"channelId":"feeder-live-redacted","rtcToken":"rtc-live-redacted","rtmToken":"rtm-live-redacted","appRtmUserId":"app_1002","devRtmUserId":"dev_2002"}}}"#;
    const LITTER_OPEN_CAMERA_FIXTURE: &str = r#"{"result":{"liveFeed":{"channel_id":"litter-open-redacted","rtc_token":"rtc-litter-open-redacted","rtm_token":"rtm-litter-open-redacted","app_rtm_user_id":"app_3001","dev_rtm_user_id":"dev_4001","uid":3001}}}"#;
    const LITTER_START_LIVE_FIXTURE: &str = r#"{"result":{"channelId":"litter-live-redacted","rtcToken":"rtc-litter-live-redacted","rtmToken":"rtm-litter-live-redacted","appRtmUserId":"app_3002","devRtmUserId":"dev_4002"}}"#;

    #[test]
    fn login_response_parses_nested_session() {
        let response = with_result(
            r#"{"result":{"apiServers":["https://api.petkt.com/6/"],"session":{"id":"s1","userId":"u1","expiresIn":3600,"region":"DE","createdAt":"2026-05-27T00:00:00.000+0000","refreshedAt":null}}}"#,
            |value| LoginResponse::try_from(value).expect("login response should parse"),
        );

        assert_eq!(response.session.id.expose_secret(), "s1");
        assert_eq!(response.api_servers, ["https://api.petkt.com/6/"]);
        assert_eq!(Session::from(response).user_id, "u1");
    }

    #[test]
    fn family_list_response_parses_groups() {
        let response = with_result(
            r#"{"result":[{"deviceList":[],"groupId":1,"name":"home","petList":[{"avatar":null,"petId":7,"petName":"mugi"}]}]}"#,
            |value| FamilyListResponse::try_from(value).expect("family list response should parse"),
        );

        assert_eq!(response.groups.len(), 1);
        assert_eq!(
            response.groups[0].pet_list[0].pet_name,
            String::from("mugi")
        );
    }

    #[test]
    fn family_list_response_stringifies_numeric_ble_ids() {
        let response = with_result(
            r#"{"result":[{"deviceList":[{"deviceId":42,"deviceName":"fountain","deviceType":"ctw3","groupId":1,"mac":"aa:bb","bleId":12345,"type":14,"typeCode":14,"uniqueId":"u-42"}],"groupId":1,"name":"home","petList":[]}]}"#,
            |value| FamilyListResponse::try_from(value).expect("family list response should parse"),
        );

        assert_eq!(
            response.groups[0].device_list[0].ble_id.as_deref(),
            Some("12345")
        );
    }

    #[test]
    fn command_response_accepts_truthy_result_values() {
        assert!(with_result(r#"{"result":1}"#, |value| {
            CommandResponse::try_from(value)
                .expect("numeric result should parse")
                .accepted
        }));
        assert!(!with_result(r#"{"result":false}"#, |value| {
            CommandResponse::try_from(value)
                .expect("boolean result should parse")
                .accepted
        }));
    }

    #[test]
    fn manual_feed_response_captures_optional_feed_id() {
        let response = with_result(r#"{"result":{"id":"42"}}"#, |value| {
            ManualFeedResponse::try_from(value).expect("manual feed response should parse")
        });

        assert!(response.accepted);
        assert_eq!(response.feed_id, Some(42));

        let response = with_result(r#"{"result":true}"#, |value| {
            ManualFeedResponse::try_from(value).expect("manual feed response should parse")
        });

        assert!(response.accepted);
        assert_eq!(response.feed_id, None);
    }

    #[test]
    fn device_detail_response_preserves_broad_settings_and_state() {
        let response = with_result(
            r#"{"result":{"id":42,"name":"d4s feeder","sn":"sn-42","firmware":"1.2.3","settings":{"lightMode":1,"selectedSound":3},"state":{"food":78,"errorDetail":"none"}}}"#,
            |value| DeviceDetailResponse::try_from(value).expect("device detail should parse"),
        );

        assert_eq!(response.id, Some(42));
        assert_eq!(response.name.as_deref(), Some("d4s feeder"));
        assert_eq!(response.sn.as_deref(), Some("sn-42"));
        assert_eq!(response.firmware.as_deref(), Some("1.2.3"));
        assert_eq!(
            response
                .settings_member("lightMode")
                .expect("settings lookup should parse")
                .expect("lightMode should exist")
                .text(),
            "1"
        );
        assert_eq!(
            response
                .state_member("food")
                .expect("state lookup should parse")
                .expect("food should exist")
                .text(),
            "78"
        );
    }

    #[test]
    fn device_detail_response_stringifies_numeric_firmware() {
        let response = with_result(
            r#"{"result":{"id":7,"deviceName":"k3 purifier","sn":"pk-7","firmware":203,"settings":{"sound":1},"state":{"mode":2}}}"#,
            |value| DeviceDetailResponse::try_from(value).expect("device detail should parse"),
        );

        assert_eq!(response.name.as_deref(), Some("k3 purifier"));
        assert_eq!(response.firmware.as_deref(), Some("203"));
        assert_eq!(
            response
                .settings_member("sound")
                .expect("settings lookup should parse")
                .expect("sound should exist")
                .text(),
            "1"
        );
        assert_eq!(
            response
                .state_member("mode")
                .expect("state lookup should parse")
                .expect("mode should exist")
                .text(),
            "2"
        );
    }

    #[test]
    fn device_detail_response_finds_nested_cloud_ble_identifiers() {
        let response = with_result(
            r#"{"result":{"id":1000024016,"name":"ctw3 fountain","deviceType":"ctw3","deviceInfo":{"bluetoothMac":"aa:bb:cc:dd:ee:ff","bleDeviceId":12345},"settings":{},"state":{}}}"#,
            |value| DeviceDetailResponse::try_from(value).expect("device detail should parse"),
        );

        assert_eq!(response.mac.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert_eq!(response.ble_id.as_deref(), Some("12345"));
    }

    #[test]
    fn live_feed_response_parses_agora_tokens_and_uid_aliases() {
        let response = with_result(
            r#"{"result":{"channelId":"ch-1","rtcToken":"rtc","rtmToken":"rtm","appRtmUserId":"app_123","devRtmUserId":"dev_456"}}"#,
            |value| LiveFeedResponse::try_from(value).expect("live feed should parse"),
        );

        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("ch-1"));
        assert_eq!(response.rtc_token.as_deref(), Some("rtc"));
        assert_eq!(response.rtm_token.as_deref(), Some("rtm"));
        assert_eq!(response.app_id.as_deref(), Some(PETKIT_AGORA_APP_ID));
        assert_eq!(response.uid, Some(123));
        assert_eq!(response.app_rtm_user_id.as_deref(), Some("app_123"));
        assert_eq!(response.dev_rtm_user_id.as_deref(), Some("dev_456"));

        let response = with_result(
            r#"{"result":{"channel_id":"ch-2","uid":"77","app_rtm_user_id":"app_123"}}"#,
            |value| LiveFeedResponse::try_from(value).expect("live feed should parse"),
        );
        assert_eq!(response.channel_id.as_deref(), Some("ch-2"));
        assert_eq!(response.uid, Some(77));

        let response = with_result(
            r#"{"result":{"data":{"channelId":"ch-3","uid":88}}}"#,
            |value| OpenCameraResponse::try_from(value).expect("open camera should parse"),
        );
        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("ch-3"));
        assert_eq!(response.uid, Some(88));

        let response = with_result(r#"{"result":true}"#, |value| {
            OpenCameraResponse::try_from(value).expect("command-shaped open camera should parse")
        });
        assert!(response.accepted);
        assert_eq!(response.channel_id, None);
    }

    #[test]
    fn live_feed_aliases_skip_null_values() {
        let response = with_result(
            r#"{"result":{"channelId":null,"channel_id":"ch-fallback","rtcToken":null,"rtc_token":"rtc-fallback","rtmToken":null,"rtm_token":"rtm-fallback","appRtmUserId":null,"app_rtm_user_id":"app_321","devRtmUserId":null,"dev_rtm_user_id":"dev_654","accepted":null,"success":true}}"#,
            |value| LiveFeedResponse::try_from(value).expect("live feed should parse"),
        );

        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("ch-fallback"));
        assert_eq!(response.rtc_token.as_deref(), Some("rtc-fallback"));
        assert_eq!(response.rtm_token.as_deref(), Some("rtm-fallback"));
        assert_eq!(response.uid, Some(321));
        assert_eq!(response.app_rtm_user_id.as_deref(), Some("app_321"));
        assert_eq!(response.dev_rtm_user_id.as_deref(), Some("dev_654"));
    }

    #[test]
    fn live_feed_response_parses_sanitized_petkit_fixtures() {
        let response = with_result(FEEDER_OPEN_CAMERA_FIXTURE, |value| {
            FeederOpenCameraResponse::try_from(value)
                .expect("feeder open_camera fixture should parse")
        });
        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("feeder-open-redacted"));
        assert_eq!(response.rtc_token.as_deref(), Some("rtc-token-redacted"));
        assert_eq!(response.rtm_token.as_deref(), Some("rtm-token-redacted"));
        assert_eq!(response.uid, Some(1001));
        assert_eq!(response.app_rtm_user_id.as_deref(), Some("app_1001"));
        assert_eq!(response.dev_rtm_user_id.as_deref(), Some("dev_2001"));

        let response = with_result(FEEDER_START_LIVE_FIXTURE, |value| {
            FeederStartLiveResponse::try_from(value)
                .expect("feeder start_live fixture should parse")
        });
        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("feeder-live-redacted"));
        assert_eq!(response.uid, Some(1002));

        let response = with_result(LITTER_OPEN_CAMERA_FIXTURE, |value| {
            LitterOpenCameraResponse::try_from(value)
                .expect("litter open_camera fixture should parse")
        });
        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("litter-open-redacted"));
        assert_eq!(response.uid, Some(3001));

        let response = with_result(LITTER_START_LIVE_FIXTURE, |value| {
            LitterStartLiveResponse::try_from(value)
                .expect("litter start_live fixture should parse")
        });
        assert!(response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("litter-live-redacted"));
        assert_eq!(response.uid, Some(3002));
    }

    #[test]
    fn live_feed_empty_or_failed_response_is_not_accepted() {
        let response = with_result(r#"{"result":{}}"#, |value| {
            LiveFeedResponse::try_from(value).expect("empty response should parse")
        });
        assert!(!response.accepted);

        let response = with_result(r#"{"result":{"data":{}}}"#, |value| {
            LiveFeedResponse::try_from(value).expect("empty nested response should parse")
        });
        assert!(!response.accepted);

        let response = with_result(
            r#"{"result":{"success":false,"channelId":"redacted","rtcToken":"redacted"}}"#,
            |value| LiveFeedResponse::try_from(value).expect("failed response should parse"),
        );
        assert!(!response.accepted);
        assert_eq!(response.channel_id.as_deref(), Some("redacted"));
    }
}
