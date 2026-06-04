//! Probe camera live-feed metadata with the async reqwest transport.
//!
//! Run with:
//! `cargo run -p petkit-client --example reqwest_async_camera_live_feed_probe --no-default-features --features async,reqwest-async`
//!
//! Shared env vars:
//! `PETKIT_EMAIL`, `PETKIT_PASSWORD`, `PETKIT_REGION`, `PETKIT_TIMEZONE_ID`,
//! `PETKIT_TIMEZONE_OFFSET`, `PETKIT_BASE_URL`
//!
//! Optional camera overrides:
//! `PETKIT_CAMERA_DEVICE_ID`, `PETKIT_CAMERA_DEVICE_NAME`

#![allow(clippy::print_stdout)]

#[cfg(not(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
)))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(
        "enable `--features async,reqwest-async` or `--features async,reqwest-native` to build this example"
            .into(),
    )
}

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
#[path = "support/common.rs"]
mod common;

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
use std::error::Error;

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
use petkit_client::ReqwestAsyncPetkitClient;
#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
use petkit_protocol::{D4hFeeder, D4shFeeder, T5Litter, T6Litter, T7Litter};
#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
use petkit_types::{
    CameraLiveFeed, DeviceFamilyKind, DeviceId, FeederDeviceType, LitterDeviceType,
};

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let region = common::region();
    let context = common::example_context();

    let discovery_client =
        ReqwestAsyncPetkitClient::new_reqwest(context.clone(), common::default_regional_base());
    let regions = discovery_client.fetch_region_servers().await?;

    let mut client = ReqwestAsyncPetkitClient::new_reqwest(
        context,
        common::resolve_regional_base(&regions, &region),
    );
    if let Some(code) = common::login_code() {
        client
            .login_with_code(&common::email(), &code, &region)
            .await?;
    } else {
        client
            .login_with_password(&common::email(), &common::password(), &region)
            .await?;
    }

    let families = client.family_list().await?;
    let device = common::select_camera_device(&families)?;
    let device_id = DeviceId::new(device.device_id)?;
    println!(
        "selected camera device id={} type={} name={}",
        device.device_id,
        device.device_type.as_str(),
        device
            .device_name
            .as_deref()
            .unwrap_or(device.unique_id.as_str())
    );

    let live_feed = match device.device_type.clone().into_family() {
        DeviceFamilyKind::Feeder(FeederDeviceType::D4h) => {
            client
                .authenticated()
                .feeder_typed::<D4hFeeder>(device_id)
                .camera_live_feed()
                .await?
        }
        DeviceFamilyKind::Feeder(FeederDeviceType::D4sh) => {
            client
                .authenticated()
                .feeder_typed::<D4shFeeder>(device_id)
                .camera_live_feed()
                .await?
        }
        DeviceFamilyKind::Litter(LitterDeviceType::T5) => {
            client
                .authenticated()
                .litter_typed::<T5Litter>(device_id)
                .camera_live_feed()
                .await?
        }
        DeviceFamilyKind::Litter(LitterDeviceType::T6) => {
            client
                .authenticated()
                .litter_typed::<T6Litter>(device_id)
                .camera_live_feed()
                .await?
        }
        DeviceFamilyKind::Litter(LitterDeviceType::T7) => {
            client
                .authenticated()
                .litter_typed::<T7Litter>(device_id)
                .camera_live_feed()
                .await?
        }
        DeviceFamilyKind::Feeder(other) => {
            return Err(format!("feeder type {other:?} does not support camera").into());
        }
        DeviceFamilyKind::Litter(other) => {
            return Err(format!("litter type {other:?} does not support camera").into());
        }
        other => {
            return Err(format!("device type {other:?} does not support camera").into());
        }
    };

    print_live_feed(&live_feed);

    let command = petkit_types::CameraRtmCommand::Heartbeat;
    let request = petkit_protocol::camera_rtm_peer_message(&live_feed, &command);
    println!("agora heartbeat request buildable: {}", request.is_ok());

    Ok(())
}

#[cfg(all(
    feature = "async",
    any(feature = "reqwest-async", feature = "reqwest-native")
))]
fn print_live_feed(live_feed: &CameraLiveFeed) {
    println!("accepted: {}", live_feed.accepted);
    println!("has whep_url: {}", live_feed.whep_url.is_some());
    println!("has app_id: {}", live_feed.app_id.is_some());
    println!("has channel_id: {}", live_feed.channel_id.is_some());
    println!("has rtc_token: {}", live_feed.rtc_token.is_some());
    println!("has rtm_token: {}", live_feed.rtm_token.is_some());
    println!(
        "has app_rtm_user_id: {}",
        live_feed.app_rtm_user_id.is_some()
    );
    println!(
        "has dev_rtm_user_id: {}",
        live_feed.dev_rtm_user_id.is_some()
    );
    println!("uid: {:?}", live_feed.uid);
}
