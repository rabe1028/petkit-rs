//! Strongly typed request-building API.
//!
//! Construction follows a 3-layer scope chain:
//!
//! - [`PublicProtocol`] — unauthenticated endpoints (region servers, login).
//! - [`AuthenticatedProtocol`] — endpoints requiring a session, including
//!   account-wide listings and the family-specific scope constructors.
//! - per-family scopes (`FeederScope`, `LitterScope`, `FountainScope`,
//!   `PurifierScope`, [`PetScope`]) — only expose operations valid for
//!   that family at compile time.
//!
//! Each scope owns a snapshot of the connection state, so callers don't
//! pass `(context, base_url, session_id, …)` to every method.

use alloc::format;
use alloc::string::String;

use core::marker::PhantomData;

use petkit_types::{
    ClientContext, DeviceId, FeederDeviceType, FountainDeviceType, LitterDeviceType, PetId,
    PurifierDeviceType,
};

use crate::{BaseUrl, HttpMethod, RequestSpec};

mod feeder;
mod fountain;
mod litter;
mod pet;
mod purifier;

pub use self::feeder::*;
pub use self::fountain::*;
pub use self::litter::*;
pub use self::pet::*;
pub use self::purifier::*;

// ---------- internal shared state ----------

#[derive(Clone, Debug)]
struct AuthCore {
    context: ClientContext,
    base_url: BaseUrl,
    session_id: String,
}

impl AuthCore {
    fn request(&self, method: HttpMethod, path: impl Into<String>) -> RequestSpec {
        RequestSpec::new(method, &self.base_url, path)
            .with_default_headers(&self.context)
            .with_session_headers(&self.session_id)
    }

    fn device_request(&self, device_str: &str, endpoint: &str, method: HttpMethod) -> RequestSpec {
        self.request(method, format!("{device_str}/{endpoint}"))
    }
}

// ---------- public (pre-auth) protocol ----------

const REGION_SERVERS_PATH: &str = "v1/regionservers";
const LOGIN_PATH: &str = "user/login";
const LOGIN_CODE_PATH: &str = "user/sendcodeforquicklogin";

/// Endpoints that do not require an authenticated session.
#[derive(Clone, Debug)]
pub struct PublicProtocol {
    context: ClientContext,
}

impl PublicProtocol {
    pub fn new(context: ClientContext) -> Self {
        Self { context }
    }

    pub fn region_servers(&self) -> RequestSpec {
        RequestSpec::new(HttpMethod::Get, &BaseUrl::Passport, REGION_SERVERS_PATH)
            .with_default_headers(&self.context)
    }

    pub fn request_login_code(&self, username: &str) -> RequestSpec {
        RequestSpec::new(HttpMethod::Get, &BaseUrl::Passport, LOGIN_CODE_PATH)
            .with_default_headers(&self.context)
            .push_query("username", username)
    }

    pub fn login_with_password(
        &self,
        username: &str,
        password_md5: &str,
        region: &str,
    ) -> RequestSpec {
        self.login_request(username, region)
            .push_form_field("password", password_md5)
    }

    pub fn login_with_code(&self, username: &str, valid_code: &str, region: &str) -> RequestSpec {
        self.login_request(username, region)
            .push_form_field("validCode", valid_code)
    }

    fn login_request(&self, username: &str, region: &str) -> RequestSpec {
        RequestSpec::new(HttpMethod::Post, &BaseUrl::Passport, LOGIN_PATH)
            .with_default_headers(&self.context)
            .push_form_field("oldVersion", self.context.profile.api_version.clone())
            .push_form_field("client", self.context.render_client_descriptor())
            .push_form_field("encrypt", "1")
            .push_form_field("region", region)
            .push_form_field("username", username)
    }
}

// ---------- authenticated protocol ----------

const REFRESH_SESSION_PATH: &str = "user/refreshsession";
const FAMILY_LIST_PATH: &str = "group/family/list";
const IOT_DEVICE_INFO_V1_PATH: &str = "user/iotDeviceInfo";
const IOT_DEVICE_INFO_V2_PATH: &str = "user/iotDeviceInfo_v2";

/// Endpoints that require an authenticated session. Use the family-specific
/// constructors ([`feeder`](Self::feeder), [`litter`](Self::litter), etc.)
/// to obtain scopes that only expose operations valid for that family.
#[derive(Clone, Debug)]
pub struct AuthenticatedProtocol {
    auth: AuthCore,
}

