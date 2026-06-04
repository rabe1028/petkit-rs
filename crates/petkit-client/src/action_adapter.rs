use petkit_types::{
    CameraRtmCommand, CustomSetting, CustomSettingValue, FeederSetting, FeederSurplusGrams,
    FountainAction, LbCommand, LitterControl, LitterModeValue, LitterWorkMode, PetkitError,
    PtzDirection, PtzKind, PurifierControl, PurifierMode, SettingInt, SettingString, SoundId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedAction {
    FeederManualFeed {
        amount: u16,
    },
    FeederManualFeedDual {
        amount1: u16,
        amount2: u16,
    },
    FeederCancelManualFeed,
    FeederCallPet,
    FeederPlaySound(SoundId),
    FeederFoodReplenished,
    FeederResetDesiccant,
    FeederUpdateSetting(FeederSetting),
    LitterControl(LitterControl),
    LitterResetN50Deodorizer,
    PurifierControl(PurifierControl),
    Fountain(FountainAction),
    CameraRtm(CameraRtmCommand),
    /// Family dispatch is intentionally left to the caller because Petkit has
    /// separate update-setting endpoints for feeder, litter, purifier, etc.
    UpdateSetting(CustomSetting),
}

pub fn parse_action(action: &str, args: &[(&str, &str)]) -> Result<ParsedAction, PetkitError> {
    let action = normalize_action(action);
    match action.as_str() {
        "feed" | "manual_feed" => Ok(ParsedAction::FeederManualFeed {
            amount: parse_positive_u16(args, "amount")?,
        }),
        "feed_dual" | "manual_feed_dual" => {
            let amount1 = parse_optional_u16(args, "amount1")?.unwrap_or(0);
            let amount2 = parse_optional_u16(args, "amount2")?.unwrap_or(0);
            if amount1 == 0 && amount2 == 0 {
                return Err(PetkitError::InvalidArgument(String::from(
                    "manual_feed_dual needs amount1 or amount2",
                )));
            }
            Ok(ParsedAction::FeederManualFeedDual { amount1, amount2 })
        }
        "cancel_feed" | "cancel_manual_feed" => Ok(ParsedAction::FeederCancelManualFeed),
        "call_pet" => Ok(ParsedAction::FeederCallPet),
        "play_sound" => Ok(ParsedAction::FeederPlaySound(SoundId::new(parse_u64_any(
            args,
            &["sound", "sound_id"],
        )?)?)),
        "food_replenished" => Ok(ParsedAction::FeederFoodReplenished),
        "reset_desiccant" => Ok(ParsedAction::FeederResetDesiccant),
        "surplus_level" | "surplus_grams" | "surplus" => Ok(ParsedAction::FeederUpdateSetting(
            FeederSetting::Surplus(FeederSurplusGrams::new(parse_u16_any(
                args,
                &["value", "grams", "level", "surplus", "surplus_grams"],
            )?)?),
        )),
        "update_setting" => Ok(ParsedAction::UpdateSetting(parse_custom_setting(args)?)),
        "camera_ptz" => {
            let direction = parse_ptz_direction(args)?;
            Ok(ParsedAction::CameraRtm(CameraRtmCommand::PtzControl {
                kind: ptz_kind_for_direction(direction),
                direction,
            }))
        }
        "camera_heartbeat" => Ok(ParsedAction::CameraRtm(CameraRtmCommand::Heartbeat)),
        "camera_start_live" => Ok(ParsedAction::CameraRtm(CameraRtmCommand::StartLive {
            is_sd: parse_optional_bool(args, "is_sd")?.unwrap_or(false),
        })),
        "camera_stop_live" => Ok(ParsedAction::CameraRtm(CameraRtmCommand::StopLive)),
        "litterbox_clean" | "scoop" => {
            Ok(ParsedAction::LitterControl(LitterControl::StartCleaning))
        }
        "litterbox_dump" | "dump_litter" => {
            Ok(ParsedAction::LitterControl(LitterControl::StartDumping))
        }
        "litterbox_deodorize" | "deodorize" => {
            Ok(ParsedAction::LitterControl(LitterControl::StartOdorRemoval))
        }
        "litterbox_level" | "level_litter" => {
            Ok(ParsedAction::LitterControl(LitterControl::StartLeveling))
        }
        "litterbox_maintenance" => Ok(ParsedAction::LitterControl(LitterControl::StartMaintenance)),
        "litterbox_exit_maintenance" => {
            Ok(ParsedAction::LitterControl(LitterControl::EndMaintenance))
        }
        "litterbox_reset" => parse_litterbox_reset(args),
        "purifier_power" => Ok(ParsedAction::PurifierControl(PurifierControl::Power(
            parse_bool(args, "on")?,
        ))),
        "purifier_mode" => Ok(ParsedAction::PurifierControl(PurifierControl::Mode(
            parse_purifier_mode(args, "mode")?,
        ))),
        "fountain_power" => Ok(ParsedAction::Fountain(if parse_bool(args, "on")? {
            FountainAction::PowerOn
        } else {
            FountainAction::PowerOff
        })),
        "fountain_reset_filter" | "reset_filter" => {
            Ok(ParsedAction::Fountain(FountainAction::ResetFilter))
        }
        "fountain_pause" | "pause" => Ok(ParsedAction::Fountain(FountainAction::Pause)),
        "fountain_resume" | "fountain_continue" | "resume" | "continue" => {
            Ok(ParsedAction::Fountain(FountainAction::Continue))
        }
        "control_device" => parse_control_device(args),
        _ => Err(PetkitError::InvalidArgument(format!(
            "unsupported Petkit action `{action}`"
        ))),
    }
}

fn parse_custom_setting(args: &[(&str, &str)]) -> Result<CustomSetting, PetkitError> {
    let key = arg_any(args, &["key", "setting", "name"]).ok_or_else(|| {
        PetkitError::InvalidArgument(String::from("update_setting requires `key`"))
    })?;
    let value = parse_custom_setting_value(args)?;
    CustomSetting::new(key, value)
}

fn parse_custom_setting_value(args: &[(&str, &str)]) -> Result<CustomSettingValue, PetkitError> {
    if let Some(raw_json) = arg_any(args, &["value_json", "json"]) {
        return Ok(CustomSettingValue::json(raw_json)?);
    }

    let raw = arg_any(args, &["value", "setting_value"]).ok_or_else(|| {
        PetkitError::InvalidArgument(String::from("update_setting requires `value`"))
    })?;
    let value_type = arg_any(args, &["type", "value_type"]).map(normalize_action);
    match value_type.as_deref() {
        Some("bool" | "boolean") => {
            return Ok(CustomSettingValue::BoolAsInt(parse_bool_value(raw)?));
        }
        Some("int" | "integer" | "number") => {
            return Ok(CustomSettingValue::Int(SettingInt::new(parse_i64_value(
                "value", raw,
            )?)?));
        }
        Some("json") => return Ok(CustomSettingValue::json(raw)?),
        Some("string" | "text") => {
            return Ok(CustomSettingValue::String(SettingString::new(raw)?));
        }
        Some(other) => {
            return Err(PetkitError::InvalidArgument(format!(
                "unsupported update_setting value type `{other}`"
            )));
        }
        None => {}
    }

    if raw.trim_start().starts_with(['{', '[']) {
        return Ok(CustomSettingValue::json(raw)?);
    }
    if let Ok(value) = raw.parse::<i64>() {
        return Ok(CustomSettingValue::Int(SettingInt::new(value)?));
    }
    if let Ok(value) = parse_bool_value(raw) {
        return Ok(CustomSettingValue::BoolAsInt(value));
    }
    Ok(CustomSettingValue::String(SettingString::new(raw)?))
}

fn parse_litterbox_reset(args: &[(&str, &str)]) -> Result<ParsedAction, PetkitError> {
    match arg(args, "target")
        .unwrap_or("resetting")
        .to_ascii_lowercase()
        .as_str()
    {
        "n50" | "deodorizer" | "odor_n50" => Ok(ParsedAction::LitterResetN50Deodorizer),
        "n60" | "deodorizer_n60" | "odor_n60" => {
            Ok(ParsedAction::LitterControl(LitterControl::Other {
                command_type: petkit_types::ControlCommandType::START,
                setting: petkit_types::CustomSetting::new(
                    petkit_types::DeviceAction::Start.as_str(),
                    petkit_types::CustomSettingValue::Int(petkit_types::SettingInt::new(
                        i64::from(LbCommand::ResetN60Deodor as u8),
                    )?),
                )?,
            }))
        }
        "level" | "litter" | "litter_level" => {
            Ok(ParsedAction::LitterControl(LitterControl::StartLeveling))
        }
        _ => Ok(ParsedAction::LitterControl(LitterControl::Other {
            command_type: petkit_types::ControlCommandType::START,
            setting: petkit_types::CustomSetting::new(
                petkit_types::DeviceAction::Start.as_str(),
                petkit_types::CustomSettingValue::Int(petkit_types::SettingInt::new(i64::from(
                    LbCommand::Resetting as u8,
                ))?),
            )?,
        })),
    }
}

fn parse_control_device(args: &[(&str, &str)]) -> Result<ParsedAction, PetkitError> {
    let family = control_device_family(args);
    let control = normalize_action(
        arg(args, "control")
            .or_else(|| arg(args, "control_action"))
            .ok_or_else(|| {
                PetkitError::InvalidArgument(String::from("control_device requires `control`"))
            })?,
    );
    match control.as_str() {
        "start" => Ok(ParsedAction::LitterControl(LitterControl::Other {
            command_type: petkit_types::ControlCommandType::START,
            setting: custom_litter_command("start_action", parse_i64(args, "value")?)?,
        })),
        "stop" => Ok(ParsedAction::LitterControl(LitterControl::Stop {
            work_mode: LitterWorkMode::new(parse_u8(args, "value")?)?,
        })),
        "continue" => Ok(ParsedAction::LitterControl(LitterControl::Continue {
            work_mode: LitterWorkMode::new(parse_u8(args, "value")?)?,
        })),
        "end" => Ok(ParsedAction::LitterControl(LitterControl::End {
            work_mode: LitterWorkMode::new(parse_u8(args, "value")?)?,
        })),
        "power" => {
            let power = parse_bool(args, "value")?;
            if family == Some(ControlDeviceFamily::Purifier) {
                Ok(ParsedAction::PurifierControl(PurifierControl::Power(power)))
            } else {
                Ok(ParsedAction::LitterControl(LitterControl::Power(power)))
            }
        }
        "mode" => {
            if family == Some(ControlDeviceFamily::Purifier) {
                Ok(ParsedAction::PurifierControl(PurifierControl::Mode(
                    parse_purifier_mode(args, "value")?,
                )))
            } else {
                Ok(ParsedAction::LitterControl(LitterControl::Mode(
                    LitterModeValue::new(parse_u8(args, "value")?)?,
                )))
            }
        }
        _ => Err(PetkitError::InvalidArgument(format!(
            "unsupported Petkit control action `{control}`"
        ))),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlDeviceFamily {
    Litter,
    Purifier,
}

fn control_device_family(args: &[(&str, &str)]) -> Option<ControlDeviceFamily> {
    let family = arg_any(
        args,
        &["family", "device_family", "device_type", "deviceType"],
    )?;
    let family = normalize_action(family);
    if matches!(family.as_str(), "purifier" | "air_purifier" | "k2" | "k3") {
        return Some(ControlDeviceFamily::Purifier);
    }
    if matches!(
        family.as_str(),
        "litter" | "litter_box" | "litterbox" | "toilet" | "pura" | "pura_max" | "pura_x"
    ) {
        return Some(ControlDeviceFamily::Litter);
    }
    None
}

fn custom_litter_command(
    key: &'static str,
    value: i64,
) -> Result<petkit_types::CustomSetting, PetkitError> {
    petkit_types::CustomSetting::new(
        key,
        petkit_types::CustomSettingValue::Int(petkit_types::SettingInt::new(value)?),
    )
}

fn normalize_action(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

fn arg<'a>(args: &'a [(&str, &str)], key: &str) -> Option<&'a str> {
    let normalized_key = normalize_action(key);
    args.iter()
        .find(|(name, _)| normalize_action(name) == normalized_key)
        .map(|(_, value)| *value)
}

fn arg_any<'a>(args: &'a [(&str, &str)], keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| arg(args, key))
}

