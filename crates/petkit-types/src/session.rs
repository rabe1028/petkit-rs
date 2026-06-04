use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use hmac::{Hmac, Mac};
use nojson::{JsonParseError, RawJsonValue};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;

use crate::{DeviceType, PetkitError};

fn secret_eq(a: &SecretString, b: &SecretString) -> bool {
    a.expose_secret() == b.expose_secret()
}

fn secret_opt_eq(a: &Option<SecretString>, b: &Option<SecretString>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => secret_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: SecretString,
    pub user_id: String,
    pub expires_in: u64,
    pub region: Option<String>,
    pub created_at: String,
    pub refreshed_at: Option<String>,
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        secret_eq(&self.id, &other.id)
            && self.user_id == other.user_id
            && self.expires_in == other.expires_in
            && self.region == other.region
            && self.created_at == other.created_at
            && self.refreshed_at == other.refreshed_at
    }
}

impl Eq for Session {}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for Session {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SecretString::from(String::try_from(value.to_member("id")?.required()?)?),
            user_id: value.to_member("userId")?.required()?.try_into()?,
            expires_in: value.to_member("expiresIn")?.required()?.try_into()?,
            region: optional_member(value.to_member("region")?)?,
            created_at: value.to_member("createdAt")?.required()?.try_into()?,
            refreshed_at: optional_member(value.to_member("refreshedAt")?)?,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct IotDeviceInfo {
    pub created_at: Option<String>,
    pub device_name: Option<String>,
    pub device_secret: Option<SecretString>,
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

impl PartialEq for IotDeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.created_at == other.created_at
            && self.device_name == other.device_name
            && secret_opt_eq(&self.device_secret, &other.device_secret)
            && self.id == other.id
            && self.iot_instance_id == other.iot_instance_id
            && self.iot_platform == other.iot_platform
            && self.mqtt_host == other.mqtt_host
            && self.mqtt_ip == other.mqtt_ip
            && self.product_key == other.product_key
            && self.region_id == other.region_id
            && self.standby_mqtt_host == other.standby_mqtt_host
            && self.standby_mqtt_ip == other.standby_mqtt_ip
            && self.device_type_id == other.device_type_id
    }
}

impl Eq for IotDeviceInfo {}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for IotDeviceInfo {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            created_at: optional_member(value.to_member("createdAt")?)?,
            device_name: optional_member(value.to_member("deviceName")?)?,
            device_secret: optional_member::<String>(value.to_member("deviceSecret")?)?
                .map(SecretString::from),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AliyunSecureMode {
    Plain,
    Tls,
}

impl AliyunSecureMode {
    #[must_use]
    pub const fn secure_mode_value(self) -> &'static str {
        match self {
            Self::Plain => "3",
            Self::Tls => "2",
        }
    }

