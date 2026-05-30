use alloc::string::{String, ToString};

use nojson::{JsonParseError, RawJsonValue};

use crate::PetkitError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceFamily {
    Feeder,
    LitterBox,
    WaterFountain,
    Purifier,
    Cozy,
    Pet,
    Unknown,
}

/// Untyped device-type enum. Use [`DeviceType::into_family`] (or the
/// per-family `TryFrom<DeviceType>` impls) to convert to a strongly typed
/// `*DeviceType` for compile-time API constraints.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Cozy,
    Feeder,
    FeederMini,
    D3,
    D4,
    D4s,
    D4h,
    D4sh,
    T3,
    T4,
    T5,
    T6,
    T7,
    W4,
    W5,
    Ctw2,
    Ctw3,
    K2,
    K3,
    Pet,
    Unknown(String),
}

impl DeviceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Cozy => "cozy",
            Self::Feeder => "feeder",
            Self::FeederMini => "feedermini",
            Self::D3 => "d3",
            Self::D4 => "d4",
            Self::D4s => "d4s",
            Self::D4h => "d4h",
            Self::D4sh => "d4sh",
            Self::T3 => "t3",
            Self::T4 => "t4",
            Self::T5 => "t5",
            Self::T6 => "t6",
            Self::T7 => "t7",
            Self::W4 => "w4",
            Self::W5 => "w5",
            Self::Ctw2 => "ctw2",
            Self::Ctw3 => "ctw3",
            Self::K2 => "k2",
            Self::K3 => "k3",
            Self::Pet => "pet",
            Self::Unknown(value) => value.as_str(),
        }
    }

    pub const fn family(&self) -> DeviceFamily {
        match self {
            Self::Cozy => DeviceFamily::Cozy,
            Self::Feeder
            | Self::FeederMini
            | Self::D3
            | Self::D4
            | Self::D4s
            | Self::D4h
            | Self::D4sh => DeviceFamily::Feeder,
            Self::T3 | Self::T4 | Self::T5 | Self::T6 | Self::T7 => DeviceFamily::LitterBox,
            Self::W4 | Self::W5 | Self::Ctw2 | Self::Ctw3 => DeviceFamily::WaterFountain,
            Self::K2 | Self::K3 => DeviceFamily::Purifier,
            Self::Pet => DeviceFamily::Pet,
            Self::Unknown(_) => DeviceFamily::Unknown,
        }
    }

    pub const fn supports_camera(&self) -> bool {
        matches!(
            self,
            Self::D4h | Self::D4sh | Self::T5 | Self::T6 | Self::T7
        )
    }

    /// Convert to the strongly-typed per-family enum.
    pub fn into_family(self) -> DeviceFamilyKind {
        match self {
            Self::Cozy => DeviceFamilyKind::Cozy,
            Self::Feeder => DeviceFamilyKind::Feeder(FeederDeviceType::Feeder),
            Self::FeederMini => DeviceFamilyKind::Feeder(FeederDeviceType::FeederMini),
            Self::D3 => DeviceFamilyKind::Feeder(FeederDeviceType::D3),
            Self::D4 => DeviceFamilyKind::Feeder(FeederDeviceType::D4),
            Self::D4s => DeviceFamilyKind::Feeder(FeederDeviceType::D4s),
            Self::D4h => DeviceFamilyKind::Feeder(FeederDeviceType::D4h),
            Self::D4sh => DeviceFamilyKind::Feeder(FeederDeviceType::D4sh),
            Self::T3 => DeviceFamilyKind::Litter(LitterDeviceType::T3),
            Self::T4 => DeviceFamilyKind::Litter(LitterDeviceType::T4),
            Self::T5 => DeviceFamilyKind::Litter(LitterDeviceType::T5),
            Self::T6 => DeviceFamilyKind::Litter(LitterDeviceType::T6),
            Self::T7 => DeviceFamilyKind::Litter(LitterDeviceType::T7),
            Self::W4 => DeviceFamilyKind::Fountain(FountainDeviceType::W4),
            Self::W5 => DeviceFamilyKind::Fountain(FountainDeviceType::W5),
            Self::Ctw2 => DeviceFamilyKind::Fountain(FountainDeviceType::Ctw2),
            Self::Ctw3 => DeviceFamilyKind::Fountain(FountainDeviceType::Ctw3),
            Self::K2 => DeviceFamilyKind::Purifier(PurifierDeviceType::K2),
            Self::K3 => DeviceFamilyKind::Purifier(PurifierDeviceType::K3),
            Self::Pet => DeviceFamilyKind::Pet,
            Self::Unknown(value) => DeviceFamilyKind::Unknown(value),
        }
    }
}

