use alloc::string::ToString;

use petkit_types::{CloudBleConnectRequest, CloudBleControlRequest};

use crate::{HttpMethod, RequestSpec};

use super::AuthCore;

const BLE_SUPPORTED_DEVICES_ENDPOINT: &str = "ble/ownSupportBleDevices";
const BLE_CONNECT_ENDPOINT: &str = "ble/connect";
const BLE_POLL_ENDPOINT: &str = "ble/poll";
const BLE_CONTROL_DEVICE_ENDPOINT: &str = "ble/controlDevice";

#[derive(Clone, Debug)]
pub struct CloudBleScope {
    pub(super) auth: AuthCore,
}

impl CloudBleScope {
    fn request(&self, endpoint: &str) -> RequestSpec {
        self.auth.request(HttpMethod::Post, endpoint)
    }

    pub fn supported_devices(&self) -> RequestSpec {
        self.request(BLE_SUPPORTED_DEVICES_ENDPOINT)
    }

    pub fn supported_devices_for_group(&self, group_id: impl ToString) -> RequestSpec {
        self.supported_devices()
            .push_form_field("groupId", group_id.to_string())
    }

    pub fn connect(&self, request: &CloudBleConnectRequest) -> RequestSpec {
        append_connection_fields(self.request(BLE_CONNECT_ENDPOINT), request)
    }

    pub fn poll(&self, request: &CloudBleConnectRequest) -> RequestSpec {
        append_connection_fields(self.request(BLE_POLL_ENDPOINT), request)
    }

    pub fn control_device(&self, request: &CloudBleControlRequest) -> RequestSpec {
        let mut spec = self
            .request(BLE_CONTROL_DEVICE_ENDPOINT)
            .push_form_field("bleId", request.ble_id_or_device_id())
            .push_form_field("deviceId", request.device_id.clone())
            .push_form_field("type", request.device_type.clone())
            .push_form_field("mac", request.mac.clone())
            .push_form_field("cmd", request.cmd.clone())
            .push_form_field("data", request.data.clone());
        if let Some(group_id) = &request.group_id {
            spec = spec.push_form_field("groupId", group_id.clone());
        }
        spec
    }
}

fn append_connection_fields(spec: RequestSpec, request: &CloudBleConnectRequest) -> RequestSpec {
    let mut spec = spec
        .push_form_field("bleId", request.ble_id.clone())
        .push_form_field("type", request.device_type.clone())
        .push_form_field("mac", request.mac.clone());
    if let Some(group_id) = &request.group_id {
        spec = spec.push_form_field("groupId", group_id.clone());
    }
    spec
}
