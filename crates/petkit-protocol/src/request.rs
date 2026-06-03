use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use petkit_types::{CHINA_BASE_URL, ClientContext, PASSPORT_BASE_URL};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaseUrl {
    Passport,
    China,
    Regional(String),
    Absolute(String),
}

impl BaseUrl {
    #[must_use]
    pub fn join(&self, path: &str) -> String {
        let base = match self {
            Self::Passport => PASSPORT_BASE_URL,
            Self::China => CHINA_BASE_URL,
            Self::Regional(base) | Self::Absolute(base) => base.as_str(),
        };
        let trimmed_base = base.trim_end_matches('/');
        let trimmed_path = path.trim_start_matches('/');
        if trimmed_path.is_empty() {
            trimmed_base.to_owned()
        } else {
            format!("{trimmed_base}/{trimmed_path}")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub name: Cow<'static, str>,
    pub value: String,
}

impl Header {
    pub fn new(name: impl Into<Cow<'static, str>>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryField {
    pub name: Cow<'static, str>,
    pub value: String,
}

impl QueryField {
    pub fn new(name: impl Into<Cow<'static, str>>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormField {
    pub name: Cow<'static, str>,
    pub value: String,
}

impl FormField {
    pub fn new(name: impl Into<Cow<'static, str>>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestSpec {
    pub method: HttpMethod,
    pub url: String,
    pub path: String,
    pub headers: Vec<Header>,
    pub query: Vec<QueryField>,
    pub form_fields: Vec<FormField>,
}

impl RequestSpec {
    #[must_use]
    pub fn new(method: HttpMethod, base_url: &BaseUrl, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            method,
            url: base_url.join(&path),
            path,
            headers: Vec::new(),
            query: Vec::new(),
            form_fields: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_default_headers(mut self, context: &ClientContext) -> Self {
        self.headers.extend(default_headers(context));
        self
    }

    #[must_use]
    pub fn with_session_headers(mut self, session_id: &str) -> Self {
        self.headers.extend(session_headers(session_id));
        self
    }

    #[must_use]
    pub fn push_header(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.push(Header::new(name, value));
        self
    }

    #[must_use]
    pub fn push_query(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<String>,
    ) -> Self {
        self.query.push(QueryField::new(name, value));
        self
    }

    #[must_use]
    pub fn push_form_field(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<String>,
    ) -> Self {
        self.form_fields.push(FormField::new(name, value));
        self
    }

    /// Append every entry of an [`petkit_types::ExtraFormPayload`] as a form field.
    #[must_use]
    pub fn extend_form(mut self, payload: &petkit_types::ExtraFormPayload) -> Self {
        self.form_fields.reserve(payload.fields().len());
        for (key, value) in payload.fields() {
            self.form_fields
                .push(FormField::new(key.clone(), value.clone()));
        }
        self
    }

    #[must_use]
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseParts {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

impl ResponseParts {
    pub fn new(status: u16, headers: Vec<Header>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }
}

pub(crate) fn default_headers(context: &ClientContext) -> Vec<Header> {
    vec![
        Header::new("Accept", "*/*"),
        Header::new("Accept-Language", context.profile.accept_language.clone()),
        Header::new("Accept-Encoding", context.profile.accept_encoding.clone()),
        Header::new("Content-Type", "application/x-www-form-urlencoded"),
        Header::new("User-Agent", context.profile.user_agent.clone()),
        Header::new("X-Img-Version", context.profile.image_version.clone()),
        Header::new("X-Locale", context.profile.locale.clone()),
        Header::new("X-Client", context.profile.x_client_value()),
        Header::new("X-Hour", context.profile.hour_format.clone()),
        Header::new("X-TimezoneId", context.timezone_id.clone()),
        Header::new("X-Api-Version", context.profile.api_version.clone()),
        Header::new("X-Timezone", context.timezone_offset_hours.clone()),
    ]
}

pub fn session_headers(session_id: &str) -> Vec<Header> {
    vec![
        Header::new("F-Session", session_id),
        Header::new("X-Session", session_id),
    ]
}