fn parse_positive_u16(args: &[(&str, &str)], key: &'static str) -> Result<u16, PetkitError> {
    let value = parse_u16(args, key)?;
    if value == 0 {
        Err(PetkitError::InvalidArgument(format!(
            "Petkit action `{key}` must be positive"
        )))
    } else {
        Ok(value)
    }
}

fn parse_u16(args: &[(&str, &str)], key: &'static str) -> Result<u16, PetkitError> {
    parse_u64(args, key).and_then(|value| {
        u16::try_from(value).map_err(|error| {
            PetkitError::InvalidArgument(format!("Petkit action `{key}` is out of range: {error}"))
        })
    })
}

fn parse_optional_u16(
    args: &[(&str, &str)],
    key: &'static str,
) -> Result<Option<u16>, PetkitError> {
    let Some(value) = arg(args, key) else {
        return Ok(None);
    };
    let value = value.parse::<u64>().map_err(|error| {
        PetkitError::InvalidArgument(format!("Petkit action `{key}` must be an integer: {error}"))
    })?;
    u16::try_from(value).map(Some).map_err(|error| {
        PetkitError::InvalidArgument(format!("Petkit action `{key}` is out of range: {error}"))
    })
}

fn parse_u16_any(args: &[(&str, &str)], keys: &[&'static str]) -> Result<u16, PetkitError> {
    parse_u64_any(args, keys).and_then(|value| {
        u16::try_from(value).map_err(|error| {
            PetkitError::InvalidArgument(format!(
                "Petkit action `{}` is out of range: {error}",
                keys[0]
            ))
        })
    })
}

fn parse_u8(args: &[(&str, &str)], key: &'static str) -> Result<u8, PetkitError> {
    parse_u64(args, key).and_then(|value| {
        u8::try_from(value).map_err(|error| {
            PetkitError::InvalidArgument(format!("Petkit action `{key}` is out of range: {error}"))
        })
    })
}

fn parse_u64(args: &[(&str, &str)], key: &'static str) -> Result<u64, PetkitError> {
    arg(args, key)
        .ok_or_else(|| PetkitError::InvalidArgument(format!("Petkit action requires `{key}`")))?
        .parse::<u64>()
        .map_err(|error| {
            PetkitError::InvalidArgument(format!(
                "Petkit action `{key}` must be an integer: {error}"
            ))
        })
}

fn parse_u64_any(args: &[(&str, &str)], keys: &[&'static str]) -> Result<u64, PetkitError> {
    let key_list = keys.join("` or `");
    arg_any(args, keys)
        .ok_or_else(|| {
            PetkitError::InvalidArgument(format!("Petkit action requires `{key_list}`"))
        })?
        .parse::<u64>()
        .map_err(|error| {
            PetkitError::InvalidArgument(format!(
                "Petkit action `{}` must be an integer: {error}",
                keys[0]
            ))
        })
}

fn parse_i64(args: &[(&str, &str)], key: &'static str) -> Result<i64, PetkitError> {
    let value = arg(args, key)
        .ok_or_else(|| PetkitError::InvalidArgument(format!("Petkit action requires `{key}`")))?;
    parse_i64_value(key, value)
}

fn parse_i64_value(key: &'static str, value: &str) -> Result<i64, PetkitError> {
    value.parse::<i64>().map_err(|error| {
        PetkitError::InvalidArgument(format!("Petkit action `{key}` must be an integer: {error}"))
    })
}

fn parse_bool(args: &[(&str, &str)], key: &'static str) -> Result<bool, PetkitError> {
    let value = arg(args, key)
        .ok_or_else(|| PetkitError::InvalidArgument(format!("Petkit action requires `{key}`")))?;
    parse_bool_value(value)
        .map_err(|_| PetkitError::InvalidArgument(format!("Petkit action `{key}` must be boolean")))
}

fn parse_optional_bool(
    args: &[(&str, &str)],
    key: &'static str,
) -> Result<Option<bool>, PetkitError> {
    let Some(value) = arg(args, key) else {
        return Ok(None);
    };
    parse_bool_value(value)
        .map(Some)
        .map_err(|_| PetkitError::InvalidArgument(format!("Petkit action `{key}` must be boolean")))
}

fn parse_bool_value(value: &str) -> Result<bool, PetkitError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Ok(true),
        "0" | "false" | "off" | "no" => Ok(false),
        _ => Err(PetkitError::InvalidArgument(String::from(
            "Petkit action value must be boolean",
        ))),
    }
}

