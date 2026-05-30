//! Typed setting payloads for `update_setting` / `pet_update_setting` requests.
//!
//! Each device family has a concrete enum exposing the most common settings
//! known from the Python ecosystem. The `Other` variant is an escape hatch
//! for keys not yet enumerated here — extend the enum upstream when
//! adding first-class support.

use alloc::format;
use alloc::string::{String, ToString};

use core::fmt;

use nojson::{DisplayJson, JsonFormatter, JsonParseError, JsonValueKind, RawJsonOwned};

use crate::{
    ControlCommandType, FeedEntryId, FeedTime, FeederSurplusGrams, LitterModeValue, LitterSandType,
    LitterStillTimeSeconds, LitterWorkMode, PetWeightGrams, PetkitError, PurifierMode, RepeatDays,
    SettingInt, SettingString, VolumeLevel,
};

/// A custom `(key, value)` pair for PETKIT settings not yet modeled as a
/// first-class enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomSetting {
    /// JSON object key, possibly a dotted path like `"settings.lightMode"`.
    key: String,
    value: CustomSettingValue,
}

impl CustomSetting {
    pub fn new(key: impl Into<String>, value: CustomSettingValue) -> Result<Self, PetkitError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(PetkitError::InvalidArgument(String::from(
                "custom setting key must not be empty",
            )));
        }
        Ok(Self { key, value })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &CustomSettingValue {
        &self.value
    }
}

/// Value payload for a custom PETKIT setting.
///
/// Boolean settings are encoded as integer flags because the PETKIT settings
/// endpoints commonly expect `0` / `1` instead of JSON `false` / `true`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustomSettingValue {
    BoolAsInt(bool),
    Int(SettingInt),
    String(SettingString),
    /// Parsed JSON value for object/array or other structured settings.
    Json(RawJsonOwned),
}

impl CustomSettingValue {
    pub fn json(raw_json: impl Into<String>) -> Result<Self, JsonParseError> {
        RawJsonOwned::parse(raw_json).map(Self::Json)
    }
}

impl DisplayJson for CustomSettingValue {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::BoolAsInt(value) => f.value(if *value { 1u8 } else { 0u8 }),
            Self::Int(value) => f.value(value.get()),
            Self::String(value) => f.value(value.as_str()),
            Self::Json(value) => value.fmt(f),
        }
    }
}

fn write_bool_int(f: &mut JsonFormatter<'_, '_>, key: &str, value: bool) -> fmt::Result {
    f.object(|f| f.member(key, if value { 1u8 } else { 0u8 }))
}

fn write_int(f: &mut JsonFormatter<'_, '_>, key: &str, value: i64) -> fmt::Result {
    f.object(|f| f.member(key, value))
}

fn write_custom_setting(f: &mut JsonFormatter<'_, '_>, setting: &CustomSetting) -> fmt::Result {
    f.object(|f| f.member(&setting.key, &setting.value))
}

/// Settings sendable to feeder devices (`D3`, `D4`, `D4s`, `D4h`, `D4sh`,
/// `Feeder`, `FeederMini`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeederSetting {
    /// LED indicator (0 = off, 1 = on).
    LightMode(bool),
    /// Child lock.
    ManualLock(bool),
    /// Low food warning.
    FoodWarn(bool),
    /// Feed sound on/off (D4 series).
    FeedSound(bool),
    /// Feed tone on/off (D4s).
    FeedTone(bool),
    /// Voice feed enable (D3/D4h/D4sh).
    SoundEnable(bool),
    /// Speaker volume (D3/D4h/D4sh).
    Volume(VolumeLevel),
    /// Pre-feed surplus threshold (D3).
    Surplus(FeederSurplusGrams),
    /// Feed notification.
    FeedNotify(bool),
    /// Escape hatch for settings not enumerated above.
    Other(CustomSetting),
}

impl DisplayJson for FeederSetting {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::LightMode(value) => write_bool_int(f, "lightMode", *value),
            Self::ManualLock(value) => write_bool_int(f, "manualLock", *value),
            Self::FoodWarn(value) => write_bool_int(f, "foodWarn", *value),
            Self::FeedSound(value) => write_bool_int(f, "feedSound", *value),
            Self::FeedTone(value) => write_bool_int(f, "feedTone", *value),
            Self::SoundEnable(value) => write_bool_int(f, "soundEnable", *value),
            Self::Volume(value) => write_int(f, "volume", i64::from(value.get())),
            Self::Surplus(value) => write_int(f, "surplus", i64::from(value.get())),
            Self::FeedNotify(value) => write_bool_int(f, "feedNotify", *value),
            Self::Other(setting) => write_custom_setting(f, setting),
        }
    }
}

