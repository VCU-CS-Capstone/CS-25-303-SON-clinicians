use derive_more::derive::{From, Into};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(default)]
pub struct LookupPage {
    pub page_size: i32,
    pub page_number: i32,
}
impl Default for LookupPage {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}
pub fn sanitize_string(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
pub mod serde_sanitize_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(super::sanitize_string(s))
    }

    pub fn serialize<S>(s: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match s {
            Some(s) => serializer.serialize_str(s),
            None => serializer.serialize_none(),
        }
    }
}
