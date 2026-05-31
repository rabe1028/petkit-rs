use petkit_types::{
    FountainAction, LbCommand, LitterControl, LitterWorkMode, PetkitError, PurifierControl,
    PurifierMode,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedAction {
    FeederManualFeed { amount: u16 },
    FeederManualFeedDual { amount1: u16, amount2: u16 },
    FeederCancelManualFeed,
    FeederCallPet,
    FeederFoodReplenished,
    FeederResetDesiccant,
    LitterControl(LitterControl),
    LitterResetN50Deodorizer,
    PurifierControl(PurifierControl),
    Fountain(FountainAction),
}

pub fn parse_action(action: &str, args: &[(&str, &str)]) -> Result<ParsedAction, PetkitError> {
    let action = normalize_action(action);
    match action.as_str() {
        "feed" | "manual_feed" => Ok(ParsedAction::FeederManualFeed {
            amount: parse_positive_u16(args, "amount")?,
        }),
        "feed_dual" | "manual_feed_dual" => {
            let amount1 = parse_u16(args, "amount1")?;
            let amount2 = parse_u16(args, "amount2")?;
            if amount1 == 0 && amount2 == 0 {
                return Err(PetkitError::InvalidArgument(String::from(
                    "manual_feed_dual needs amount1 or amount2",
                )));
            }
            Ok(ParsedAction::FeederManualFeedDual { amount1, amount2 })
        }
        "cancel_feed" | "cancel_manual_feed" => Ok(ParsedAction::FeederCancelManualFeed),
        "call_pet" => Ok(ParsedAction::FeederCallPet),
        "food_replenished" => Ok(ParsedAction::FeederFoodReplenished),
        "reset_desiccant" => Ok(ParsedAction::FeederResetDesiccant),
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
        "power" => Ok(ParsedAction::PurifierControl(PurifierControl::Power(
            parse_bool(args, "value")?,
        ))),
        "mode" => Ok(ParsedAction::PurifierControl(PurifierControl::Mode(
            parse_purifier_mode(args, "value")?,
        ))),
        _ => Err(PetkitError::InvalidArgument(format!(
            "unsupported Petkit control action `{control}`"
        ))),
    }
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
    args.iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map(|(_, value)| *value)
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

fn parse_i64(args: &[(&str, &str)], key: &'static str) -> Result<i64, PetkitError> {
    arg(args, key)
        .ok_or_else(|| PetkitError::InvalidArgument(format!("Petkit action requires `{key}`")))?
        .parse::<i64>()
        .map_err(|error| {
            PetkitError::InvalidArgument(format!(
                "Petkit action `{key}` must be an integer: {error}"
            ))
        })
}

fn parse_bool(args: &[(&str, &str)], key: &'static str) -> Result<bool, PetkitError> {
    let value = arg(args, key)
        .ok_or_else(|| PetkitError::InvalidArgument(format!("Petkit action requires `{key}`")))?;
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Ok(true),
        "0" | "false" | "off" | "no" => Ok(false),
        _ => Err(PetkitError::InvalidArgument(format!(
            "Petkit action `{key}` must be boolean"
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
    fn rejects_double_zero_dual_feed() {
        assert!(parse_action("manual_feed_dual", &[("amount1", "0"), ("amount2", "0")]).is_err());
    }

    #[test]
    fn parses_named_purifier_mode() {
        assert_eq!(
            parse_action("purifier_mode", &[("mode", "strong")]).expect("mode should parse"),
            ParsedAction::PurifierControl(PurifierControl::Mode(PurifierMode::Strong))
        );
    }
}
