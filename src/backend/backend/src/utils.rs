mod header;
pub use header::HeaderValueExt;
pub mod base64_utils {
    use base64::{engine::general_purpose::STANDARD, DecodeError, Engine};
    use tracing::instrument;
    #[instrument(skip(input), name = "base64_utils::decode")]
    #[inline(always)]
    pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
        STANDARD.decode(input)
    }

    #[inline(always)]
    pub fn encode(input: impl AsRef<[u8]>) -> String {
        STANDARD.encode(input)
    }
    #[inline(always)]
    pub fn encode_basic_header(username: impl AsRef<str>, password: impl AsRef<str>) -> String {
        STANDARD.encode(format!("{}:{}", username.as_ref(), password.as_ref()))
    }
    pub mod serde_base64 {
        use serde::{Deserialize, Serialize};

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;
            super::decode(string).map_err(serde::de::Error::custom)
        }
        pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            super::encode(data).serialize(serializer)
        }
    }
}