fn parse_purifier_mode(
    args: &[(&str, &str)],
    key: &'static str,
) -> Result<PurifierMode, PetkitError> {
    let value = arg(args, key)
        .ok_or_else(|| PetkitError::InvalidArgument(format!("Petkit action requires `{key}`")))?;
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" | "0" => Ok(PurifierMode::Auto),
        "silent" | "sleep" | "1" => Ok(PurifierMode::Silent),
        "standard" | "normal" | "2" => Ok(PurifierMode::Standard),
        "strong" | "turbo" | "3" => Ok(PurifierMode::Strong),
        _ => Err(PetkitError::InvalidArgument(format!(
            "unsupported purifier mode `{value}`"
        ))),
    }
}

fn parse_ptz_direction(args: &[(&str, &str)]) -> Result<PtzDirection, PetkitError> {
    let value = arg_any(args, &["direction", "ptz_dir", "dir"]).ok_or_else(|| {
        PetkitError::InvalidArgument(String::from("camera_ptz requires direction"))
    })?;
    match normalize_action(value).as_str() {
        "up" => Ok(PtzDirection::Up),
        "down" => Ok(PtzDirection::Down),
        "left" => Ok(PtzDirection::Left),
        "right" => Ok(PtzDirection::Right),
        "stop" => Ok(PtzDirection::Stop),
        raw => raw
            .parse::<i64>()
            .map(PtzDirection::Custom)
            .map_err(|error| {
                PetkitError::InvalidArgument(format!(
                    "unsupported camera_ptz direction `{value}`: {error}"
                ))
            }),
    }
}

