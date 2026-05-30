use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use nojson::{JsonParseError, JsonValueKind, RawJsonOwned, RawJsonValue};

use crate::{AccountGroup, IotConfigSet, RegionServersPayload, Session};

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
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for LoginResponse {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            session: value.to_member("session")?.required()?.try_into()?,
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

fn optional_string_member<'text, 'raw, 'a>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<String>, JsonParseError> {
    match member.optional() {
        Some(value) if value.kind() == JsonValueKind::Null => Ok(None),
        Some(value) => String::try_from(value).map(Some),
        None => Ok(None),
    }
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
    fn login_response_parses_nested_session() {
        let response = with_result(
            r#"{"result":{"session":{"id":"s1","userId":"u1","expiresIn":3600,"region":"DE","createdAt":"2026-05-27T00:00:00.000+0000","refreshedAt":null}}}"#,
            |value| LoginResponse::try_from(value).expect("login response should parse"),
        );

        assert_eq!(response.session.id, "s1");
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
}
