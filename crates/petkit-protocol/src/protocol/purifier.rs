use alloc::string::ToString;

use petkit_types::{to_kv_string, DeviceId, PurifierControl, PurifierDeviceType, PurifierSetting};

use crate::{HttpMethod, RequestSpec};

use super::{
    AuthCore, CONTROL_DEVICE_ENDPOINT, DEVICE_DATA_ENDPOINT, DEVICE_DETAIL_ENDPOINT,
    UPDATE_SETTING_ENDPOINT, UPDATE_SETTING_OLD_ENDPOINT,
};

// ---------- purifier scope ----------

#[derive(Clone, Debug)]
pub struct PurifierScope {
    pub(super) auth: AuthCore,
    pub(super) device_type: PurifierDeviceType,
    pub(super) device_id: DeviceId,
}

impl PurifierScope {
    pub fn device_type(&self) -> PurifierDeviceType {
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
    ///
    /// PETKIT routes K3 purifiers through `deviceData` while K2 uses the
    /// more common `device_detail` endpoint.
    pub fn device_detail(&self) -> RequestSpec {
        let endpoint = if self.device_type.uses_device_data_endpoint() {
            DEVICE_DATA_ENDPOINT
        } else {
            DEVICE_DETAIL_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("id", self.device_id.to_string())
    }

    pub fn update_setting(&self, setting: &PurifierSetting) -> RequestSpec {
        let endpoint = if self.device_type.uses_legacy_update_setting_endpoint() {
            UPDATE_SETTING_OLD_ENDPOINT
        } else {
            UPDATE_SETTING_ENDPOINT
        };
        self.request(endpoint)
            .push_form_field("id", self.device_id.to_string())
            .push_form_field("kv", to_kv_string(setting))
    }

    pub fn control_device(&self, command: &PurifierControl) -> RequestSpec {
        self.request(CONTROL_DEVICE_ENDPOINT)
            .push_form_field("id", self.device_id.to_string())
            .push_form_field("kv", to_kv_string(command))
            .push_form_field("type", command.command_type())
    }
}
