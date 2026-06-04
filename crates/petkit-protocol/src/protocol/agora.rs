use alloc::format;
use alloc::string::String;

use petkit_types::{CameraLiveFeed, CameraRtmCommand, PetkitError, json_string};

use crate::{BaseUrl, HttpMethod, RequestSpec};

pub fn camera_rtm_peer_message(
    live_feed: &CameraLiveFeed,
    command: &CameraRtmCommand,
) -> Result<RequestSpec, PetkitError> {
    let app_id = required(live_feed.app_id.as_deref(), "app_id")?;
    let token = required(live_feed.rtm_token.as_deref(), "rtm_token")?;
    let user_id = required(live_feed.app_rtm_user_id.as_deref(), "app_rtm_user_id")?;
    let destination = required(live_feed.dev_rtm_user_id.as_deref(), "dev_rtm_user_id")?;
    let payload = command.payload_json();
    let body = format!(
        r#"{{"destination":{},"enable_offline_messaging":false,"enable_historical_messaging":false,"payload":{}}}"#,
        json_string(destination),
        json_string(&payload)
    );
    Ok(RequestSpec::new(
        HttpMethod::Post,
        &BaseUrl::Absolute(String::from("https://api.agora.io")),
        format!("/dev/v2/project/{app_id}/rtm/users/{user_id}/peer_messages?wait_for_ack=true"),
    )
    .push_header("Authorization", format!("Bearer {token}"))
    .push_header("Content-Type", "application/json")
    .with_json_body(body))
}

fn required<'a>(value: Option<&'a str>, field: &'static str) -> Result<&'a str, PetkitError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PetkitError::InvalidArgument(format!("camera RTM command requires {field}")))
}
