//! Read a litter box detail payload with a custom ureq blocking transport.
//!
//! Run with:
//! `cargo run -p petkit-client --example ureq_blocking_litter_detail --no-default-features --features blocking,ureq-blocking`
//!
//! Shared env vars:
//! `PETKIT_EMAIL`, `PETKIT_PASSWORD`, `PETKIT_REGION`, `PETKIT_TIMEZONE_ID`,
//! `PETKIT_TIMEZONE_OFFSET`, `PETKIT_BASE_URL`
//!
//! Optional litter overrides:
//! `PETKIT_LITTER_DEVICE_ID`, `PETKIT_LITTER_DEVICE_TYPE`, `PETKIT_LITTER_DEVICE_NAME`

#![allow(clippy::print_stdout)]

#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
#[path = "support/common.rs"]
mod common;
#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
use std::error::Error;

#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
use petkit_client::UreqBlockingPetkitClient;
#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
use petkit_types::LitterDeviceDetailResponse;
#[cfg(all(feature = "blocking", feature = "ureq-blocking"))]
fn main() -> Result<(), Box<dyn Error>> {
    let region = common::region();
    let context = common::example_context();

    let discovery_client =
        UreqBlockingPetkitClient::new_ureq(context.clone(), common::default_regional_base());
    let regions = discovery_client.fetch_region_servers()?;

    let mut client = UreqBlockingPetkitClient::new_ureq(
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

#[cfg(not(all(feature = "blocking", feature = "ureq-blocking")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(
        std::io::Error::other("enable `--features blocking,ureq-blocking` to build this example")
            .into(),
    )
}
