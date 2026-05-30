use alloc::string::ToString;

use petkit_types::{to_kv_string, PetId, PetSetting};

use crate::{HttpMethod, RequestSpec};

use super::AuthCore;

// ---------- pet scope ----------

const PET_UPDATE_SETTING_ENDPOINT: &str = "updatepetprops";

#[derive(Clone, Debug)]
pub struct PetScope {
    pub(super) auth: AuthCore,
    pub(super) pet_id: PetId,
}

impl PetScope {
    pub fn pet_id(&self) -> PetId {
        self.pet_id
    }

    pub fn update_setting(&self, setting: &PetSetting) -> RequestSpec {
        self.auth
            .device_request("pet", PET_UPDATE_SETTING_ENDPOINT, HttpMethod::Post)
            .push_form_field("petId", self.pet_id.to_string())
            .push_form_field("kv", to_kv_string(setting))
    }
}
