use alloc::string::ToString;

use core::marker::PhantomData;

use petkit_types::{
    DeviceId, LitterControl, LitterDeviceType, LitterSetting, ScheduleLimit, to_kv_string,
};

use crate::{HttpMethod, RequestSpec};

use super::{
    AuthCore, CLOUD_VIDEO_ENDPOINT, CONTROL_DEVICE_ENDPOINT, DEVICE_DETAIL_ENDPOINT,
    GET_DOWNLOAD_M3U8_ENDPOINT, GET_M3U8_ENDPOINT, LIVE_ENDPOINT, SCHEDULE_COMPLETE_ENDPOINT,
    SCHEDULE_ENDPOINT, SCHEDULE_HISTORY_ENDPOINT, SCHEDULE_REMOVE_ENDPOINT, SCHEDULE_SAVE_ENDPOINT,
    TEMP_OPEN_CAMERA_ENDPOINT, UPDATE_SETTING_ENDPOINT,
};

// ---------- litter scope ----------

const LITTER_DEODORANT_RESET_ENDPOINT: &str = "deodorantReset";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DynamicLitter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T3Litter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T4Litter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T5Litter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T6Litter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T7Litter;

mod model_seal {
    pub trait Sealed {}

    impl Sealed for super::T3Litter {}
    impl Sealed for super::T4Litter {}
    impl Sealed for super::T5Litter {}
    impl Sealed for super::T6Litter {}
    impl Sealed for super::T7Litter {}
}

pub trait LitterModel: model_seal::Sealed {
    const DEVICE_TYPE: LitterDeviceType;
}

pub trait LitterSupportsN50Deodorizer: LitterModel {}
pub trait LitterSupportsCamera: LitterModel {}

macro_rules! litter_model {
    ($marker:ty, $device_type:expr) => {
        impl LitterModel for $marker {
            const DEVICE_TYPE: LitterDeviceType = $device_type;
        }
    };
}

litter_model!(T3Litter, LitterDeviceType::T3);
litter_model!(T4Litter, LitterDeviceType::T4);
litter_model!(T5Litter, LitterDeviceType::T5);
litter_model!(T6Litter, LitterDeviceType::T6);
litter_model!(T7Litter, LitterDeviceType::T7);

impl LitterSupportsN50Deodorizer for T4Litter {}
impl LitterSupportsN50Deodorizer for T5Litter {}
impl LitterSupportsN50Deodorizer for T6Litter {}
impl LitterSupportsCamera for T5Litter {}
impl LitterSupportsCamera for T6Litter {}
impl LitterSupportsCamera for T7Litter {}

/// Litter request scope.
///
/// `LitterScope<DynamicLitter>` is returned by [`crate::AuthenticatedProtocol::litter`]
/// when the model is known only at runtime. Model-specific operations such as
/// camera streaming and N50 deodorizer reset require a typed scope from
/// [`crate::AuthenticatedProtocol::litter_typed`].
///
/// ```compile_fail
/// use petkit_protocol::{AuthenticatedProtocol, BaseUrl, T3Litter};
/// use petkit_types::{ClientContext, ClientProfile};
///
/// let context = ClientContext::new(ClientProfile::default(), "UTC", "0");
/// let auth = AuthenticatedProtocol::new(
///     context,
///     BaseUrl::Regional("https://api.petkt.com/latest/".into()),
///     "session-id",
/// );
///
/// // T3 does not expose the N50 deodorizer endpoint.
/// let _ = auth.litter_typed::<T3Litter>(42).reset_n50_deodorizer();
/// ```
#[derive(Clone, Debug)]
pub struct LitterScope<M = DynamicLitter> {
    pub(super) auth: AuthCore,
    pub(super) device_type: LitterDeviceType,
    pub(super) device_id: DeviceId,
    pub(super) _model: PhantomData<M>,
}

impl<M> LitterScope<M> {
    pub fn device_type(&self) -> LitterDeviceType {
        self.device_type
    }

    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub fn with_model<N>(&self) -> LitterScope<N> {
        LitterScope {
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

    pub fn update_setting(&self, setting: &LitterSetting) -> RequestSpec {
        self.request(UPDATE_SETTING_ENDPOINT)
            .push_form_field("id", self.device_id.to_string())
            .push_form_field("kv", to_kv_string(setting))
    }

    pub fn control_device(&self, command: &LitterControl) -> RequestSpec {
        self.request(CONTROL_DEVICE_ENDPOINT)
            .push_form_field("id", self.device_id.to_string())
            .push_form_field("kv", to_kv_string(command))
            .push_form_field("type", command.command_type())
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

impl<M> LitterScope<M>
where
    M: LitterSupportsN50Deodorizer,
{
    pub fn reset_n50_deodorizer(&self) -> RequestSpec {
        self.request(LITTER_DEODORANT_RESET_ENDPOINT)
            .push_form_field("deviceId", self.device_id.to_string())
    }
}

impl<M> LitterScope<M>
where
    M: LitterSupportsCamera,
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