/// Settings sendable to litter box devices (`T3`, `T4`, `T5`, `T6`, `T7`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LitterSetting {
    LightMode(bool),
    ManualLock(bool),
    DisturbMode(bool),
    AutoWork(bool),
    AvoidRepeat(bool),
    FixedTimeClear(bool),
    Kitten(bool),
    StillTime(LitterStillTimeSeconds),
    AutoRefresh(bool),
    FixedTimeRefresh(bool),
    SandType(LitterSandType),
    Volume(VolumeLevel),
    /// Escape hatch for settings not enumerated above.
    Other(CustomSetting),
}

impl DisplayJson for LitterSetting {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::LightMode(value) => write_bool_int(f, "lightMode", *value),
            Self::ManualLock(value) => write_bool_int(f, "manualLock", *value),
            Self::DisturbMode(value) => write_bool_int(f, "disturbMode", *value),
            Self::AutoWork(value) => write_bool_int(f, "autoWork", *value),
            Self::AvoidRepeat(value) => write_bool_int(f, "avoidRepeat", *value),
            Self::FixedTimeClear(value) => write_bool_int(f, "fixedTimeClear", *value),
            Self::Kitten(value) => write_bool_int(f, "kitten", *value),
            Self::StillTime(value) => write_int(f, "stillTime", i64::from(value.get())),
            Self::AutoRefresh(value) => write_bool_int(f, "autoRefresh", *value),
            Self::FixedTimeRefresh(value) => write_bool_int(f, "fixedTimeRefresh", *value),
            Self::SandType(value) => write_int(f, "sandType", i64::from(value.get())),
            Self::Volume(value) => write_int(f, "volume", i64::from(value.get())),
            Self::Other(setting) => write_custom_setting(f, setting),
        }
    }
}

/// Settings sendable to air purifier devices (`K2`, `K3`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PurifierSetting {
    LightMode(bool),
    Sound(bool),
    Other(CustomSetting),
}

impl DisplayJson for PurifierSetting {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::LightMode(value) => write_bool_int(f, "lightMode", *value),
            Self::Sound(value) => write_bool_int(f, "sound", *value),
            Self::Other(setting) => write_custom_setting(f, setting),
        }
    }
}

/// Settings sendable to fountain devices (`W4`, `W5`, `Ctw2`, `Ctw3`).
///
/// Most fountain control is done over BLE; only `Other` is provided for the
/// rare HTTP-driven setting use cases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FountainSetting {
    Other(CustomSetting),
}

impl DisplayJson for FountainSetting {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::Other(setting) => write_custom_setting(f, setting),
        }
    }
}

/// Settings sendable via `pet_update_setting`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PetSetting {
    /// Pet weight in grams.
    Weight(PetWeightGrams),
    Other(CustomSetting),
}

impl DisplayJson for PetSetting {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::Weight(value) => write_int(f, "weight", i64::from(value.get())),
            Self::Other(setting) => write_custom_setting(f, setting),
        }
    }
}

/// JSON string fragment to be re-used as a setting value alongside other
/// fields (used by `extra_form`-style payloads that flatten multiple keys
/// into form fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtraFormPayload {
    fields: alloc::vec::Vec<(String, String)>,
}

impl ExtraFormPayload {
    fn new() -> Self {
        Self {
            fields: alloc::vec::Vec::new(),
        }
    }

    fn push(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    fn push_int(self, key: impl Into<String>, value: i64) -> Self {
        self.push(key, value.to_string())
    }

    pub fn fields(&self) -> &[(String, String)] {
        &self.fields
    }
}

impl Default for ExtraFormPayload {
    fn default() -> Self {
        Self::new()
    }
}

/// Identifier for a scheduled feed entry, used by
/// `remove_daily_feed` / `restore_daily_feed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedIdentifier {
    /// Schedule id (used by D4s/D4h/D4sh and newer models).
    Id(FeedEntryId),
    /// Schedule time (HHMM) and optional legacy `name` slot.
    LegacyTime {
        time: FeedTime,
        name: Option<String>,
    },
}

impl FeedIdentifier {
    pub fn by_id(id: u64) -> Result<Self, PetkitError> {
        Ok(Self::Id(FeedEntryId::new(id)?))
    }

    pub fn by_time(time: u32) -> Result<Self, PetkitError> {
        Ok(Self::LegacyTime {
            time: FeedTime::new_hhmm(time)?,
            name: None,
        })
    }

    pub fn by_time_with_name(time: u32, name: impl Into<String>) -> Result<Self, PetkitError> {
        Ok(Self::LegacyTime {
            time: FeedTime::new_hhmm(time)?,
            name: Some(name.into()),
        })
    }

