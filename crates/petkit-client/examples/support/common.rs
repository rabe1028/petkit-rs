#![allow(dead_code)]

use std::env;
use std::error::Error;
use std::io;

use nojson::{JsonValueKind, RawJsonOwned};
use petkit_protocol::BaseUrl;
use petkit_types::{
    AccountGroup, ClientContext, ClientProfile, DeviceDetailResponse, DeviceId, DeviceSummary,
    DeviceType, FeederDeviceType, LitterDeviceType, PetkitError, PurifierDeviceType,
    RegionServersPayload,
};

#[derive(Clone, Debug)]
pub(crate) struct SelectedDevice<K> {
    pub(crate) device_type: K,
    pub(crate) device_id: DeviceId,
    pub(crate) device_name: String,
}

pub(crate) fn email() -> String {
    env_or("PETKIT_EMAIL", "user@example.com")
}

pub(crate) fn password() -> String {
    env_or("PETKIT_PASSWORD", "password")
}

pub(crate) fn region() -> String {
    env_or("PETKIT_REGION", "DE")
}

pub(crate) fn example_context() -> ClientContext {
    ClientContext::new(
        ClientProfile::default(),
        env_or("PETKIT_TIMEZONE_ID", "UTC"),
        env_or("PETKIT_TIMEZONE_OFFSET", "0"),
    )
}

pub(crate) fn default_regional_base() -> BaseUrl {
    BaseUrl::Regional(env_or("PETKIT_BASE_URL", "https://api.petkt.com/latest/"))
}

pub(crate) fn resolve_regional_base(regions: &RegionServersPayload, region: &str) -> BaseUrl {
    regions
        .list
        .iter()
        .find(|server| server.id.eq_ignore_ascii_case(region))
        .map(|server| BaseUrl::Regional(server.gateway.clone()))
        .unwrap_or_else(default_regional_base)
}

pub(crate) fn select_feeder_device(
    groups: &[AccountGroup],
) -> Result<SelectedDevice<FeederDeviceType>, io::Error> {
    select_device::<FeederDeviceType>(
        groups,
        "feeder",
        "PETKIT_FEEDER_DEVICE_ID",
        "PETKIT_FEEDER_DEVICE_TYPE",
        "PETKIT_FEEDER_DEVICE_NAME",
    )
}

pub(crate) fn select_litter_device(
    groups: &[AccountGroup],
) -> Result<SelectedDevice<LitterDeviceType>, io::Error> {
    select_device::<LitterDeviceType>(
        groups,
        "litter",
        "PETKIT_LITTER_DEVICE_ID",
        "PETKIT_LITTER_DEVICE_TYPE",
        "PETKIT_LITTER_DEVICE_NAME",
    )
}

pub(crate) fn select_purifier_device(
    groups: &[AccountGroup],
) -> Result<SelectedDevice<PurifierDeviceType>, io::Error> {
    select_device::<PurifierDeviceType>(
        groups,
        "purifier",
        "PETKIT_PURIFIER_DEVICE_ID",
        "PETKIT_PURIFIER_DEVICE_TYPE",
        "PETKIT_PURIFIER_DEVICE_NAME",
    )
}

pub(crate) fn print_device_detail(
    label: &str,
    response: &DeviceDetailResponse,
    settings_keys: &[&str],
    state_keys: &[&str],
) -> Result<(), Box<dyn Error>> {
    println!("selected device: {label}");
    if let Some(id) = response.id {
        println!("detail id: {id}");
    }
    if let Some(name) = &response.name {
        println!("detail name: {name}");
    }
    if let Some(sn) = &response.sn {
        println!("detail sn: {sn}");
    }
    if let Some(firmware) = &response.firmware {
        println!("detail firmware: {firmware}");
    }

    for key in settings_keys {
        let value = response.settings_member(key)?.map_or_else(
            || String::from("<missing>"),
            |value| render_raw_json(&value),
        );
        println!("settings.{key}: {value}");
    }

    for key in state_keys {
        let value = response.state_member(key)?.map_or_else(
            || String::from("<missing>"),
            |value| render_raw_json(&value),
        );
        println!("state.{key}: {value}");
    }

    Ok(())
}

fn select_device<K>(
    groups: &[AccountGroup],
    label: &str,
    id_key: &str,
    type_key: &str,
    name_key: &str,
) -> Result<SelectedDevice<K>, io::Error>
where
    K: TryFrom<DeviceType, Error = PetkitError> + Copy + Eq,
{
    let requested_id = env_device_id(id_key)?;
    let requested_type = env_device_type::<K>(label, type_key)?;
    let requested_name = env::var(name_key).ok();

    for device in groups.iter().flat_map(|group| group.device_list.iter()) {
        let Ok(device_type) = K::try_from(device.device_type.clone()) else {
            continue;
        };
        let Ok(device_id) = DeviceId::new(device.device_id) else {
            continue;
        };
        if requested_id.is_some_and(|requested_id| device_id != requested_id) {
            continue;
        }
        if requested_type.is_some_and(|requested_type| device_type != requested_type) {
            continue;
        }
        return Ok(SelectedDevice {
            device_type,
            device_id,
            device_name: display_device_name(device),
        });
    }

    if let (Some(device_id), Some(device_type)) = (requested_id, requested_type) {
        return Ok(SelectedDevice {
            device_type,
            device_id,
            device_name: requested_name.unwrap_or_else(|| format!("{label} #{device_id}")),
        });
    }

    Err(io::Error::other(format!(
        "no {label} device found; set both {id_key} and {type_key} to target one explicitly"
    )))
}

fn env_device_id(key: &str) -> Result<Option<DeviceId>, io::Error> {
    match env::var(key) {
        Ok(value) => {
            let id = value
                .trim()
                .parse::<u64>()
                .map_err(|error| io::Error::other(format!("invalid integer in {key}: {error}")))?;
            DeviceId::new(id)
                .map(Some)
                .map_err(|error| io::Error::other(format!("invalid {key}: {error}")))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(io::Error::other(format!("failed to read {key}: {error}"))),
    }
}

fn env_device_type<K>(label: &str, key: &str) -> Result<Option<K>, io::Error>
where
    K: TryFrom<DeviceType, Error = PetkitError>,
{
    match env::var(key) {
        Ok(value) => K::try_from(DeviceType::from(value.trim().to_ascii_lowercase()))
            .map(Some)
            .map_err(|error| {
                io::Error::other(format!("invalid {label} device type in {key}: {error}"))
            }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(io::Error::other(format!("failed to read {key}: {error}"))),
    }
}

fn display_device_name(device: &DeviceSummary) -> String {
    device
        .device_name
        .clone()
        .unwrap_or_else(|| device.unique_id.clone())
}

fn render_raw_json(value: &RawJsonOwned) -> String {
    match value.value().kind() {
        JsonValueKind::String => value
            .value()
            .to_unquoted_string_str()
            .map(|value| value.into_owned())
            .unwrap_or_else(|_| value.text().to_owned()),
        _ => value.text().to_owned(),
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}
