#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod ble;
mod protocol;
mod request;
mod response;

pub use ble::{
    BLE_END_FRAME, BLE_START_FRAME, BleEncodedCommand, BleFrameCommand, BleGattWriteError,
    BleGattWriter, FountainBleClient, FountainBleSettings, build_ble_frame,
    build_fountain_ble_command, build_fountain_ble_frame_command,
    build_fountain_ble_frame_command_with_settings, encode_ble_data, write_fountain_ble_frame,
    write_fountain_ble_frame_with_settings,
};
pub use protocol::{
    AuthenticatedProtocol, CAMERA_RTM_FALLBACK_BASE_URL, CAMERA_RTM_PRIMARY_BASE_URL,
    CloudBleScope, D3Feeder, D4Feeder, D4hFeeder, D4sFeeder, D4shFeeder, DualHopperFeederModel,
    DualManualFeedAmount, DynamicFeeder, DynamicLitter, FeederMiniFeeder, FeederModel, FeederScope,
    FeederSupportsCalibration, FeederSupportsCallPet, FeederSupportsCamera,
    FeederSupportsFoodReplenished, FeederSupportsSound, FountainScope, FreshElementFeeder,
    LitterModel, LitterScope, LitterSupportsCamera, LitterSupportsN50Deodorizer, ManualFeedAmount,
    PetScope, PublicProtocol, PurifierScope, SingleHopperFeederModel, SingleManualFeedAmount,
    T3Litter, T4Litter, T5Litter, T6Litter, T7Litter, camera_rtm_peer_message,
    camera_rtm_peer_message_for_base,
};
pub use request::{
    BaseUrl, FormField, Header, HttpMethod, QueryField, RequestBody, RequestSpec, ResponseParts,
    session_headers,
};
pub use response::{parse_api_response, parse_json_response, parse_text_response};

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use alloc::string::String;

    use petkit_types::{
        CameraLiveFeed, CameraRtmCommand, ClientContext, ClientProfile, CloudBleConnectRequest,
        CloudBleControlRequest, DeviceDetailResponse, DeviceId, FeederDeviceType, PetkitDay,
        PetkitError, PetkitErrorCode, PtzDirection, PtzKind, PurifierDeviceType,
    };

    use super::{
        AuthenticatedProtocol, BaseUrl, BleGattWriter, D4sFeeder, D4shFeeder, DualManualFeedAmount,
        FeederMiniFeeder, FountainBleClient, FountainBleSettings, HttpMethod, PublicProtocol,
        ResponseParts, SingleManualFeedAmount, T6Litter, build_ble_frame,
        build_fountain_ble_command, camera_rtm_peer_message, parse_api_response,
        parse_text_response, write_fountain_ble_frame,
    };

    fn context() -> ClientContext {
        ClientContext::new(ClientProfile::default(), "Europe/Berlin", "2.0")
    }

    fn authenticated() -> AuthenticatedProtocol {
        AuthenticatedProtocol::new(
            context(),
            BaseUrl::Regional("https://api.petkt.com/latest/".into()),
            "session-id",
        )
    }

    fn device_id(value: u64) -> DeviceId {
        DeviceId::new(value).expect("test device id should be non-zero")
    }

    fn day() -> PetkitDay {
        PetkitDay::new("20260527").expect("test day should be valid")
    }

    #[test]
    fn region_server_request_uses_passport_base() {
        let request = PublicProtocol::new(context()).region_servers();
        assert_eq!(request.url(), "https://passport.petkt.com/v1/regionservers");
        assert!(request.form_fields.is_empty());
    }

    #[test]
    fn login_request_includes_python_like_client_descriptor() {
        let request = PublicProtocol::new(context()).login_with_password(
            "user@example.com",
            "deadbeef",
            "DE",
        );
        let client = request
            .form_fields
            .iter()
            .find(|field| field.name == "client")
            .expect("client field must exist");
        assert!(client.value.contains("'locale': 'en-US'"));
        assert!(client.value.contains("'timezoneId': 'Europe/Berlin'"));
    }

    #[test]
    fn login_request_can_use_regional_base() {
        let request = PublicProtocol::with_login_base_url(
            context(),
            BaseUrl::Regional("https://api.petktasia.com/latest/".to_owned()),
        )
        .login_with_password("user@example.com", "deadbeef", "JP");
        assert_eq!(request.url(), "https://api.petktasia.com/latest/user/login");
    }

    #[test]
    fn manual_feed_uses_legacy_endpoint_for_feeder_mini() {
        let request = authenticated()
            .feeder_typed::<FeederMiniFeeder>(device_id(42))
            .manual_feed(
                SingleManualFeedAmount::<FeederMiniFeeder>::new(10)
                    .expect("amount should be valid"),
                &day(),
            );

        assert_eq!(request.path, "feedermini/save_dailyfeed");
    }

    #[test]
    fn manual_feed_allows_one_sided_dual_hopper() {
        let request = authenticated()
            .feeder_typed::<D4sFeeder>(device_id(42))
            .manual_feed(
                DualManualFeedAmount::<D4sFeeder>::new(5, 0).expect("amount should be valid"),
                &day(),
            );

        let amount2 = request
            .form_fields
            .iter()
            .find(|f| f.name == "amount2")
            .expect("amount2 field must exist");
        assert_eq!(amount2.value, "0");
    }

    #[test]
    fn manual_feed_rejects_double_zero_dual_hopper() {
        let error = DualManualFeedAmount::<D4sFeeder>::new(0, 0)
            .expect_err("zero/zero dual feed should be rejected");

        assert!(matches!(error, PetkitError::InvalidArgument(_)));
    }

    #[test]
    fn ble_frame_matches_python_layout() {
        let frame = build_ble_frame(&[220, 1, 3, 0, 1, 0, 2], 5);
        assert_eq!(frame, vec![250, 252, 253, 220, 1, 5, 3, 0, 1, 0, 2, 251]);

        let encoded = build_fountain_ble_command(petkit_types::FountainAction::Pause, 5)
            .expect("pause command must exist");
        assert_eq!(encoded.cmd, 220);
        assert!(!encoded.data.is_empty());
    }

    #[test]
    fn ble_gatt_writer_receives_raw_frame() {
        #[derive(Default)]
        struct Writer {
            frame: Vec<u8>,
        }

        impl BleGattWriter for Writer {
            type Error = core::convert::Infallible;

            fn write_frame(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
                self.frame = frame.to_vec();
                Ok(())
            }
        }

        let mut writer = Writer::default();
        let command = write_fountain_ble_frame(&mut writer, petkit_types::FountainAction::Pause, 5)
            .expect("write should succeed");

        assert_eq!(writer.frame, command.frame);
        assert_eq!(command.cmd, 220);
    }

    #[test]
    fn fountain_ble_client_builds_setting_backed_actions() {
        let settings = FountainBleSettings::new(5, 40, true, 2, 300, 600, false, 1320, 360)
            .expect("settings should be valid");
        let command = FountainBleClient::new(petkit_types::FountainDeviceType::W5)
            .command_with_settings(petkit_types::FountainAction::LightHigh, 9, &settings)
            .expect("light command should include settings");

        assert_eq!(command.cmd, 221);
        assert_eq!(
            command.frame,
            vec![
                250, 252, 253, 221, 1, 9, 13, 0, 5, 40, 1, 3, 1, 44, 2, 88, 0, 5, 40, 1, 104, 251
            ]
        );
    }

    #[test]
    fn fountain_ble_brightness_rejects_lamp_off_settings() {
        let settings = FountainBleSettings::new(5, 40, false, 2, 300, 600, false, 1320, 360)
            .expect("settings should be valid");

        let error = FountainBleClient::new(petkit_types::FountainDeviceType::W5)
            .command_with_settings(petkit_types::FountainAction::LightHigh, 9, &settings)
            .expect_err("brightness should require the light to be on");

        assert!(matches!(error, PetkitError::InvalidArgument(_)));
    }

    #[test]
    fn fountain_ble_settings_can_be_read_from_cloud_device_detail() {
        let response = ResponseParts::new(
            200,
            vec![],
            br#"{"result":{"settings":{"smartWorkingTime":"5","smartSleepTime":40,"lampRingSwitch":1,"lampRingBrightness":2,"lampRingLightUpTime":300,"lampRingGoOutTime":600,"noDisturbingSwitch":false,"noDisturbingStartTime":1320,"noDisturbingEndTime":360}}}"#.to_vec(),
        );
        let detail: DeviceDetailResponse =
            parse_api_response(&response).expect("device detail should parse");

        let settings = FountainBleSettings::from_device_detail(&detail)
            .expect("fountain BLE settings should parse");

        assert_eq!(
            settings,
            FountainBleSettings::new(5, 40, true, 2, 300, 600, false, 1320, 360)
                .expect("settings should be valid")
        );
    }

    #[test]
    fn fountain_ble_settings_reject_out_of_range_values() {
        assert!(FountainBleSettings::new(5, 40, true, 4, 300, 600, false, 1320, 360).is_err());
        assert!(FountainBleSettings::new(5, 40, true, 2, 1440, 600, false, 1320, 360).is_err());
        assert!(FountainBleSettings::new(5, 40, true, 2, 300, 600, false, 1440, 360).is_err());
    }

    #[test]
    fn cloud_ble_requests_use_relay_endpoints_and_form_fields() {
        let connect = CloudBleConnectRequest::new("42", "W5", "aa:bb").with_group_id("7");
        let request = authenticated().cloud_ble().connect(&connect);
        assert_eq!(request.path, "ble/connect");
        assert!(
            request
                .form_fields
                .iter()
                .any(|f| f.name == "bleId" && f.value == "42")
        );
        assert!(
            request
                .form_fields
                .iter()
                .any(|f| f.name == "type" && f.value == "W5")
        );
        assert!(
            request
                .form_fields
                .iter()
                .any(|f| f.name == "mac" && f.value == "aa:bb")
        );
        assert!(
            request
                .form_fields
                .iter()
                .any(|f| f.name == "groupId" && f.value == "7")
        );

        let control = CloudBleControlRequest::new("42", "W5", "aa:bb", "220", "encoded");
        let request = authenticated().cloud_ble().control_device(&control);
        assert_eq!(request.path, "ble/controlDevice");
        assert!(
            request
                .form_fields
                .iter()
                .any(|f| f.name == "cmd" && f.value == "220")
        );
        assert!(
            request
                .form_fields
                .iter()
                .any(|f| f.name == "data" && f.value == "encoded")
        );
    }

    #[test]
    fn camera_rtm_peer_message_builds_agora_json_request() {
        let live_feed = CameraLiveFeed {
            accepted: true,
            whep_url: None,
            app_id: Some(String::from("app-id")),
            channel_id: Some(String::from("channel")),
            rtc_token: Some(String::from("rtc")),
            rtm_token: Some(String::from("rtm")),
            uid: Some(1001),
            app_rtm_user_id: Some(String::from("app_1001")),
            dev_rtm_user_id: Some(String::from("dev_2001")),
        };
        let request = camera_rtm_peer_message(
            &live_feed,
            &CameraRtmCommand::PtzControl {
                kind: PtzKind::Move,
                direction: PtzDirection::Left,
            },
        )
        .expect("rtm request should build");
        assert!(
            request
                .url
                .contains("/dev/v2/project/app-id/rtm/users/app_1001/peer_messages")
        );
        assert!(request.url.contains("wait_for_ack=true"));
        assert!(
            request
                .headers
                .iter()
                .any(|header| header.name == "x-agora-token" && header.value.as_str() == "rtm")
        );
        assert!(
            request.headers.iter().any(|header| {
                header.name == "x-agora-uid" && header.value.as_str() == "app_1001"
            })
        );
        assert!(request.headers.iter().any(|header| {
            header.name == "Authorization" && header.value.as_str() == "agora token=rtm"
        }));
        let body = request.body.expect("json body should exist").body;
        assert!(body.contains(r#""destination":"dev_2001""#));
        assert!(body.contains(r#""cmd\":\"ptz_ctrl\""#));
    }

    #[test]
    fn camera_start_live_uses_get_query_parameters() {
        use petkit_types::LitterDeviceType;

        let feeder = authenticated()
            .feeder(FeederDeviceType::D4sh, device_id(42))
            .with_model::<D4shFeeder>()
            .start_live();
        assert_eq!(feeder.method, HttpMethod::Get);
        assert_eq!(feeder.form_fields, []);
        assert!(
            feeder
                .query
                .iter()
                .any(|field| field.name == "deviceId" && field.value == "42")
        );
        assert!(
            feeder
                .query
                .iter()
                .any(|field| field.name == "definition" && field.value == "2")
        );

        let litter = authenticated()
            .litter(LitterDeviceType::T6, device_id(77))
            .with_model::<T6Litter>()
            .start_live();
        assert_eq!(litter.method, HttpMethod::Get);
        assert_eq!(litter.form_fields, []);
        assert!(
            litter
                .query
                .iter()
                .any(|field| field.name == "deviceId" && field.value == "77")
        );
    }

    #[test]
    fn feeder_mini_settings_use_settings_prefix() {
        use petkit_types::FeederSetting;

        let request = authenticated()
            .feeder(FeederDeviceType::FeederMini, device_id(42))
            .update_setting(&FeederSetting::LightMode(true));
        let kv = request
            .form_fields
            .iter()
            .find(|f| f.name == "kv")
            .expect("kv field must exist");
        assert_eq!(kv.value, r#"{"settings.lightMode":1}"#);
        assert_eq!(request.path, "feedermini/update");
    }

    #[test]
    fn feeder_mini_feed_notify_uses_settings_prefix() {
        use petkit_types::FeederSetting;

        let request = authenticated()
            .feeder(FeederDeviceType::FeederMini, device_id(42))
            .update_setting(&FeederSetting::FeedNotify(true));
        let kv = request
            .form_fields
            .iter()
            .find(|f| f.name == "kv")
            .expect("kv field must exist");
        assert_eq!(kv.value, r#"{"settings.feedNotify":1}"#);
    }

    #[test]
    fn modern_feeder_settings_use_bare_keys() {
        use petkit_types::FeederSetting;

        let request = authenticated()
            .feeder(FeederDeviceType::D4, device_id(42))
            .update_setting(&FeederSetting::LightMode(true));
        let kv = request
            .form_fields
            .iter()
            .find(|f| f.name == "kv")
            .expect("kv field must exist");
        assert_eq!(kv.value, r#"{"lightMode":1}"#);
        assert_eq!(request.path, "d4/updateSettings");
    }

    #[test]
    fn feeder_device_detail_uses_id_field() {
        let request = authenticated()
            .feeder(FeederDeviceType::D4s, device_id(42))
            .device_detail();
        let id = request
            .form_fields
            .iter()
            .find(|field| field.name == "id")
            .expect("id field must exist");

        assert_eq!(request.path, "d4s/device_detail");
        assert_eq!(id.value, "42");
    }

    #[test]
    fn purifier_k3_device_detail_uses_device_data_path() {
        let request = authenticated()
            .purifier(PurifierDeviceType::K3, device_id(77))
            .device_detail();
        let id = request
            .form_fields
            .iter()
            .find(|field| field.name == "id")
            .expect("id field must exist");

        assert_eq!(request.path, "k3/deviceData");
        assert_eq!(id.value, "77");
    }

    #[test]
    fn fountain_device_detail_uses_device_data_path() {
        use petkit_types::FountainDeviceType;

        let request = authenticated()
            .fountain(FountainDeviceType::Ctw3, device_id(42))
            .device_detail();
        let id = request
            .form_fields
            .iter()
            .find(|field| field.name == "id")
            .expect("id field must exist");

        assert_eq!(request.path, "ctw3/deviceData");
        assert_eq!(id.value, "42");
    }

    #[test]
    fn parse_api_response_maps_petkit_error_codes() {
        let response = ResponseParts::new(
            200,
            vec![],
            br#"{"error":{"code":5,"msg":"expired"}}"#.to_vec(),
        );
        let error = parse_api_response::<alloc::string::String>(&response)
            .expect_err("response should map to a protocol error");

        assert!(matches!(
            error,
            PetkitError::Api {
                code: PetkitErrorCode::SessionExpired,
                ..
            }
        ));
    }

    #[test]
    fn parse_api_response_returns_http_status_for_non_2xx() {
        let response = ResponseParts::new(500, vec![], br#"{"result":"ok"}"#.to_vec());
        let error = parse_api_response::<alloc::string::String>(&response)
            .expect_err("non-2xx responses should return HTTP status errors");

        assert!(matches!(error, PetkitError::HttpStatus { status: 500 }));
    }

    #[test]
    fn parse_api_response_rejects_missing_result_in_success_envelope() {
        let response = ResponseParts::new(200, vec![], br#"{"error":null}"#.to_vec());
        let error = parse_api_response::<alloc::string::String>(&response)
            .expect_err("successful envelopes without result should be rejected");

        assert!(matches!(
            error,
            PetkitError::InvalidResponse("missing `result` field")
        ));
    }

    #[test]
    fn parse_api_response_returns_decode_for_invalid_json() {
        let response = ResponseParts::new(200, vec![], br#"{"result":"ok""#.to_vec());
        let error = parse_api_response::<alloc::string::String>(&response)
            .expect_err("invalid JSON should fail to decode");

        assert!(matches!(error, PetkitError::Decode(_)));
    }

    #[test]
    fn parse_text_response_returns_utf8_body() {
        let response = ResponseParts::new(200, vec![], b"hello".to_vec());
        let body = parse_text_response(response).expect("valid UTF-8 should parse");

        assert_eq!(body, "hello");
    }

    #[test]
    fn parse_text_response_returns_decode_for_invalid_utf8() {
        let response = ResponseParts::new(200, vec![], vec![0xff, 0xfe]);
        let error = parse_text_response(response).expect_err("invalid UTF-8 should fail to decode");

        assert!(matches!(error, PetkitError::Decode(_)));
    }
}