    #[must_use]
    pub const fn default_port(self) -> u16 {
        match self {
            Self::Plain => 1883,
            Self::Tls => 8883,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliyunMqttOptions {
    pub secure_mode: AliyunSecureMode,
    pub client_id: Option<String>,
}

impl AliyunMqttOptions {
    #[must_use]
    pub const fn new(secure_mode: AliyunSecureMode) -> Self {
        Self {
            secure_mode,
            client_id: None,
        }
    }

    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }
}

impl Default for AliyunMqttOptions {
    fn default() -> Self {
        Self::new(AliyunSecureMode::Plain)
    }
}

#[derive(Clone, Debug)]
pub struct AliyunMqttConnectionSummary {
    pub broker_id: String,
    pub host: String,
    pub port: u16,
    pub product_key: String,
    pub device_name: String,
    pub client_id: String,
    pub username: String,
    pub password: SecretString,
    pub subscribe_topic: String,
    pub publish_topic: String,
}

impl PartialEq for AliyunMqttConnectionSummary {
    fn eq(&self, other: &Self) -> bool {
        self.broker_id == other.broker_id
            && self.host == other.host
            && self.port == other.port
            && self.product_key == other.product_key
            && self.device_name == other.device_name
            && self.client_id == other.client_id
            && self.username == other.username
            && secret_eq(&self.password, &other.password)
            && self.subscribe_topic == other.subscribe_topic
            && self.publish_topic == other.publish_topic
    }
}

impl Eq for AliyunMqttConnectionSummary {}

impl IotConfigSet {
    #[must_use]
    pub fn preferred_aliyun_device(&self) -> Option<&IotDeviceInfo> {
        self.petkit.as_ref().or(self.ali.as_ref())
    }

    pub fn aliyun_mqtt_connection_summary(
        &self,
        options: &AliyunMqttOptions,
    ) -> Result<AliyunMqttConnectionSummary, PetkitError> {
        self.preferred_aliyun_device()
            .ok_or_else(|| {
                PetkitError::InvalidArgument(String::from(
                    "IoT config does not contain a petkit or ali MQTT device",
                ))
            })?
            .aliyun_mqtt_connection_summary(options)
    }
}

impl IotDeviceInfo {
    pub fn aliyun_mqtt_connection_summary(
        &self,
        options: &AliyunMqttOptions,
    ) -> Result<AliyunMqttConnectionSummary, PetkitError> {
        let product_key = required_iot_field(self.product_key.as_deref(), "productKey")?;
        let device_name = required_iot_field(self.device_name.as_deref(), "deviceName")?;
        let device_secret = required_iot_field(
            self.device_secret.as_ref().map(ExposeSecret::expose_secret),
            "deviceSecret",
        )?;
        let (host, port) = self.mqtt_endpoint(options.secure_mode)?;
        let raw_client_id = options.client_id.as_deref().unwrap_or(device_name);
        let content =
            format!("clientId{raw_client_id}deviceName{device_name}productKey{product_key}");
        let mut mac = Hmac::<Sha256>::new_from_slice(device_secret.as_bytes())
            .map_err(|error| PetkitError::InvalidArgument(error.to_string()))?;
        mac.update(content.as_bytes());
        let password = hex::encode(mac.finalize().into_bytes());
        let base = format!("/{product_key}/{device_name}/user");

        Ok(AliyunMqttConnectionSummary {
            broker_id: String::from("petkit"),
            host,
            port,
            product_key: product_key.to_string(),
            device_name: device_name.to_string(),
            client_id: format!(
                "{raw_client_id}|securemode={},signmethod=hmacsha256|",
                options.secure_mode.secure_mode_value()
            ),
            username: format!("{device_name}&{product_key}"),
            password: SecretString::from(password),
            subscribe_topic: format!("{base}/get"),
            publish_topic: format!("{base}/update"),
        })
    }

    fn mqtt_endpoint(&self, secure_mode: AliyunSecureMode) -> Result<(String, u16), PetkitError> {
        if let Some(mqtt_host) = self.mqtt_host.as_deref() {
            return parse_mqtt_host(mqtt_host, secure_mode);
        }

        let product_key = required_iot_field(self.product_key.as_deref(), "productKey")?;
        let region_id = required_iot_field(self.region_id.as_deref(), "regionId")?;
        Ok((
            format!("{product_key}.iot-as-mqtt.{region_id}.aliyuncs.com"),
            secure_mode.default_port(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceSummary {
    pub device_id: u64,
    pub device_name: Option<String>,
    pub device_type: DeviceType,
    pub group_id: u64,
    pub mac: Option<String>,
    pub ble_id: Option<String>,
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
            mac: optional_string_any(
                value,
                &[
                    "mac",
                    "btMac",
                    "bt_mac",
                    "deviceMac",
                    "bleMac",
                    "ble_mac",
                    "bluetoothMac",
                    "bluetooth_mac",
                    "macAddress",
                    "mac_address",
                ],
            )?,
            ble_id: optional_string_any(
                value,
                &["bleId", "ble_id", "bleDeviceId", "ble_device_id"],
            )?,
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

fn optional_string_any(
    value: RawJsonValue<'_, '_>,
    keys: &[&'static str],
) -> Result<Option<String>, JsonParseError> {
    for key in keys {
        match value.to_member(key)?.optional() {
            Some(value) if value.kind() == nojson::JsonValueKind::Null => {}
            Some(value) => return String::try_from(value).map(Some),
            None => {}
        }
    }
    Ok(None)
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

fn required_iot_field<'a>(
    value: Option<&'a str>,
    field: &'static str,
) -> Result<&'a str, PetkitError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(PetkitError::InvalidArgument(format!(
            "IoT MQTT config missing {field}"
        )));
    };
    Ok(value)
}

fn parse_mqtt_host(raw: &str, secure_mode: AliyunSecureMode) -> Result<(String, u16), PetkitError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PetkitError::InvalidArgument(String::from(
            "IoT MQTT host must not be empty",
        )));
    }

    let lowered = trimmed.to_ascii_lowercase();
    let without_scheme = ["tcp://", "ssl://", "mqtt://", "mqtts://"]
        .iter()
        .find_map(|scheme| {
            lowered
                .starts_with(scheme)
                .then(|| &trimmed[scheme.len()..])
        })
        .unwrap_or(trimmed);

    let (host, port) = if let Some((host, port)) = without_scheme.rsplit_once(':') {
        if port.as_bytes().iter().all(u8::is_ascii_digit) {
            let parsed = port.parse::<u16>().map_err(|error| {
                PetkitError::InvalidArgument(format!("invalid IoT MQTT port `{port}`: {error}"))
            })?;
            (host, parsed)
        } else {
            (without_scheme, secure_mode.default_port())
        }
    } else {
        (without_scheme, secure_mode.default_port())
    };

    let host = host.trim();
    if host.is_empty() {
        return Err(PetkitError::InvalidArgument(format!(
            "invalid IoT MQTT host `{raw}`"
        )));
    }
    Ok((host.to_string(), port))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod iot_tests {
    use super::*;

    #[test]
    fn aliyun_mqtt_summary_matches_petkit_sidecar_shape() {
        let config = IotConfigSet {
            petkit: Some(IotDeviceInfo {
                mqtt_host: Some(String::from("ssl://mqtt.example:1883")),
                product_key: Some(String::from("pk123")),
                device_name: Some(String::from("dn456")),
                device_secret: Some(SecretString::from(String::from("secret789"))),
                ..IotDeviceInfo::default()
            }),
            ali: None,
        };

        let summary = config
            .aliyun_mqtt_connection_summary(&AliyunMqttOptions::default())
            .expect("mqtt summary should build");

        assert_eq!(summary.host, "mqtt.example");
        assert_eq!(summary.port, 1883);
        assert_eq!(
            summary.client_id,
            "dn456|securemode=3,signmethod=hmacsha256|"
        );
        assert_eq!(summary.username, "dn456&pk123");
        assert_eq!(summary.subscribe_topic, "/pk123/dn456/user/get");
        assert_eq!(summary.publish_topic, "/pk123/dn456/user/update");
        assert_eq!(
            summary.password.expose_secret(),
            "9e298bf7d08381ce089fc02b62ebc5fb740bb4622c10678639d84a8c71564ec0"
        );
    }

    #[test]
    fn aliyun_mqtt_summary_can_target_tls_defaults() {
        let info = IotDeviceInfo {
            product_key: Some(String::from("pk123")),
            device_name: Some(String::from("dn456")),
            device_secret: Some(SecretString::from(String::from("secret789"))),
            region_id: Some(String::from("cn-shanghai")),
            ..IotDeviceInfo::default()
        };

        let summary = info
            .aliyun_mqtt_connection_summary(&AliyunMqttOptions::new(AliyunSecureMode::Tls))
            .expect("tls summary should build");

        assert_eq!(summary.host, "pk123.iot-as-mqtt.cn-shanghai.aliyuncs.com");
        assert_eq!(summary.port, 8883);
        assert_eq!(
            summary.client_id,
            "dn456|securemode=2,signmethod=hmacsha256|"
        );
    }
}
