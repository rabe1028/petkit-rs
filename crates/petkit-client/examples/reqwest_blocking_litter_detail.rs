//! Read a litter box detail payload with the blocking reqwest transport.
//!
//! Run with:
//! `cargo run -p petkit-client --example reqwest_blocking_litter_detail --no-default-features --features blocking,reqwest-blocking`
//!
//! Shared env vars:
//! `PETKIT_EMAIL`, `PETKIT_PASSWORD`, `PETKIT_REGION`, `PETKIT_TIMEZONE_ID`,
//! `PETKIT_TIMEZONE_OFFSET`, `PETKIT_BASE_URL`
//!
//! Optional litter overrides:
//! `PETKIT_LITTER_DEVICE_ID`, `PETKIT_LITTER_DEVICE_TYPE`, `PETKIT_LITTER_DEVICE_NAME`

#![allow(clippy::print_stdout)]

#[cfg(not(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
)))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(
        "enable `--features blocking,reqwest-blocking` or `--features blocking,reqwest-native` to build this example"
            .into(),
    )
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
#[path = "support/common.rs"]
mod common;

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
use std::error::Error;

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
use petkit_client::ReqwestBlockingPetkitClient;
#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
use petkit_types::LitterDeviceDetailResponse;

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
fn main() -> Result<(), Box<dyn Error>> {
    let region = common::region();
    let context = common::example_context();

    let discovery_client =
        ReqwestBlockingPetkitClient::new_reqwest(context.clone(), common::default_regional_base());
    let regions = discovery_client.fetch_region_servers()?;

    let mut client = ReqwestBlockingPetkitClient::new_reqwest(
        context,
        common::resolve_regional_base(&regions, &region),
    );
    client.login_with_password(&common::email(), &common::password(), &region)?;

    let families = client.family_list()?;
    let device = common::select_litter_device(&families)?;
    println!("selected litter box: {}", device.device_name);

    let litter = client
        .authenticated()
        .litter(device.device_type, device.device_id);
    let request = litter.device_detail_request();
    println!("request path: {}", request.path);

    let response: LitterDeviceDetailResponse = litter.device_detail()?;
    common::print_device_detail(
        &device.device_name,
        &response,
        &["autoWork", "manualLock", "volume"],
        &["workState", "sandPercent", "errorDetail"],
    )?;

    Ok(())
}
