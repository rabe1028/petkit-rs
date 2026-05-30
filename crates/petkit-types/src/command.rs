use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceCommand {
    Power,
    ControlDevice,
    UpdateSetting,
    OpenCamera,
}

impl DeviceCommand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Power => "power_device",
            Self::ControlDevice => "control_device",
            Self::UpdateSetting => "update_setting",
            Self::OpenCamera => "open_camera",
        }
    }
}

impl fmt::Display for DeviceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeederCommand {
    CallPet,
    Calibration,
    ManualFeed,
    CancelManualFeed,
    FoodReplenished,
    ResetDesiccant,
    SuspendFeed,
    RestoreFeed,
    SaveRepeats,
}

impl FeederCommand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CallPet => "call_pet",
            Self::Calibration => "food_reset",
            Self::ManualFeed => "manual_feed",
            Self::CancelManualFeed => "cancelRealtimeFeed",
            Self::FoodReplenished => "food_replenished",
            Self::ResetDesiccant => "desiccant_reset",
            Self::SuspendFeed => "suspend_feed",
            Self::RestoreFeed => "restore_feed",
            Self::SaveRepeats => "save_repeats",
        }
    }
}

impl fmt::Display for FeederCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LitterCommand {
    ResetN50Deodorizer,
}

impl LitterCommand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResetN50Deodorizer => "reset_deodorizer",
        }
    }
}

impl fmt::Display for LitterCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PetCommand {
    UpdateSetting,
}

impl PetCommand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UpdateSetting => "pet_update_setting",
        }
    }
}

impl fmt::Display for PetCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceAction {
    Continue,
    End,
    Start,
    Stop,
    Mode,
    Power,
}

impl DeviceAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Continue => "continue_action",
            Self::End => "end_action",
            Self::Start => "start_action",
            Self::Stop => "stop_action",
            Self::Mode => "mode_action",
            Self::Power => "power_action",
        }
    }
}

impl fmt::Display for DeviceAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FountainAction {
    ModeNormal,
    ModeSmart,
    ModeStandard,
    ModeIntermittent,
    Pause,
    Continue,
    PowerOff,
    PowerOn,
    ResetFilter,
    DoNotDisturb,
    DoNotDisturbOff,
    LightLow,
    LightMedium,
    LightHigh,
    LightOn,
    LightOff,
}

impl FountainAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModeNormal => "Normal",
            Self::ModeSmart => "Smart",
            Self::ModeStandard => "Standard",
            Self::ModeIntermittent => "Intermittent",
            Self::Pause => "Pause",
            Self::Continue => "Continue",
            Self::PowerOff => "Power Off",
            Self::PowerOn => "Power On",
            Self::ResetFilter => "Reset Filter",
            Self::DoNotDisturb => "Do Not Disturb",
            Self::DoNotDisturbOff => "Do Not Disturb Off",
            Self::LightLow => "Light Low",
            Self::LightMedium => "Light Medium",
            Self::LightHigh => "Light High",
            Self::LightOn => "Light On",
            Self::LightOff => "Light Off",
        }
    }
}

impl fmt::Display for FountainAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LbCommand {
    Cleaning = 0,
    Dumping = 1,
    OdorRemoval = 2,
    Resetting = 3,
    Leveling = 4,
    Calibrating = 5,
    ResetDeodor = 6,
    Light = 7,
    ResetN50Deodor = 8,
    Maintenance = 9,
    ResetN60Deodor = 10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PurifierMode {
    Auto = 0,
    Silent = 1,
    Standard = 2,
    Strong = 3,
}

impl PurifierMode {
    pub const fn wire_value(self) -> i64 {
        self as i64
    }
}
