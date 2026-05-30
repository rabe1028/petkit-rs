use alloc::string::ToString;

use petkit_types::{to_kv_string, DeviceId, FountainDeviceType, FountainSetting};

use crate::{HttpMethod, RequestSpec};

use super::{AuthCore, DEVICE_DETAIL_ENDPOINT, UPDATE_SETTING_ENDPOINT};

// ---------- fountain scope ----------

#[derive(Clone, Debug)]
pub struct FountainScope {
    pub(super) auth: AuthCore,
    pub(super) device_type: FountainDeviceType,
    pub(super) device_id: DeviceId,
}

impl FountainScope {
    pub fn device_type(&self) -> FountainDeviceType {
        self.device_type
    }

    pub fn device_id(&self) -> DeviceId {
        self.device_id
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

    /// Most fountain control happens over BLE; this exists for the rare
    /// HTTP-driven setting case via `FountainSetting::Other`.
    pub fn update_setting(&self, setting: &FountainSetting) -> RequestSpec {
        self.request(UPDATE_SETTING_ENDPOINT)
            .push_form_field("id", self.device_id.to_string())
            .push_form_field("kv", to_kv_string(setting))
    }
}