impl From<String> for DeviceType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "cozy" => Self::Cozy,
            "feeder" => Self::Feeder,
            "feedermini" => Self::FeederMini,
            "d3" => Self::D3,
            "d4" => Self::D4,
            "d4s" => Self::D4s,
            "d4h" => Self::D4h,
            "d4sh" => Self::D4sh,
            "t3" => Self::T3,
            "t4" => Self::T4,
            "t5" => Self::T5,
            "t6" => Self::T6,
            "t7" => Self::T7,
            "w4" => Self::W4,
            "w5" => Self::W5,
            "ctw2" => Self::Ctw2,
            "ctw3" => Self::Ctw3,
            "k2" => Self::K2,
            "k3" => Self::K3,
            "pet" => Self::Pet,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl From<DeviceType> for String {
    fn from(value: DeviceType) -> Self {
        match value {
            DeviceType::Unknown(value) => value,
            other => other.as_str().to_string(),
        }
    }
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for DeviceType {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let raw: String = value.try_into()?;
        Ok(Self::from(raw))
    }
}

/// Result of [`DeviceType::into_family`]: routes a `DeviceType` to its
/// strongly typed family enum.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceFamilyKind {
    Feeder(FeederDeviceType),
    Litter(LitterDeviceType),
    Fountain(FountainDeviceType),
    Purifier(PurifierDeviceType),
    Cozy,
    Pet,
    Unknown(String),
}

/// Feeder family device types: `Feeder`, `FeederMini`, `D3`, `D4`, `D4s`, `D4h`, `D4sh`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeederDeviceType {
    Feeder,
    FeederMini,
    D3,
    D4,
    D4s,
    D4h,
    D4sh,
}

impl FeederDeviceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Feeder => "feeder",
            Self::FeederMini => "feedermini",
            Self::D3 => "d3",
            Self::D4 => "d4",
            Self::D4s => "d4s",
            Self::D4h => "d4h",
            Self::D4sh => "d4sh",
        }
    }

    pub const fn is_dual_hopper(self) -> bool {
        matches!(self, Self::D4s | Self::D4sh)
    }

    pub const fn supports_camera(self) -> bool {
        matches!(self, Self::D4h | Self::D4sh)
    }

    pub const fn uses_legacy_manual_feed_endpoint(self) -> bool {
        matches!(self, Self::Feeder | Self::FeederMini)
    }

    pub const fn uses_legacy_desiccant_endpoint(self) -> bool {
        matches!(self, Self::Feeder | Self::FeederMini)
    }

    pub const fn uses_legacy_update_setting_endpoint(self) -> bool {
        matches!(self, Self::FeederMini)
    }

    pub const fn uses_legacy_schedule_endpoint(self) -> bool {
        matches!(self, Self::Feeder | Self::FeederMini)
    }

    pub const fn uses_legacy_suspend_feed_endpoint(self) -> bool {
        matches!(self, Self::Feeder | Self::FeederMini)
    }
}

impl From<FeederDeviceType> for DeviceType {
    fn from(value: FeederDeviceType) -> Self {
        match value {
            FeederDeviceType::Feeder => Self::Feeder,
            FeederDeviceType::FeederMini => Self::FeederMini,
            FeederDeviceType::D3 => Self::D3,
            FeederDeviceType::D4 => Self::D4,
            FeederDeviceType::D4s => Self::D4s,
            FeederDeviceType::D4h => Self::D4h,
            FeederDeviceType::D4sh => Self::D4sh,
        }
    }
}

impl TryFrom<DeviceType> for FeederDeviceType {
    type Error = PetkitError;
    fn try_from(value: DeviceType) -> Result<Self, Self::Error> {
        match value {
            DeviceType::Feeder => Ok(Self::Feeder),
            DeviceType::FeederMini => Ok(Self::FeederMini),
            DeviceType::D3 => Ok(Self::D3),
            DeviceType::D4 => Ok(Self::D4),
            DeviceType::D4s => Ok(Self::D4s),
            DeviceType::D4h => Ok(Self::D4h),
            DeviceType::D4sh => Ok(Self::D4sh),
            other => Err(PetkitError::InvalidArgument(alloc::format!(
                "device `{}` is not a feeder",
                other.as_str()
            ))),
        }
    }
}