    pub fn to_form(&self) -> ExtraFormPayload {
        match self {
            Self::Id(id) => ExtraFormPayload::new().push("id", id.to_string()),
            Self::LegacyTime { time, name } => {
                let mut payload = ExtraFormPayload::new().push("time", time.to_string());
                if let Some(name) = name {
                    payload = payload.push("name", name);
                }
                payload
            }
        }
    }
}

/// Repeat-schedule payload for `save_repeats`. Mirrors the petkit field
/// names that need to be sent as separate form fields rather than a
/// JSON-encoded `kv`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatSchedule {
    /// JSON-encoded repeat content. Format depends on the feeder model.
    repeats: String,
    /// Bitmask of repeat days (`Mon..Sun`).
    repeat_days: Option<RepeatDays>,
}

impl RepeatSchedule {
    pub fn new(repeats: impl Into<String>) -> Result<Self, PetkitError> {
        let repeats = parse_json_array(repeats.into(), "repeats")?;
        Ok(Self {
            repeats,
            repeat_days: None,
        })
    }

    pub fn with_repeat_days(mut self, repeat_days: RepeatDays) -> Self {
        self.repeat_days = Some(repeat_days);
        self
    }

    pub fn repeats(&self) -> &str {
        &self.repeats
    }

    pub const fn repeat_days(&self) -> Option<RepeatDays> {
        self.repeat_days
    }

    pub fn to_form(&self) -> ExtraFormPayload {
        let mut payload = ExtraFormPayload::new().push("repeats", self.repeats.clone());
        if let Some(days) = self.repeat_days {
            payload = payload.push_int("repeatDays", i64::from(days.get()));
        }
        payload
    }
}

/// Feed daily list payload for `save_feed`. The wire format is a
/// JSON-encoded array string put into the `feedDailyList` form field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedDailyList {
    /// Pre-formatted JSON array, e.g. `r#"[{"amount":10,"time":420}]"#`.
    raw_json: String,
}

impl FeedDailyList {
    pub fn from_json(raw_json: impl Into<String>) -> Result<Self, PetkitError> {
        Ok(Self {
            raw_json: parse_json_array(raw_json.into(), "feedDailyList")?,
        })
    }

    pub fn raw_json(&self) -> &str {
        &self.raw_json
    }
}

/// `control_device` payload for litter boxes.
///
/// Each variant maps to a `(type, kv-json)` pair sent on the wire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LitterControl {
    /// `{"start_action": 0}` — cleaning.
    StartCleaning,
    /// `{"start_action": 1}` — dumping.
    StartDumping,
    /// `{"start_action": 2}` — odor removal.
    StartOdorRemoval,
    /// `{"start_action": 4}` — leveling.
    StartLeveling,
    /// `{"start_action": 9}` — enter maintenance.
    StartMaintenance,
    /// `{"end_action": 9}` — exit maintenance.
    EndMaintenance,
    /// `{"stop_action": <work_mode>}`.
    Stop { work_mode: LitterWorkMode },
    /// `{"continue_action": <work_mode>}`.
    Continue { work_mode: LitterWorkMode },
    /// `{"end_action": <work_mode>}`.
    End { work_mode: LitterWorkMode },
    /// `{"power_action": 0_or_1}`.
    Power(bool),
    /// `{"mode_action": <value>}` (Pura MAX light mode etc.).
    Mode(LitterModeValue),
    /// Escape hatch: write the kv object yourself, alongside the `type`
    /// form field name.
    Other {
        command_type: ControlCommandType,
        setting: CustomSetting,
    },
}

impl LitterControl {
    /// Value of the `type` form field that accompanies the JSON `kv` body.
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::StartCleaning
            | Self::StartDumping
            | Self::StartOdorRemoval
            | Self::StartLeveling
            | Self::StartMaintenance => "start",
            Self::EndMaintenance | Self::End { .. } => "end",
            Self::Stop { .. } => "stop",
            Self::Continue { .. } => "continue",
            Self::Power(_) => "power",
            Self::Mode(_) => "mode",
            Self::Other { command_type, .. } => command_type.as_str(),
        }
    }
}

impl DisplayJson for LitterControl {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::StartCleaning => write_int(f, "start_action", 0),
            Self::StartDumping => write_int(f, "start_action", 1),
            Self::StartOdorRemoval => write_int(f, "start_action", 2),
            Self::StartLeveling => write_int(f, "start_action", 4),
            Self::StartMaintenance => write_int(f, "start_action", 9),
            Self::EndMaintenance => write_int(f, "end_action", 9),
            Self::Stop { work_mode } => write_int(f, "stop_action", i64::from(work_mode.get())),
            Self::Continue { work_mode } => {
                write_int(f, "continue_action", i64::from(work_mode.get()))
            }
            Self::End { work_mode } => write_int(f, "end_action", i64::from(work_mode.get())),
            Self::Power(value) => write_int(f, "power_action", if *value { 1 } else { 0 }),
            Self::Mode(value) => write_int(f, "mode_action", i64::from(value.get())),
            Self::Other { setting, .. } => write_custom_setting(f, setting),
        }
    }
}