impl AuthenticatedProtocol {
    pub fn new(context: ClientContext, base_url: BaseUrl, session_id: impl Into<String>) -> Self {
        Self {
            auth: AuthCore {
                context,
                base_url,
                session_id: session_id.into(),
            },
        }
    }

    pub fn refresh_session(&self) -> RequestSpec {
        // Refresh hits the Passport host with default headers.
        RequestSpec::new(HttpMethod::Post, &BaseUrl::Passport, REFRESH_SESSION_PATH)
            .with_default_headers(&self.auth.context)
            .with_session_headers(&self.auth.session_id)
            .push_form_field("oldVersion", self.auth.context.profile.api_version.clone())
    }

    /// Update the session id used by every subsequent authenticated request.
    pub fn set_session(&mut self, session_id: impl Into<String>) {
        self.auth.session_id = session_id.into();
    }

    /// Currently-stored session id.
    pub fn session_id(&self) -> &str {
        &self.auth.session_id
    }

    pub fn family_list(&self) -> RequestSpec {
        self.auth.request(HttpMethod::Post, FAMILY_LIST_PATH)
    }

    pub fn iot_device_info_v1(&self) -> RequestSpec {
        self.auth.request(HttpMethod::Get, IOT_DEVICE_INFO_V1_PATH)
    }

    pub fn iot_device_info_v2(&self) -> RequestSpec {
        self.auth.request(HttpMethod::Get, IOT_DEVICE_INFO_V2_PATH)
    }

    pub fn feeder(&self, device_type: FeederDeviceType, device_id: DeviceId) -> FeederScope {
        FeederScope {
            auth: self.auth.clone(),
            device_type,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn feeder_typed<M>(&self, device_id: DeviceId) -> FeederScope<M>
    where
        M: FeederModel,
    {
        FeederScope {
            auth: self.auth.clone(),
            device_type: M::DEVICE_TYPE,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn litter(&self, device_type: LitterDeviceType, device_id: DeviceId) -> LitterScope {
        LitterScope {
            auth: self.auth.clone(),
            device_type,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn litter_typed<M>(&self, device_id: DeviceId) -> LitterScope<M>
    where
        M: LitterModel,
    {
        LitterScope {
            auth: self.auth.clone(),
            device_type: M::DEVICE_TYPE,
            device_id,
            _model: PhantomData,
        }
    }

    pub fn fountain(&self, device_type: FountainDeviceType, device_id: DeviceId) -> FountainScope {
        FountainScope {
            auth: self.auth.clone(),
            device_type,
            device_id,
        }
    }

    pub fn purifier(&self, device_type: PurifierDeviceType, device_id: DeviceId) -> PurifierScope {
        PurifierScope {
            auth: self.auth.clone(),
            device_type,
            device_id,
        }
    }

    pub fn pet(&self, pet_id: PetId) -> PetScope {
        PetScope {
            auth: self.auth.clone(),
            pet_id,
        }
    }
}

// ---------- shared device endpoints ----------

const CONTROL_DEVICE_ENDPOINT: &str = "controlDevice";
const DEVICE_DATA_ENDPOINT: &str = "deviceData";
const DEVICE_DETAIL_ENDPOINT: &str = "device_detail";
const UPDATE_SETTING_ENDPOINT: &str = "updateSettings";
const UPDATE_SETTING_OLD_ENDPOINT: &str = "update";
const TEMP_OPEN_CAMERA_ENDPOINT: &str = "temporary/open/camera";
const LIVE_ENDPOINT: &str = "start/live";
const CLOUD_VIDEO_ENDPOINT: &str = "cloud/video";
const GET_M3U8_ENDPOINT: &str = "getM3u8";
const GET_DOWNLOAD_M3U8_ENDPOINT: &str = "getDownloadM3u8";
const SCHEDULE_ENDPOINT: &str = "schedule/schedules";
const SCHEDULE_SAVE_ENDPOINT: &str = "schedule/save";
const SCHEDULE_REMOVE_ENDPOINT: &str = "schedule/remove";
const SCHEDULE_COMPLETE_ENDPOINT: &str = "schedule/complete";
const SCHEDULE_HISTORY_ENDPOINT: &str = "schedule/userHistorySchedules";