/// Litter box family device types: `T3`, `T4`, `T5`, `T6`, `T7`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LitterDeviceType {
    T3,
    T4,
    T5,
    T6,
    T7,
}

impl LitterDeviceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::T3 => "t3",
            Self::T4 => "t4",
            Self::T5 => "t5",
            Self::T6 => "t6",
            Self::T7 => "t7",
        }
    }

    pub const fn supports_camera(self) -> bool {
        matches!(self, Self::T5 | Self::T6 | Self::T7)
    }

    /// True for litter boxes that ship with an N50 deodorizer module.
    pub const fn supports_n50_deodorizer(self) -> bool {
        matches!(self, Self::T4 | Self::T5 | Self::T6)
    }

    /// True for litter boxes that ship with an N60 deodorant spray system.
    pub const fn supports_n60_deodorizer(self) -> bool {
        matches!(self, Self::T5 | Self::T6 | Self::T7)
    }
}

impl From<LitterDeviceType> for DeviceType {
    fn from(value: LitterDeviceType) -> Self {
        match value {
            LitterDeviceType::T3 => Self::T3,
            LitterDeviceType::T4 => Self::T4,
            LitterDeviceType::T5 => Self::T5,
            LitterDeviceType::T6 => Self::T6,
            LitterDeviceType::T7 => Self::T7,
        }
    }
}

impl TryFrom<DeviceType> for LitterDeviceType {
    type Error = PetkitError;
    fn try_from(value: DeviceType) -> Result<Self, Self::Error> {
        match value {
            DeviceType::T3 => Ok(Self::T3),
            DeviceType::T4 => Ok(Self::T4),
            DeviceType::T5 => Ok(Self::T5),
            DeviceType::T6 => Ok(Self::T6),
            DeviceType::T7 => Ok(Self::T7),
            other => Err(PetkitError::InvalidArgument(alloc::format!(
                "device `{}` is not a litter box",
                other.as_str()
            ))),
        }
    }
}

/// Water fountain family device types: `W4`, `W5`, `Ctw2`, `Ctw3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FountainDeviceType {
    W4,
    W5,
    Ctw2,
    Ctw3,
}

impl FountainDeviceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::W4 => "w4",
            Self::W5 => "w5",
            Self::Ctw2 => "ctw2",
            Self::Ctw3 => "ctw3",
        }
    }
}

impl From<FountainDeviceType> for DeviceType {
    fn from(value: FountainDeviceType) -> Self {
        match value {
            FountainDeviceType::W4 => Self::W4,
            FountainDeviceType::W5 => Self::W5,
            FountainDeviceType::Ctw2 => Self::Ctw2,
            FountainDeviceType::Ctw3 => Self::Ctw3,
        }
    }
}

impl TryFrom<DeviceType> for FountainDeviceType {
    type Error = PetkitError;
    fn try_from(value: DeviceType) -> Result<Self, Self::Error> {
        match value {
            DeviceType::W4 => Ok(Self::W4),
            DeviceType::W5 => Ok(Self::W5),
            DeviceType::Ctw2 => Ok(Self::Ctw2),
            DeviceType::Ctw3 => Ok(Self::Ctw3),
            other => Err(PetkitError::InvalidArgument(alloc::format!(
                "device `{}` is not a water fountain",
                other.as_str()
            ))),
        }
    }
}

/// Air purifier family device types: `K2`, `K3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PurifierDeviceType {
    K2,
    K3,
}

impl PurifierDeviceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::K2 => "k2",
            Self::K3 => "k3",
        }
    }

    pub const fn uses_legacy_update_setting_endpoint(self) -> bool {
        matches!(self, Self::K3)
    }

    pub const fn uses_device_data_endpoint(self) -> bool {
        matches!(self, Self::K3)
    }
}

impl From<PurifierDeviceType> for DeviceType {
    fn from(value: PurifierDeviceType) -> Self {
        match value {
            PurifierDeviceType::K2 => Self::K2,
            PurifierDeviceType::K3 => Self::K3,
        }
    }
}

impl TryFrom<DeviceType> for PurifierDeviceType {
    type Error = PetkitError;
    fn try_from(value: DeviceType) -> Result<Self, Self::Error> {
        match value {
            DeviceType::K2 => Ok(Self::K2),
            DeviceType::K3 => Ok(Self::K3),
            other => Err(PetkitError::InvalidArgument(alloc::format!(
                "device `{}` is not an air purifier",
                other.as_str()
            ))),
        }
    }
}