fn ptz_kind_for_direction(direction: PtzDirection) -> PtzKind {
    match direction {
        PtzDirection::Stop => PtzKind::Stop,
        PtzDirection::Up
        | PtzDirection::Down
        | PtzDirection::Left
        | PtzDirection::Right
        | PtzDirection::Custom(_) => PtzKind::Move,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_sidecar_action_names() {
        assert_eq!(
            parse_action("feed", &[("amount", "10")]).expect("feed should parse"),
            ParsedAction::FeederManualFeed { amount: 10 }
        );
        assert_eq!(
            parse_action("litterbox-clean", &[]).expect("clean should parse"),
            ParsedAction::LitterControl(LitterControl::StartCleaning)
        );
        assert_eq!(
            parse_action("purifier_power", &[("on", "true")]).expect("power should parse"),
            ParsedAction::PurifierControl(PurifierControl::Power(true))
        );
        assert_eq!(
            parse_action("fountain_reset_filter", &[]).expect("fountain should parse"),
            ParsedAction::Fountain(FountainAction::ResetFilter)
        );
    }

    #[test]
    fn parses_play_sound_aliases_with_sound_id_validation() {
        assert_eq!(
            parse_action("play_sound", &[("sound", "7")]).expect("play_sound should parse"),
            ParsedAction::FeederPlaySound(SoundId::new(7).expect("sound id should be valid"))
        );
        assert_eq!(
            parse_action("play-sound", &[("sound-id", "8")])
                .expect("play-sound alias should parse"),
            ParsedAction::FeederPlaySound(SoundId::new(8).expect("sound id should be valid"))
        );
        assert!(parse_action("play_sound", &[("sound_id", "0")]).is_err());
    }

    #[test]
    fn parses_surplus_level_aliases_with_feeder_validation() {
        assert_eq!(
            parse_action("surplus_level", &[("value", "20")]).expect("surplus_level should parse"),
            ParsedAction::FeederUpdateSetting(FeederSetting::Surplus(
                FeederSurplusGrams::new(20).expect("surplus should be valid")
            ))
        );
        assert_eq!(
            parse_action("surplus-level", &[("grams", "100")])
                .expect("surplus-level alias should parse"),
            ParsedAction::FeederUpdateSetting(FeederSetting::Surplus(
                FeederSurplusGrams::new(100).expect("surplus should be valid")
            ))
        );
        assert!(parse_action("surplus_level", &[("value", "19")]).is_err());
    }

    #[test]
    fn parses_generic_update_setting_for_caller_dispatch() {
        assert_eq!(
            parse_action("update_setting", &[("key", "feedNotify"), ("value", "1")])
                .expect("update_setting should parse"),
            ParsedAction::UpdateSetting(
                CustomSetting::new(
                    "feedNotify",
                    CustomSettingValue::Int(SettingInt::new(1).expect("int should be valid"))
                )
                .expect("custom setting should be valid")
            )
        );
        assert_eq!(
            parse_action(
                "update-setting",
                &[
                    ("key", "settings.window"),
                    ("value", "true"),
                    ("type", "bool")
                ]
            )
            .expect("update-setting should parse"),
            ParsedAction::UpdateSetting(
                CustomSetting::new("settings.window", CustomSettingValue::BoolAsInt(true))
                    .expect("custom setting should be valid")
            )
        );
        assert!(
            parse_action(
                "update_setting",
                &[("key", "settings.raw"), ("value_json", "{\"mode\":2}")]
            )
            .is_ok()
        );
    }

    #[test]
    fn routes_generic_control_device_power_by_family() {
        assert_eq!(
            parse_action(
                "control_device",
                &[
                    ("control", "power"),
                    ("value", "1"),
                    ("family", "litterbox")
                ]
            )
            .expect("litter control power should parse"),
            ParsedAction::LitterControl(LitterControl::Power(true))
        );
        assert_eq!(
            parse_action(
                "control_device",
                &[("control", "power"), ("value", "0"), ("family", "purifier")]
            )
            .expect("purifier control power should parse"),
            ParsedAction::PurifierControl(PurifierControl::Power(false))
        );
        assert_eq!(
            parse_action("control_device", &[("control", "power"), ("value", "true")])
                .expect("unscoped control power should stay in control_device family"),
            ParsedAction::LitterControl(LitterControl::Power(true))
        );
    }

    #[test]
    fn parses_camera_ptz_as_rtm_command() {
        assert_eq!(
            parse_action("camera_ptz", &[("direction", "left")]).expect("camera ptz should parse"),
            ParsedAction::CameraRtm(CameraRtmCommand::PtzControl {
                kind: PtzKind::Move,
                direction: PtzDirection::Left
            })
        );
    }

    #[test]
    fn parses_camera_ptz_stop_as_stop_command_type() {
        assert_eq!(
            parse_action("camera_ptz", &[("direction", "stop")]).expect("camera ptz should parse"),
            ParsedAction::CameraRtm(CameraRtmCommand::PtzControl {
                kind: PtzKind::Stop,
                direction: PtzDirection::Stop
            })
        );
    }

    #[test]
    fn rejects_double_zero_dual_feed() {
        assert!(parse_action("manual_feed_dual", &[("amount1", "0"), ("amount2", "0")]).is_err());
        assert!(parse_action("manual_feed_dual", &[]).is_err());
    }

    #[test]
    fn defaults_missing_dual_feed_hopper_to_zero() {
        assert_eq!(
            parse_action("manual_feed_dual", &[("amount1", "5")])
                .expect("one-sided amount1 should parse"),
            ParsedAction::FeederManualFeedDual {
                amount1: 5,
                amount2: 0
            }
        );
        assert_eq!(
            parse_action("manual-feed-dual", &[("amount2", "6")])
                .expect("one-sided amount2 should parse"),
            ParsedAction::FeederManualFeedDual {
                amount1: 0,
                amount2: 6
            }
        );
        assert!(parse_action("manual_feed_dual", &[("amount1", "not-an-int")]).is_err());
    }

    #[test]
    fn parses_named_purifier_mode() {
        assert_eq!(
            parse_action("purifier_mode", &[("mode", "strong")]).expect("mode should parse"),
            ParsedAction::PurifierControl(PurifierControl::Mode(PurifierMode::Strong))
        );
    }
}
