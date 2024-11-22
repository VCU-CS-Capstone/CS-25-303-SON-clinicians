use serde::Deserialize;
#[derive(Debug)]
pub struct RedCapValue(pub String);
mod _serde_red_cap_value {
    use serde::de::Visitor;

    use super::RedCapValue;

    pub struct RedCapValueVisitor;
    impl<'de> Visitor<'de> for RedCapValueVisitor {
        type Value = super::RedCapValue;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string")
        }
        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RedCapValue(v.to_string()))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RedCapValue(v.to_string()))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RedCapValue(v.to_string()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RedCapValue(v))
        }
        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RedCapValue(v.to_string()))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RedCapValue(v.to_string()))
        }
    }
    impl<'de> serde::de::Deserialize<'de> for super::RedCapValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_any(RedCapValueVisitor)
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(deserialize_with = "fix_red_cap_bad_data_format::deserialize_number")]
    pub record: usize,
    #[serde(
        deserialize_with = "fix_red_cap_bad_data_format::deserialize_string_but_none_is_empty"
    )]
    pub redcap_repeat_instrument: Option<String>,
    #[serde(deserialize_with = "fix_red_cap_bad_data_format::deserialize_number_option")]
    pub redcap_repeat_instance: Option<usize>,
    pub field_name: String,
    pub value: String,
}
pub mod fix_red_cap_bad_data_format {
    use serde::{de::Visitor, Deserialize, Deserializer};

    pub struct ValueVisitor;
    impl<'de> Visitor<'de> for ValueVisitor {
        type Value = Option<usize>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or an integer")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.is_empty() {
                return Ok(None);
            }
            Ok(Some(v.parse().map_err(serde::de::Error::custom)?))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.is_empty() {
                return Ok(None);
            }
            Ok(Some(v.parse().map_err(serde::de::Error::custom)?))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v as usize))
        }
    }

    pub fn deserialize_number_option<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }

    pub fn deserialize_number<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_any(ValueVisitor)?
            .ok_or_else(|| serde::de::Error::custom("Expected a value"))
    }
    pub fn deserialize_string_but_none_is_empty<'de, D>(
        deserializer: D,
    ) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}
