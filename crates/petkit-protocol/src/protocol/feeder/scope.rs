use alloc::string::{String, ToString};

use core::marker::PhantomData;

use petkit_types::{
    CalibrationAction, DeviceId, FeedDailyList, FeedEntryId, FeedIdentifier, FeederDeviceType,
    FeederSetting, PetkitDay, PetkitError, RepeatSchedule, ScheduleLimit, SoundId,
};

use crate::{HttpMethod, RequestSpec};

use super::super::{
    AuthCore, CLOUD_VIDEO_ENDPOINT, DEVICE_DETAIL_ENDPOINT, GET_DOWNLOAD_M3U8_ENDPOINT,
    GET_M3U8_ENDPOINT, LIVE_ENDPOINT, SCHEDULE_COMPLETE_ENDPOINT, SCHEDULE_ENDPOINT,
    SCHEDULE_HISTORY_ENDPOINT, SCHEDULE_REMOVE_ENDPOINT, SCHEDULE_SAVE_ENDPOINT,
    TEMP_OPEN_CAMERA_ENDPOINT, UPDATE_SETTING_ENDPOINT, UPDATE_SETTING_OLD_ENDPOINT,
};
use super::amount::manual_feed_amount;
use super::{
    encode_feeder_setting, DynamicFeeder, FeederModel, FeederSupportsCalibration,
    FeederSupportsCallPet, FeederSupportsCamera, FeederSupportsFoodReplenished,
    FeederSupportsSound, ManualFeedAmount, FEEDER_CALL_PET_ENDPOINT,
    FEEDER_CANCEL_REALTIME_FEED_ENDPOINT, FEEDER_DESICCANT_RESET_NEW_ENDPOINT,
    FEEDER_DESICCANT_RESET_OLD_ENDPOINT, FEEDER_FRESH_ELEMENT_CALIBRATION_ENDPOINT,
    FEEDER_FRESH_ELEMENT_CANCEL_FEED_ENDPOINT, FEEDER_MANUAL_FEED_NEW_ENDPOINT,
    FEEDER_MANUAL_FEED_OLD_ENDPOINT, FEEDER_PLAY_SOUND_ENDPOINT, FEEDER_REMOVE_DAILY_FEED_ENDPOINT,
    FEEDER_REPLENISHED_FOOD_ENDPOINT, FEEDER_RESTORE_DAILY_FEED_ENDPOINT,
    FEEDER_RESTORE_FEED_NEW_ENDPOINT, FEEDER_RESTORE_FEED_OLD_ENDPOINT, FEEDER_SAVE_FEED_ENDPOINT,
    FEEDER_SAVE_REPEATS_NEW_ENDPOINT, FEEDER_SAVE_REPEATS_OLD_ENDPOINT,
    FEEDER_SUSPEND_FEED_NEW_ENDPOINT, FEEDER_SUSPEND_FEED_OLD_ENDPOINT,
};

/// Feeder request scope.
///
/// `FeederScope<DynamicFeeder>` is returned by [`crate::AuthenticatedProtocol::feeder`]
/// when the model is known only at runtime. Model-specific operations such as
/// manual feeding require a typed scope from
/// [`crate::AuthenticatedProtocol::feeder_typed`].
///
/// ```compile_fail
/// use petkit_protocol::{
///     AuthenticatedProtocol, BaseUrl, D4sFeeder, SingleManualFeedAmount,
/// };
/// use petkit_types::{ClientContext, ClientProfile, DeviceId, PetkitDay};
///
/// let context = ClientContext::new(ClientProfile::default(), "UTC", "0");
/// let auth = AuthenticatedProtocol::new(
///     context,
///     BaseUrl::Regional("https://api.petkt.com/latest/".into()),
///     "session-id",
/// );
///
/// // D4S is a dual-hopper feeder, so a single-hopper payload is not accepted.
/// let _ = auth
///     .feeder_typed::<D4sFeeder>(DeviceId::new(42).unwrap())
///     .manual_feed(
///         SingleManualFeedAmount::<D4sFeeder>::new(5).unwrap(),
///         &PetkitDay::new("20260527").unwrap(),
///     );
/// ```
#[derive(Clone, Debug)]
pub struct FeederScope<M = DynamicFeeder> {
    pub(in crate::protocol) auth: AuthCore,
    pub(in crate::protocol) device_type: FeederDeviceType,
    pub(in crate::protocol) device_id: DeviceId,
    pub(in crate::protocol) _model: PhantomData<M>,
}

