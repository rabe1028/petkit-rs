use alloc::format;
use alloc::string::String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientProfile {
    pub locale: String,
    pub model_name: String,
    pub os_version: String,
    pub phone_brand: String,
    pub platform: String,
    pub source: String,
    pub api_version: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub user_agent: String,
    pub hour_format: String,
    pub image_version: String,
}

impl Default for ClientProfile {
    fn default() -> Self {
        Self {
            locale: String::from("en-US"),
            model_name: String::from("23127PN0CG"),
            os_version: String::from("16.1"),
            phone_brand: String::from("Xiaomi"),
            platform: String::from("android"),
            source: String::from("app.petkit-android"),
            api_version: String::from("13.2.1"),
            accept_language: String::from("en-US;q=1, it-US;q=0.9"),
            accept_encoding: String::from("gzip, deflate"),
            user_agent: String::from("okhttp/3.14.9"),
            hour_format: String::from("24"),
            image_version: String::from("1"),
        }
    }
}

impl ClientProfile {
    pub fn x_client_value(&self) -> String {
        format!("{}({};{})", self.platform, self.os_version, self.model_name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientContext {
    pub profile: ClientProfile,
    pub timezone_id: String,
    pub timezone_offset_hours: String,
}

impl ClientContext {
    pub fn new(
        profile: ClientProfile,
        timezone_id: impl Into<String>,
        timezone_offset_hours: impl Into<String>,
    ) -> Self {
        Self {
            profile,
            timezone_id: timezone_id.into(),
            timezone_offset_hours: timezone_offset_hours.into(),
        }
    }

    pub fn render_client_descriptor(&self) -> String {
        format!(
            "{{'locale': '{}', 'name': '{}', 'osVersion': '{}', 'phoneBrand': '{}', 'platform': '{}', 'source': '{}', 'version': '{}', 'timezoneId': '{}', 'timezone': '{}'}}",
            self.profile.locale,
            self.profile.model_name,
            self.profile.os_version,
            self.profile.phone_brand,
            self.profile.platform,
            self.profile.source,
            self.profile.api_version,
            self.timezone_id,
            self.timezone_offset_hours,
        )
    }
}