/// `control_device` payload for air purifiers (`K2`, `K3`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PurifierControl {
    /// `{"power_action": 0_or_1}`.
    Power(bool),
    /// `{"mode_action": <0..=3>}`.
    Mode(PurifierMode),
    Other {
        command_type: ControlCommandType,
        setting: CustomSetting,
    },
}

impl PurifierControl {
    pub fn command_type(&self) -> &'static str {
        match self {
            Self::Power(_) => "power",
            Self::Mode(_) => "mode",
            Self::Other { command_type, .. } => command_type.as_str(),
        }
    }
}

impl DisplayJson for PurifierControl {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::Power(value) => write_int(f, "power_action", if *value { 1 } else { 0 }),
            Self::Mode(value) => write_int(f, "mode_action", value.wire_value()),
            Self::Other { setting, .. } => write_custom_setting(f, setting),
        }
    }
}

/// Build the `kv` JSON string for a `DisplayJson` payload using nojson.
pub fn to_kv_string<T: DisplayJson>(value: &T) -> String {
    format!("{}", nojson::Json(value))
}

fn parse_json_array(raw_json: String, field: &'static str) -> Result<String, PetkitError> {
    let parsed = RawJsonOwned::parse(raw_json)?;
    if parsed.value().kind() != JsonValueKind::Array {
        return Err(PetkitError::InvalidArgument(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(parsed.text().to_string())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use alloc::{string::String, vec};

    use super::*;

    #[test]
    fn feeder_lightmode_serializes_as_int() {
        let kv = to_kv_string(&FeederSetting::LightMode(true));
        assert_eq!(kv, r#"{"lightMode":1}"#);
    }

    #[test]
    fn feeder_volume_serializes_as_int() {
        let kv = to_kv_string(&FeederSetting::Volume(
            VolumeLevel::new(7).expect("volume should be valid"),
        ));
        assert_eq!(kv, r#"{"volume":7}"#);
    }

    #[test]
    fn custom_setting_with_json_value_inlines_json() {
        let kv = to_kv_string(&LitterSetting::Other(
            CustomSetting::new(
                "schedule",
                CustomSettingValue::json("[1,2,3]").expect("raw JSON should parse"),
            )
            .expect("custom setting should be valid"),
        ));
        assert_eq!(kv, r#"{"schedule":[1,2,3]}"#);
    }

    #[test]
    fn custom_setting_with_string_value_quotes_it() {
        let kv = to_kv_string(&PurifierSetting::Other(
            CustomSetting::new(
                "mode",
                CustomSettingValue::String(
                    SettingString::new("auto").expect("setting string should be valid"),
                ),
            )
            .expect("custom setting should be valid"),
        ));
        assert_eq!(kv, r#"{"mode":"auto"}"#);
    }

    #[test]
    fn custom_setting_rejects_invalid_json_value() {
        assert!(CustomSettingValue::json("[1,2,3").is_err());
    }

    #[test]
    fn pet_weight() {
        let kv = to_kv_string(&PetSetting::Weight(
            PetWeightGrams::new(3500).expect("weight should be valid"),
        ));
        assert_eq!(kv, r#"{"weight":3500}"#);
    }

    #[test]
    fn feed_identifier_by_id_emits_only_id() {
        let payload = FeedIdentifier::by_id(42)
            .expect("id should be valid")
            .to_form();
        assert_eq!(
            payload.fields(),
            vec![(String::from("id"), String::from("42"))]
        );
    }

    #[test]
    fn feed_identifier_legacy_time_can_carry_name() {
        let payload = FeedIdentifier::by_time_with_name(730, "breakfast")
            .expect("time should be valid")
            .to_form();
        assert_eq!(
            payload.fields(),
            vec![
                (String::from("time"), String::from("730")),
                (String::from("name"), String::from("breakfast")),
            ]
        );
    }

    #[test]
    fn purifier_mode_serializes_from_typed_mode() {
        let kv = to_kv_string(&PurifierControl::Mode(PurifierMode::Strong));
        assert_eq!(kv, r#"{"mode_action":3}"#);
    }

    #[test]
    fn feed_daily_list_rejects_non_json_array() {
        assert!(FeedDailyList::from_json(r#"{"amount":10}"#).is_err());
        assert!(FeedDailyList::from_json(r#"[{"amount":10}]"#).is_ok());
    }

    #[test]
    fn repeat_schedule_rejects_invalid_json() {
        assert!(RepeatSchedule::new("[1,2,3").is_err());
    }
}