impl<M> FeederScope<M> {
    pub fn device_type(&self) -> FeederDeviceType {
        self.device_type
    }

    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub fn with_model<N>(&self) -> FeederScope<N> {
        FeederScope {
            auth: self.auth.clone(),
            device_type: self.device_type,
            device_id: self.device_id,
            _model: PhantomData,
        }
    }

    fn request(&self, endpoint: &str) -> RequestSpec {
        self.auth
            .device_request(self.device_type.as_str(), endpoint, HttpMethod::Post)
    }

    /// Read the broad device detail payload, including `settings` and `state`.
    pub fn device_detail(&self) -> RequestSpec {
        self.request(DEVICE_DETAIL_ENDPOINT)
            .push_form_field("id", self.device_id.to_string())
    }

    pub fn update_setting(&self, setting: &FeederSetting) -> RequestSpec {
        let endpoint = if self.device_type.uses_legacy_update_setting_endpoint() {
            UPDATE_SETTING_OLD_ENDPOINT
        } else {
            UPDATE_SETTING_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("id", self.device_id.to_string())
            .push_form_field("kv", encode_feeder_setting(self.device_type, setting))
    }

    pub fn cancel_manual_feed(
        &self,
        day: &PetkitDay,
        manual_feed_id: Option<FeedEntryId>,
    ) -> Result<RequestSpec, PetkitError> {
        let endpoint = if matches!(self.device_type, FeederDeviceType::Feeder) {
            FEEDER_FRESH_ELEMENT_CANCEL_FEED_ENDPOINT
        } else {
            FEEDER_CANCEL_REALTIME_FEED_ENDPOINT
        };
        let mut request = self
            .request(endpoint)
            .push_form_field("day", day.to_string())
            .push_form_field("deviceId", self.device_id.to_string());

        if matches!(
            self.device_type,
            FeederDeviceType::D4h | FeederDeviceType::D4s | FeederDeviceType::D4sh
        ) {
            let id = manual_feed_id.ok_or_else(|| {
                PetkitError::InvalidArgument(String::from(
                    "manual_feed_id is required for D4H/D4S/D4SH feeders",
                ))
            })?;
            request = request.push_form_field("id", id.to_string());
        }
        Ok(request)
    }

    pub fn remove_daily_feed(
        &self,
        day: &PetkitDay,
        feed_identifier: &FeedIdentifier,
    ) -> RequestSpec {
        self.request(FEEDER_REMOVE_DAILY_FEED_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
            .push_form_field("day", day.to_string())
            .extend_form(&feed_identifier.to_form())
    }

    pub fn restore_daily_feed(
        &self,
        day: &PetkitDay,
        feed_identifier: &FeedIdentifier,
    ) -> RequestSpec {
        self.request(FEEDER_RESTORE_DAILY_FEED_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
            .push_form_field("day", day.to_string())
            .extend_form(&feed_identifier.to_form())
    }

    pub fn save_feed(&self, feed_daily_list: &FeedDailyList) -> RequestSpec {
        self.request(FEEDER_SAVE_FEED_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
            .push_form_field("feedDailyList", feed_daily_list.raw_json())
    }

    pub fn suspend_feed(&self) -> RequestSpec {
        let endpoint = if self.device_type.uses_legacy_suspend_feed_endpoint() {
            FEEDER_SUSPEND_FEED_OLD_ENDPOINT
        } else {
            FEEDER_SUSPEND_FEED_NEW_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn restore_feed(&self) -> RequestSpec {
        let endpoint = if self.device_type.uses_legacy_suspend_feed_endpoint() {
            FEEDER_RESTORE_FEED_OLD_ENDPOINT
        } else {
            FEEDER_RESTORE_FEED_NEW_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn save_repeats(&self, schedule: &RepeatSchedule) -> RequestSpec {
        let endpoint = if self.device_type.uses_legacy_schedule_endpoint() {
            FEEDER_SAVE_REPEATS_OLD_ENDPOINT
        } else {
            FEEDER_SAVE_REPEATS_NEW_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("deviceId", self.device_id.to_string())
            .extend_form(&schedule.to_form())
    }

    pub fn reset_desiccant(&self) -> RequestSpec {
        let endpoint = if self.device_type.uses_legacy_desiccant_endpoint() {
            FEEDER_DESICCANT_RESET_OLD_ENDPOINT
        } else {
            FEEDER_DESICCANT_RESET_NEW_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn schedule_list(&self, limit: ScheduleLimit) -> RequestSpec {
        self.request(SCHEDULE_ENDPOINT)
            .push_query("limit", limit.to_string())
    }

    pub fn schedule_save(&self) -> RequestSpec {
        self.request(SCHEDULE_SAVE_ENDPOINT)
    }

    pub fn schedule_remove(&self) -> RequestSpec {
        self.request(SCHEDULE_REMOVE_ENDPOINT)
    }

    pub fn schedule_complete(&self) -> RequestSpec {
        self.request(SCHEDULE_COMPLETE_ENDPOINT)
    }

    pub fn schedule_history(&self) -> RequestSpec {
        self.request(SCHEDULE_HISTORY_ENDPOINT)
    }
}

impl<M> FeederScope<M>
where
    M: FeederModel,
{
    pub fn manual_feed<A>(&self, amount: A, day: &PetkitDay) -> RequestSpec
    where
        A: ManualFeedAmount<M>,
    {
        let endpoint = if M::DEVICE_TYPE.uses_legacy_manual_feed_endpoint() {
            FEEDER_MANUAL_FEED_OLD_ENDPOINT
        } else {
            FEEDER_MANUAL_FEED_NEW_ENDPOINT
        };
        let request = self
            .request(endpoint)
            .push_form_field("day", day.to_string())
            .push_form_field("deviceId", self.device_id.to_string())
            .push_form_field("name", "")
            .push_form_field("time", "-1");

        match manual_feed_amount::Sealed::into_parts(amount) {
            manual_feed_amount::Parts::Single { amount } => {
                request.push_form_field("amount", amount.to_string())
            }
            manual_feed_amount::Parts::Dual { amount1, amount2 } => request
                .push_form_field("amount1", amount1.to_string())
                .push_form_field("amount2", amount2.to_string()),
        }
    }
}

impl<M> FeederScope<M>
where
    M: FeederSupportsFoodReplenished,
{
    pub fn food_replenished(&self) -> RequestSpec {
        self.request(FEEDER_REPLENISHED_FOOD_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
            .push_form_field("noRemind", "3")
    }
}

impl<M> FeederScope<M>
where
    M: FeederSupportsCalibration,
{
    pub fn calibration(&self, action: CalibrationAction) -> RequestSpec {
        self.request(FEEDER_FRESH_ELEMENT_CALIBRATION_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
            .push_form_field("action", action.get().to_string())
    }
}

impl<M> FeederScope<M>
where
    M: FeederSupportsCallPet,
{
    pub fn call_pet(&self) -> RequestSpec {
        self.request(FEEDER_CALL_PET_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
    }
}

impl<M> FeederScope<M>
where
    M: FeederSupportsSound,
{
    pub fn play_sound(&self, sound_id: SoundId) -> RequestSpec {
        self.request(FEEDER_PLAY_SOUND_ENDPOINT)
            .push_form_field("soundId", sound_id.to_string())
            .push_form_field("deviceId", self.device_id.to_string())
    }
}

impl<M> FeederScope<M>
where
    M: FeederSupportsCamera,
{
    pub fn open_camera(&self) -> RequestSpec {
        self.request(TEMP_OPEN_CAMERA_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn start_live(&self) -> RequestSpec {
        self.request(LIVE_ENDPOINT)
            .push_form_field("definition", "2")
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn cloud_video(&self) -> RequestSpec {
        self.request(CLOUD_VIDEO_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn get_m3u8(&self) -> RequestSpec {
        self.request(GET_M3U8_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
    }

    pub fn get_download_m3u8(&self) -> RequestSpec {
        self.request(GET_DOWNLOAD_M3U8_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
    }
}
