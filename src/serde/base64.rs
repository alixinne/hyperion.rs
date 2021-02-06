//! Base64 deserialization of Vec<u8>

use std::fmt;

/// Serde visitor for deserializing Base64-encoded values
struct Base64Visitor;

impl<'a> serde::de::Visitor<'a> for Base64Visitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("base64 image")
    }

    fn visit_str<A>(self, string: &str) -> Result<Self::Value, A>
    where
        A: serde::de::Error,
    {
        base64::decode(string).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

/// Decode a base64-encoded value
///
/// # Parameters
///
/// `deserializer`: Serde deserializer
pub fn from_base64<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(Base64Visitor {})
}
