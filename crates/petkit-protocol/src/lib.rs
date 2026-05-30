#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod ble;
mod protocol;
mod request;
mod response;

pub use ble::{
    build_ble_frame, build_fountain_ble_command, encode_ble_data, BleEncodedCommand, BLE_END_FRAME,
    BLE_START_FRAME,
};
pub use protocol::{
    AuthenticatedProtocol, D3Feeder, D4Feeder, D4hFeeder, D4sFeeder, D4shFeeder,
    DualHopperFeederModel, DualManualFeedAmount, DynamicFeeder, DynamicLitter, FeederMiniFeeder,
    FeederModel, FeederScope, FeederSupportsCalibration, FeederSupportsCallPet,
    FeederSupportsCamera, FeederSupportsFoodReplenished, FeederSupportsSound, FountainScope,
    FreshElementFeeder, LitterModel, LitterScope, LitterSupportsCamera,
    LitterSupportsN50Deodorizer, ManualFeedAmount, PetScope, PublicProtocol, PurifierScope,
    SingleHopperFeederModel, SingleManualFeedAmount, T3Litter, T4Litter, T5Litter, T6Litter,
    T7Litter,
};
pub use request::{
    session_headers, BaseUrl, FormField, Header, HttpMethod, QueryField, RequestSpec, ResponseParts,
};
pub use response::{parse_api_response, parse_text_response};

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use petkit_types::{
        ClientContext, ClientProfile, DeviceId, FeederDeviceType, PetkitDay, PetkitError,
        PetkitErrorCode, PurifierDeviceType,
    };

    use super::{
        build_ble_frame, build_fountain_ble_command, parse_api_response, parse_text_response,
        AuthenticatedProtocol, BaseUrl, D4sFeeder, DualManualFeedAmount, FeederMiniFeeder,
        PublicProtocol, ResponseParts, SingleManualFeedAmount,
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
