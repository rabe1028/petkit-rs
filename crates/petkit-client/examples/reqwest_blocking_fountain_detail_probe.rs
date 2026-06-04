//! Probe a fountain detail payload and Cloud BLE metadata with blocking reqwest.
//!
//! Run with:
//! `cargo run -p petkit-client --example reqwest_blocking_fountain_detail_probe --no-default-features --features blocking,reqwest-blocking`
//!
//! Shared env vars:
//! `PETKIT_EMAIL`, `PETKIT_PASSWORD`, `PETKIT_REGION`, `PETKIT_TIMEZONE_ID`,
//! `PETKIT_TIMEZONE_OFFSET`, and `PETKIT_BASE_URL`
//!
//! Optional fountain overrides:
//! `PETKIT_FOUNTAIN_DEVICE_ID`, `PETKIT_FOUNTAIN_DEVICE_TYPE`,
//! `PETKIT_FOUNTAIN_DEVICE_NAME`

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
use petkit_types::{DeviceSummary, FountainDeviceDetailResponse, flatten_devices};

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
    if let Some(code) = common::login_code() {
        client.login_with_code(&common::email(), &code, &region)?;
    } else {
        client.login_with_password(&common::email(), &common::password(), &region)?;
    }

    let families = client.family_list()?;
    println!("family count: {}", families.len());

    let selected = common::select_fountain_device(&families)?;
    println!("selected fountain: {}", selected.device_name);

    let summary = flatten_devices(&families)
        .into_iter()
        .find(|device| device.device_id == selected.device_id.get());
    print_summary("family summary", summary.as_ref());

    let fountain = client
        .authenticated()
        .fountain(selected.device_type, selected.device_id);
    let request = fountain.device_detail_request();
    println!("detail request path: {}", request.path);

    let detail: FountainDeviceDetailResponse = fountain.device_detail()?;
    print_detail(&detail);

    if let Some(summary) = summary.as_ref() {
        let resolved = client
            .authenticated()
            .cloud_ble()
            .resolve_cloud_ble_metadata(summary)?;
        match resolved {
            Some(metadata) => {
                println!("resolved cloud BLE metadata: true");
                println!("resolved device_type: {}", metadata.device_type);
                println!(
                    "resolved group_id: {}",
                    optional(metadata.group_id.as_deref())
                );
                println!("resolved mac: {}", metadata.mac);
                println!("resolved ble_id: {}", optional(metadata.ble_id.as_deref()));
            }
            None => println!("resolved cloud BLE metadata: false"),
        }
    } else {
        println!("family summary not found; metadata resolver skipped");
    }

    Ok(())
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
fn print_summary(label: &str, summary: Option<&DeviceSummary>) {
    let Some(summary) = summary else {
        println!("{label}: <missing>");
        return;
    };
    println!("{label}:");
    println!("  device_id: {}", summary.device_id);
    println!("  device_type: {}", summary.device_type.as_str());
    println!("  group_id: {}", summary.group_id);
    println!("  type: {}", optional_u64(summary.device_type_id));
    println!("  type_code: {}", optional_u64(summary.type_code));
    println!("  unique_id: {}", summary.unique_id);
    println!("  mac: {}", optional(summary.mac.as_deref()));
    println!("  ble_id: {}", optional(summary.ble_id.as_deref()));
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
fn print_detail(detail: &FountainDeviceDetailResponse) {
    println!("device detail:");
    println!("  id: {}", optional_u64(detail.id));
    println!("  device_type: {}", optional(detail.device_type.as_deref()));
    println!("  group_id: {}", optional_u64(detail.group_id));
    println!("  mac: {}", optional(detail.mac.as_deref()));
    println!("  ble_id: {}", optional(detail.ble_id.as_deref()));
    println!("  sn: {}", optional(detail.sn.as_deref()));
    println!("  firmware: {}", optional(detail.firmware.as_deref()));
    println!(
        "  cloud_ble_metadata: {}",
        detail.cloud_ble_metadata().is_some()
    );
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
fn optional(value: Option<&str>) -> &str {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("<missing>")
}

#[cfg(all(
    feature = "blocking",
    any(feature = "reqwest-blocking", feature = "reqwest-native")
))]
fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| String::from("<missing>"), |value| value.to_string())
}
