use alloc::string::String;
use alloc::vec::Vec;

use nojson::{JsonParseError, RawJsonValue};

use crate::DeviceType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_in: u64,
    pub region: Option<String>,
    pub created_at: String,
    pub refreshed_at: Option<String>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for Session {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.to_member("id")?.required()?.try_into()?,
            user_id: value.to_member("userId")?.required()?.try_into()?,
            expires_in: value.to_member("expiresIn")?.required()?.try_into()?,
            region: optional_member(value.to_member("region")?)?,
            created_at: value.to_member("createdAt")?.required()?.try_into()?,
            refreshed_at: optional_member(value.to_member("refreshedAt")?)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct IotDeviceInfo {
    pub created_at: Option<String>,
    pub device_name: Option<String>,
    pub device_secret: Option<String>,
    pub id: Option<u64>,
    pub iot_instance_id: Option<String>,
    pub iot_platform: Option<String>,
    pub mqtt_host: Option<String>,
    pub mqtt_ip: Option<String>,
    pub product_key: Option<String>,
    pub region_id: Option<String>,
    pub standby_mqtt_host: Option<String>,
    pub standby_mqtt_ip: Option<String>,
    pub device_type_id: Option<u64>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for IotDeviceInfo {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            created_at: optional_member(value.to_member("createdAt")?)?,
            device_name: optional_member(value.to_member("deviceName")?)?,
            device_secret: optional_member(value.to_member("deviceSecret")?)?,
            id: optional_member(value.to_member("id")?)?,
            iot_instance_id: optional_member(value.to_member("iotInstanceId")?)?,
            iot_platform: optional_member(value.to_member("iotPlatform")?)?,
            mqtt_host: optional_member(value.to_member("mqttHost")?)?,
            mqtt_ip: optional_member(value.to_member("mqttIp")?)?,
            product_key: optional_member(value.to_member("productKey")?)?,
            region_id: optional_member(value.to_member("regionId")?)?,
            standby_mqtt_host: optional_member(value.to_member("standbyMqttHost")?)?,
            standby_mqtt_ip: optional_member(value.to_member("standbyMqttIp")?)?,
            device_type_id: optional_member(value.to_member("type")?)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct IotConfigSet {
    pub ali: Option<IotDeviceInfo>,
    pub petkit: Option<IotDeviceInfo>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for IotConfigSet {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            ali: optional_member(value.to_member("ali")?)?,
            petkit: optional_member(value.to_member("petkit")?)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceSummary {
    pub device_id: u64,
    pub device_name: Option<String>,
    pub device_type: DeviceType,
    pub group_id: u64,
    pub device_type_id: Option<u64>,
    pub type_code: Option<u64>,
    pub unique_id: String,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for DeviceSummary {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            device_id: value.to_member("deviceId")?.required()?.try_into()?,
            device_name: optional_member(value.to_member("deviceName")?)?,
            device_type: value.to_member("deviceType")?.required()?.try_into()?,
            group_id: value.to_member("groupId")?.required()?.try_into()?,
            device_type_id: optional_member(value.to_member("type")?)?,
            type_code: optional_member(value.to_member("typeCode")?)?,
            unique_id: value.to_member("uniqueId")?.required()?.try_into()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetSummary {
    pub avatar: Option<String>,
    pub pet_id: u64,
    pub pet_name: String,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for PetSummary {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            avatar: optional_member(value.to_member("avatar")?)?,
            pet_id: value.to_member("petId")?.required()?.try_into()?,
            pet_name: value.to_member("petName")?.required()?.try_into()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountGroup {
    pub device_list: Vec<DeviceSummary>,
    pub group_id: Option<u64>,
    pub name: Option<String>,
    pub pet_list: Vec<PetSummary>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for AccountGroup {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            device_list: optional_list(value.to_member("deviceList")?)?,
            group_id: optional_member(value.to_member("groupId")?)?,
            name: optional_member(value.to_member("name")?)?,
            pet_list: optional_list(value.to_member("petList")?)?,
        })
    }
}

fn optional_member<'text, 'raw, 'a, T>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Option<T>, JsonParseError>
where
    T: TryFrom<RawJsonValue<'text, 'raw>, Error = JsonParseError>,
{
    match member.optional() {
        Some(value) if value.kind() == nojson::JsonValueKind::Null => Ok(None),
        Some(value) => T::try_from(value).map(Some),
        None => Ok(None),
    }
}

fn optional_list<'text, 'raw, 'a, T>(
    member: nojson::RawJsonMember<'text, 'raw, 'a>,
) -> Result<Vec<T>, JsonParseError>
where
    T: TryFrom<RawJsonValue<'text, 'raw>, Error = JsonParseError>,
{
    match member.optional() {
        Some(value) if value.kind() == nojson::JsonValueKind::Null => Ok(Vec::new()),
        Some(value) => value
            .to_array()?
            .map(T::try_from)
            .collect::<Result<Vec<_>, _>>(),
        None => Ok(Vec::new()),
    }
}
