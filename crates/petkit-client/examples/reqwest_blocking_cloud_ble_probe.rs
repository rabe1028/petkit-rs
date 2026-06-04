//! Probe PETKIT Cloud BLE relay metadata with the blocking reqwest transport.
//!
//! Run with:
//! `cargo run -p petkit-client --example reqwest_blocking_cloud_ble_probe --no-default-features --features blocking,reqwest-blocking`
//!
//! Shared env vars:
//! `PETKIT_EMAIL`, `PETKIT_PASSWORD`, `PETKIT_REGION`, `PETKIT_TIMEZONE_ID`,
//! `PETKIT_TIMEZONE_OFFSET`, `PETKIT_BASE_URL`
//!
//! Optional:
//! `PETKIT_CLOUD_BLE_CONNECT=1` attempts `ble/connect` and `ble/poll` for the
//! first discovered device with Cloud BLE metadata. By default this example is
//! read-only and only calls discovery endpoints.

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
use petkit_types::{CloudBleConnectRequest, flatten_devices};

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

    let cloud_ble = client.authenticated().cloud_ble();
    for family in &families {
        let Some(group_id) = family.group_id else {
            continue;
        };
        let supported = cloud_ble.supported_devices_for_group(group_id)?;
        println!(
            "supported BLE relay devices for group {group_id}: {}",
            supported.len()
        );
        for device in &supported {
            println!(
                "relay device id={} mac={} type={:?} low_version={:?}",
                device.id, device.mac, device.type_id, device.low_version
            );
        }
    }

    let mut selected = None;
    let devices = flatten_devices(&families);
    println!("device count: {}", devices.len());
    for device in devices {
        let metadata = device.cloud_ble_metadata();
        println!(
            "device id={} type={} name={} cloud_ble_metadata={}",
            device.device_id,
            device.device_type.as_str(),
            device
                .device_name
                .as_deref()
                .unwrap_or(device.unique_id.as_str()),
            metadata.is_some()
        );
        if selected.is_none() {
            selected = metadata.map(|metadata| (device.device_id.to_string(), metadata));
        }
    }

    if common::env_flag("PETKIT_CLOUD_BLE_CONNECT") {
        let (device_id, metadata) =
            selected.ok_or("no device exposes Cloud BLE metadata; cannot connect")?;
        let request = CloudBleConnectRequest::from_metadata(&metadata, device_id)?;
        let connection = cloud_ble.connect(&request)?;
        println!(
            "connect accepted={} state={:?}",
            connection.accepted, connection.state
        );
        let state = cloud_ble.poll(&request)?;
        println!("poll state={state:?}");
    } else {
        println!("set PETKIT_CLOUD_BLE_CONNECT=1 to also call ble/connect and ble/poll");
    }

    Ok(())
}
