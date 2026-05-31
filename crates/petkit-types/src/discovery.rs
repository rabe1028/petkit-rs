use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{AccountGroup, DeviceId, DeviceSummary, PetkitError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceLookup<'a> {
    NumericId(u64),
    UniqueId(&'a str),
    OpaqueId(&'a str),
    Any(&'a str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceCatalog {
    devices: Vec<DeviceSummary>,
}

impl DeviceSummary {
    pub fn opaque_id(&self) -> String {
        format!("{}:{}", self.device_type.as_str(), self.device_id)
    }

    pub fn device_id_value(&self) -> Result<DeviceId, PetkitError> {
        DeviceId::new(self.device_id)
    }

    pub fn matches_lookup(&self, lookup: DeviceLookup<'_>) -> bool {
        match lookup {
            DeviceLookup::NumericId(id) => self.device_id == id,
            DeviceLookup::UniqueId(unique_id) => id_eq(&self.unique_id, unique_id),
            DeviceLookup::OpaqueId(opaque_id) => self.matches_opaque_id(opaque_id),
            DeviceLookup::Any(value) => self.matches_any_id(value),
        }
    }

    fn matches_any_id(&self, value: &str) -> bool {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return false;
        }
        if let Ok(id) = trimmed.parse::<u64>() {
            return self.device_id == id;
        }
        self.matches_opaque_id(trimmed) || id_eq(&self.unique_id, trimmed)
    }

    fn matches_opaque_id(&self, value: &str) -> bool {
        let trimmed = value.trim();
        if id_eq(&self.opaque_id(), trimmed) {
            return true;
        }
        let Some((kind, id)) = trimmed.split_once(':').or_else(|| trimmed.split_once('/')) else {
            return false;
        };
        id_eq(kind, self.device_type.as_str())
            && id.trim().parse::<u64>().ok() == Some(self.device_id)
    }
}

impl DeviceCatalog {
    pub fn from_groups(groups: &[AccountGroup]) -> Self {
        Self {
            devices: flatten_devices(groups),
        }
    }

    pub fn from_devices(devices: impl Into<Vec<DeviceSummary>>) -> Self {
        Self {
            devices: devices.into(),
        }
    }

    pub fn devices(&self) -> &[DeviceSummary] {
        &self.devices
    }

    pub fn into_devices(self) -> Vec<DeviceSummary> {
        self.devices
    }

    pub fn resolve(&self, lookup: DeviceLookup<'_>) -> Result<&DeviceSummary, PetkitError> {
        resolve_device(&self.devices, lookup)
    }

    pub fn resolve_id(&self, lookup: DeviceLookup<'_>) -> Result<DeviceId, PetkitError> {
        self.resolve(lookup)?.device_id_value()
    }
}

pub fn flatten_devices(groups: &[AccountGroup]) -> Vec<DeviceSummary> {
    groups
        .iter()
        .flat_map(|group| group.device_list.iter().cloned())
        .collect()
}

pub fn resolve_device<'a>(
    devices: &'a [DeviceSummary],
    lookup: DeviceLookup<'_>,
) -> Result<&'a DeviceSummary, PetkitError> {
    let mut matches = devices
        .iter()
        .filter(|device| device.matches_lookup(lookup))
        .take(2);
    let Some(device) = matches.next() else {
        return Err(PetkitError::InvalidArgument(String::from(
            "device identifier did not match any discovered device",
        )));
    };
    if matches.next().is_some() {
        return Err(PetkitError::InvalidArgument(String::from(
            "device identifier matched multiple discovered devices",
        )));
    }
    Ok(device)
}

fn id_eq(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use alloc::vec;

    use crate::DeviceType;

    use super::*;

    fn device(device_id: u64, unique_id: &str) -> DeviceSummary {
        DeviceSummary {
            device_id,
            device_name: Some(String::from("Kitchen feeder")),
            device_type: DeviceType::D4s,
            group_id: 1,
            device_type_id: Some(10),
            type_code: Some(20),
            unique_id: unique_id.to_string(),
        }
    }

    #[test]
    fn catalog_flattens_and_resolves_device_ids() {
        let groups = vec![AccountGroup {
            device_list: vec![device(42, "D4S_ABC")],
            group_id: Some(1),
            name: Some(String::from("home")),
            pet_list: Vec::new(),
        }];
        let catalog = DeviceCatalog::from_groups(&groups);

        assert_eq!(catalog.devices().len(), 1);
        assert_eq!(
            catalog
                .resolve(DeviceLookup::Any("d4s:42"))
                .expect("opaque id should resolve")
                .unique_id,
            "D4S_ABC"
        );
        assert_eq!(
            catalog
                .resolve(DeviceLookup::UniqueId("d4s_abc"))
                .expect("unique id should resolve")
                .device_id,
            42
        );
        assert_eq!(
            catalog
                .resolve_id(DeviceLookup::NumericId(42))
                .expect("numeric id should resolve")
                .get(),
            42
        );
    }
}
