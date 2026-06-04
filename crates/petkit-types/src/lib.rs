#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod camera;
mod client;
mod cloud_ble;
mod command;
mod device;
mod discovery;
mod error;
mod media;
mod region;
mod response;
mod session;
mod setting;
mod value;

pub use camera::{AgoraRtmResponse, CameraRtmCommand, PtzDirection, PtzKind, json_string};
pub use client::{ClientContext, ClientProfile};
pub use cloud_ble::{
    CloudBleConnectRequest, CloudBleConnection, CloudBleControlRequest, CloudBleControlResponse,
    CloudBleDevice, CloudBleDevicesResponse, CloudBleMetadata, CloudBlePollRequest,
    CloudBlePollState, CloudBleRelayOptions,
};
pub use command::{
    DeviceAction, DeviceCommand, FeederCommand, FountainAction, LbCommand, LitterCommand,
    PetCommand, PurifierMode,
};
pub use device::{
    DeviceFamily, DeviceFamilyKind, DeviceType, FeederDeviceType, FountainDeviceType,
    LitterDeviceType, PurifierDeviceType,
};
pub use discovery::{DeviceCatalog, DeviceLookup, flatten_devices, resolve_device};
pub use error::{PetkitError, PetkitErrorCode};
pub use media::{
    CloudVideoResponse, GetDownloadM3u8Response, GetM3u8Response, M3u8Response, MediaEventType,
    MediaListResponse, MediaMetadata, MediaType, latest_image_metadata, latest_video_metadata,
};
pub use region::{
    CHINA_BASE_URL, DEFAULT_COUNTRY, DEFAULT_TIMEZONE, PASSPORT_BASE_URL, RegionServer,
    RegionServerGroup, RegionServersPayload, gateway_label, group_region_servers,
};
pub use response::{
    CalibrationResponse, CallPetResponse, CameraLiveFeed, CancelManualFeedResponse,
    CommandResponse, ControlDeviceResponse, DeviceDetailResponse, FamilyListResponse,
    FeederCalibrationResponse, FeederCallPetResponse, FeederCancelManualFeedResponse,
    FeederDeviceDetailResponse, FeederFoodReplenishedResponse, FeederManualFeedResponse,
    FeederOpenCameraResponse, FeederPlaySoundResponse, FeederRemoveDailyFeedResponse,
    FeederResetDesiccantResponse, FeederRestoreDailyFeedResponse, FeederRestoreFeedResponse,
    FeederSaveFeedResponse, FeederSaveRepeatsResponse, FeederScheduleCompleteResponse,
    FeederScheduleRemoveResponse, FeederScheduleSaveResponse, FeederSettingsReadResponse,
    FeederStartLiveResponse, FeederSuspendFeedResponse, FeederUpdateSettingResponse,
    FoodReplenishedResponse, FountainDeviceDetailResponse, FountainSettingsReadResponse,
    FountainUpdateSettingResponse, IotDeviceInfoResponse, IotDeviceInfoV1Response,
    IotDeviceInfoV2Response, LitterControlDeviceResponse, LitterDeviceDetailResponse,
    LitterOpenCameraResponse, LitterResetN50DeodorizerResponse, LitterScheduleCompleteResponse,
    LitterScheduleRemoveResponse, LitterScheduleSaveResponse, LitterSettingsReadResponse,
    LitterStartLiveResponse, LitterUpdateSettingResponse, LiveFeedResponse, LoginResponse,
    ManualFeedResponse, OpenCameraResponse, PetUpdateSettingResponse, PlaySoundResponse,
    PuraMaxControlDeviceResponse, PuraMaxResetDeodorizerResponse, PurifierControlDeviceResponse,
    PurifierDeviceDetailResponse, PurifierSettingsReadResponse, PurifierUpdateSettingResponse,
    RefreshSessionResponse, RegionServersResponse, RemoveDailyFeedResponse,
    RequestLoginCodeResponse, ResetDesiccantResponse, ResetN50DeodorizerResponse,
    RestoreDailyFeedResponse, RestoreFeedResponse, SaveFeedResponse, SaveRepeatsResponse,
    ScheduleCompleteResponse, ScheduleRemoveResponse, ScheduleSaveResponse, StartLiveResponse,
    SuspendFeedResponse, UpdateSettingResponse,
};
pub use session::{
    AccountGroup, AliyunMqttConnectionSummary, AliyunMqttOptions, AliyunSecureMode, DeviceSummary,
    IotConfigSet, IotDeviceInfo, PetSummary, Session,
};
pub use setting::{
    CustomSetting, CustomSettingValue, ExtraFormPayload, FeedDailyList, FeedIdentifier,
    FeederSetting, FountainSetting, LitterControl, LitterSetting, PetSetting, PurifierControl,
    PurifierSetting, RepeatSchedule, to_kv_string,
};
pub use value::{
    CalibrationAction, ControlCommandType, DeviceId, FeedEntryId, FeedTime, FeederSurplusGrams,
    LitterModeValue, LitterSandType, LitterStillTimeSeconds, LitterWorkMode, PetId, PetWeightGrams,
    PetkitDay, RepeatDays, ScheduleLimit, SettingInt, SettingString, SoundId, VolumeLevel,
};

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use alloc::string::String;

    use crate::{
        DeviceFamily, DeviceType, RegionServer, RegionServersPayload, gateway_label,
        group_region_servers,
    };

    #[test]
    fn grouping_region_servers_adds_china_gateway() {
        let payload = RegionServersPayload {
            list: vec![RegionServer {
                account_type: String::from("overseas"),
                gateway: String::from("https://api.eu-pet.com/latest/"),
                id: String::from("DE"),
                name: String::from("Germany"),
            }],
        };

        let groups = group_region_servers(&payload);
        assert_eq!(groups.len(), 2);
        assert!(
            groups
                .iter()
                .any(|group| group.gateway == crate::CHINA_BASE_URL)
        );
        assert_eq!(gateway_label("https://api.eu-pet.com/latest/"), "Europe");
    }

    #[test]
    fn device_type_helpers_capture_family_and_camera_support() {
        use crate::{FeederDeviceType, LitterDeviceType, PurifierDeviceType};
        assert_eq!(DeviceType::T6.family(), DeviceFamily::LitterBox);
        assert!(DeviceType::T6.supports_camera());
        assert!(FeederDeviceType::D4s.is_dual_hopper());
        assert!(FeederDeviceType::FeederMini.uses_legacy_manual_feed_endpoint());
        assert!(LitterDeviceType::T6.supports_camera());
        assert!(PurifierDeviceType::K3.uses_device_data_endpoint());
    }
}
