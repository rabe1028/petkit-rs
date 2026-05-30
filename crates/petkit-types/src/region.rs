use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use nojson::{JsonParseError, JsonValueKind, RawJsonValue};

pub const DEFAULT_COUNTRY: &str = "DE";
pub const DEFAULT_TIMEZONE: &str = "Europe/Berlin";
pub const PASSPORT_BASE_URL: &str = "https://passport.petkt.com/";
pub const CHINA_BASE_URL: &str = "https://api.petkit.cn/6/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionServer {
    pub account_type: String,
    pub gateway: String,
    pub id: String,
    pub name: String,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for RegionServer {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            account_type: value.to_member("accountType")?.required()?.try_into()?,
            gateway: value.to_member("gateway")?.required()?.try_into()?,
            id: value.to_member("id")?.required()?.try_into()?,
            name: value.to_member("name")?.required()?.try_into()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RegionServersPayload {
    pub list: Vec<RegionServer>,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for RegionServersPayload {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let list = match value.to_member("list")?.optional() {
            Some(member) if member.kind() == JsonValueKind::Null => Vec::new(),
            Some(member) => member
                .to_array()?
                .map(RegionServer::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        Ok(Self { list })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionServerGroup {
    pub gateway: String,
    pub label: String,
    pub countries: Vec<String>,
    pub representative_country: String,
}

pub fn gateway_label(gateway: &str) -> &str {
    match gateway {
        "https://api.eu-pet.com/latest/" => "Europe",
        "https://api.petkt.com/latest/" => "International (Americas, global)",
        "https://api.petktasia.com/latest/" => "Asia",
        "https://api-ru.petkit.cn/latest/" => "Russia",
        CHINA_BASE_URL => "China",
        _ => gateway,
    }
}

pub fn group_region_servers(payload: &RegionServersPayload) -> Vec<RegionServerGroup> {
    let mut countries_by_gateway: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in &payload.list {
        countries_by_gateway
            .entry(entry.gateway.clone())
            .or_default()
            .push(entry.id.clone());
    }

    let mut groups = countries_by_gateway
        .into_iter()
        .map(|(gateway, mut countries)| {
            countries.sort();
            let representative_country = countries
                .first()
                .cloned()
                .unwrap_or_else(|| String::from("CN"));
            RegionServerGroup {
                label: gateway_label(&gateway).to_string(),
                gateway,
                countries,
                representative_country,
            }
        })
        .collect::<Vec<_>>();

    if !groups.iter().any(|group| group.gateway == CHINA_BASE_URL) {
        groups.push(RegionServerGroup {
            gateway: CHINA_BASE_URL.to_string(),
            label: String::from("China"),
            countries: vec![String::from("CN")],
            representative_country: String::from("CN"),
        });
    }

    groups.sort_by(|left, right| left.label.cmp(&right.label));
    groups
}
