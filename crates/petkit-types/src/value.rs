use alloc::format;
use alloc::string::String;

use core::fmt;
use core::num::{NonZeroU16, NonZeroU32, NonZeroU64};

use crate::{LbCommand, PetkitError};

macro_rules! non_zero_id {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(NonZeroU64);

        impl $name {
            pub fn new(value: u64) -> Result<Self, PetkitError> {
                NonZeroU64::new(value).map(Self).ok_or_else(|| {
                    PetkitError::InvalidArgument(format!("{} must be non-zero", $label))
                })
            }

            #[must_use]
            pub const fn get(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl TryFrom<u64> for $name {
            type Error = PetkitError;

            fn try_from(value: u64) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                value.get()
            }
        }
    };
}

non_zero_id!(DeviceId, "device_id");
non_zero_id!(PetId, "pet_id");
non_zero_id!(FeedEntryId, "feed entry id");
non_zero_id!(SoundId, "sound_id");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScheduleLimit(NonZeroU16);

impl ScheduleLimit {
    pub fn new(value: u16) -> Result<Self, PetkitError> {
        let value = NonZeroU16::new(value).ok_or_else(|| {
            PetkitError::InvalidArgument(String::from("schedule limit must be non-zero"))
        })?;
        if value.get() <= 200 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "schedule limit must be <= 200, got `{}`",
                value.get()
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

impl fmt::Display for ScheduleLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VolumeLevel(u8);

impl VolumeLevel {
    pub fn new(value: u8) -> Result<Self, PetkitError> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "volume must be in 0..=100, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeederSurplusGrams(u16);

impl FeederSurplusGrams {
    pub fn new(value: u16) -> Result<Self, PetkitError> {
        if (20..=100).contains(&value) {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "feeder surplus must be in 20..=100 grams, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LitterStillTimeSeconds(u16);

impl LitterStillTimeSeconds {
    pub fn new(value: u16) -> Result<Self, PetkitError> {
        if value <= 3600 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "litter still time must be <= 3600 seconds, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LitterSandType(u8);

impl LitterSandType {
    pub fn new(value: u8) -> Result<Self, PetkitError> {
        if value <= 10 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "litter sand type must be in 0..=10, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PetWeightGrams(NonZeroU32);

impl PetWeightGrams {
    pub fn new(value: u32) -> Result<Self, PetkitError> {
        let value = NonZeroU32::new(value).ok_or_else(|| {
            PetkitError::InvalidArgument(String::from("pet weight must be non-zero"))
        })?;
        if value.get() <= 200_000 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "pet weight must be <= 200000 grams, got `{}`",
                value.get()
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LitterWorkMode(u8);

impl LitterWorkMode {
    pub fn new(value: u8) -> Result<Self, PetkitError> {
        if value <= LbCommand::ResetN60Deodor as u8 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "litter work mode must be in 0..=10, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl From<LbCommand> for LitterWorkMode {
    fn from(value: LbCommand) -> Self {
        Self(value as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LitterModeValue(u8);

impl LitterModeValue {
    pub fn new(value: u8) -> Result<Self, PetkitError> {
        if value <= 10 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "litter mode value must be in 0..=10, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CalibrationAction(i32);

impl CalibrationAction {
    pub fn new(value: i32) -> Result<Self, PetkitError> {
        if matches!(value, 0 | 1) {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "calibration action must be 0 or 1, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RepeatDays(u8);

impl RepeatDays {
    pub fn new(bitmask: u8) -> Result<Self, PetkitError> {
        if bitmask != 0 && bitmask <= 0b0111_1111 {
            Ok(Self(bitmask))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "repeat days bitmask must be non-zero and use only 7 low bits, got `{bitmask}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ControlCommandType(&'static str);

impl ControlCommandType {
    pub const START: Self = Self("start");
    pub const STOP: Self = Self("stop");
    pub const CONTINUE: Self = Self("continue");
    pub const END: Self = Self("end");
    pub const POWER: Self = Self("power");
    pub const MODE: Self = Self("mode");

    pub fn custom(value: &'static str) -> Result<Self, PetkitError> {
        if value.trim().is_empty() {
            Err(PetkitError::InvalidArgument(String::from(
                "control command type must not be empty",
            )))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SettingInt(i64);

impl SettingInt {
    pub fn new(value: i64) -> Result<Self, PetkitError> {
        if value >= 0 {
            Ok(Self(value))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "setting integer must be non-negative, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SettingString(String);

impl SettingString {
    pub fn new(value: impl Into<String>) -> Result<Self, PetkitError> {
        let value = value.into();
        if value.is_empty() {
            Err(PetkitError::InvalidArgument(String::from(
                "setting string must not be empty",
            )))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PetkitDay(String);

impl PetkitDay {
    pub fn new(value: impl Into<String>) -> Result<Self, PetkitError> {
        let value = value.into();
        validate_yyyymmdd(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PetkitDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeedTime(u16);

impl FeedTime {
    pub fn new_hhmm(value: u32) -> Result<Self, PetkitError> {
        let hour = value / 100;
        let minute = value % 100;
        if hour <= 23 && minute <= 59 {
            Ok(Self(value as u16))
        } else {
            Err(PetkitError::InvalidArgument(format!(
                "feed time must be HHMM in 0000..2359 with minutes <= 59, got `{value}`"
            )))
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for FeedTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn validate_yyyymmdd(value: &str) -> Result<(), PetkitError> {
    if value.len() != 8 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(PetkitError::InvalidArgument(format!(
            "day must be an 8 digit YYYYMMDD string, got `{value}`"
        )));
    }

    let year = parse_decimal(&value[0..4]);
    let month = parse_decimal(&value[4..6]);
    let day = parse_decimal(&value[6..8]);
    if !(1..=12).contains(&month) {
        return Err(PetkitError::InvalidArgument(format!(
            "month must be in 1..=12, got `{month}`"
        )));
    }

    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        return Err(PetkitError::InvalidArgument(format!(
            "day must be in 1..={max_day} for {year:04}{month:02}, got `{day}`"
        )));
    }

    Ok(())
}

fn parse_decimal(value: &str) -> u32 {
    value
        .bytes()
        .fold(0, |acc, byte| acc * 10 + u32::from(byte - b'0'))
}

const fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

const fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn petkit_day_rejects_impossible_dates() {
        assert!(PetkitDay::new("20260229").is_err());
        assert!(PetkitDay::new("20260228").is_ok());
        assert!(PetkitDay::new("20240229").is_ok());
    }

    #[test]
    fn feed_time_rejects_invalid_hhmm() {
        assert!(FeedTime::new_hhmm(2360).is_err());
        assert!(FeedTime::new_hhmm(2400).is_err());
        assert!(FeedTime::new_hhmm(2359).is_ok());
    }

    #[test]
    fn ids_reject_zero() {
        assert!(DeviceId::new(0).is_err());
        assert!(PetId::new(0).is_err());
        assert!(FeedEntryId::new(0).is_err());
    }

    #[test]
    fn bounded_values_reject_out_of_range_values() {
        assert!(ScheduleLimit::new(0).is_err());
        assert!(ScheduleLimit::new(201).is_err());
        assert!(VolumeLevel::new(101).is_err());
        assert!(FeederSurplusGrams::new(19).is_err());
        assert!(LitterStillTimeSeconds::new(3601).is_err());
        assert!(PetWeightGrams::new(0).is_err());
        assert!(LitterWorkMode::new(11).is_err());
        assert!(CalibrationAction::new(2).is_err());
        assert!(RepeatDays::new(0).is_err());
        assert!(RepeatDays::new(0b1000_0000).is_err());
        assert!(SettingInt::new(-1).is_err());
        assert!(SettingString::new("").is_err());
    }
}
