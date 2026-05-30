use alloc::format;
use alloc::string::String;

use petkit_types::{to_kv_string, FeederDeviceType, FeederSetting};

mod amount;
mod model;
mod scope;

pub use self::amount::*;
pub use self::model::*;
pub use self::scope::*;

/// `Feeder` / `FeederMini` use a nested `settings.<key>` wire format for
/// the shared settings; all other feeder models use the bare key. See
/// `petkit-types/docs/settings-research.md` for the per-key matrix.
fn encode_feeder_setting(device_type: FeederDeviceType, setting: &FeederSetting) -> String {
    let bool_int = |v: bool| if v { 1 } else { 0 };
    let legacy = matches!(
        device_type,
        FeederDeviceType::Feeder | FeederDeviceType::FeederMini
    );
    let is_mini = matches!(device_type, FeederDeviceType::FeederMini);

    match setting {
        FeederSetting::LightMode(v) if legacy => {
            format!(r#"{{"settings.lightMode":{}}}"#, bool_int(*v))
        }
        FeederSetting::ManualLock(v) if legacy => {
            format!(r#"{{"settings.manualLock":{}}}"#, bool_int(*v))
        }
        FeederSetting::FeedNotify(v) if is_mini => {
            format!(r#"{{"settings.feedNotify":{}}}"#, bool_int(*v))
        }
        // Other variants either don't apply to legacy feeders, or `Other`
        // is already in caller-controlled form — emit verbatim.
        _ => to_kv_string(setting),
    }
}

// ---------- feeder scope ----------

const FEEDER_REMOVE_DAILY_FEED_ENDPOINT: &str = "removeDailyFeed";
const FEEDER_RESTORE_DAILY_FEED_ENDPOINT: &str = "restoreDailyFeed";
const FEEDER_MANUAL_FEED_OLD_ENDPOINT: &str = "save_dailyfeed";
const FEEDER_MANUAL_FEED_NEW_ENDPOINT: &str = "saveDailyFeed";
const FEEDER_CANCEL_REALTIME_FEED_ENDPOINT: &str = "cancelRealtimeFeed";
const FEEDER_FRESH_ELEMENT_CANCEL_FEED_ENDPOINT: &str = "cancel_realtime_feed";
const FEEDER_REPLENISHED_FOOD_ENDPOINT: &str = "added";
const FEEDER_FRESH_ELEMENT_CALIBRATION_ENDPOINT: &str = "food_reset";
const FEEDER_DESICCANT_RESET_OLD_ENDPOINT: &str = "desiccant_reset";
const FEEDER_DESICCANT_RESET_NEW_ENDPOINT: &str = "desiccantReset";
const FEEDER_SAVE_FEED_ENDPOINT: &str = "saveFeed";
const FEEDER_SUSPEND_FEED_OLD_ENDPOINT: &str = "suspend_feed";
const FEEDER_SUSPEND_FEED_NEW_ENDPOINT: &str = "suspendFeed";
const FEEDER_RESTORE_FEED_OLD_ENDPOINT: &str = "restore_feed";
const FEEDER_RESTORE_FEED_NEW_ENDPOINT: &str = "restoreFeed";
const FEEDER_SAVE_REPEATS_OLD_ENDPOINT: &str = "save_repeats";
const FEEDER_SAVE_REPEATS_NEW_ENDPOINT: &str = "saveRepeats";
const FEEDER_PLAY_SOUND_ENDPOINT: &str = "playSound";
const FEEDER_CALL_PET_ENDPOINT: &str = "callPet";
