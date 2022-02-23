use std::fmt;

/// Serde visitor for deserializing string values which might
/// be literally a "NONE"-string
struct ConfigOptionalVisitor;

impl<'a> serde::de::Visitor<'a> for ConfigOptionalVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A useful type-string or \"NONE\"-string")
    }

    fn visit_str<A>(self, string: &str) -> Result<Self::Value, A>
    where
        A: serde::de::Error,
    {
        let value = match string.to_lowercase().as_str() {
            "none" => None,
            _ => Some(string.to_owned()),
        };

        Ok(value)
    }
}

/// Decode a ConfigOptional value
///
/// # Parameters
///
/// `deserializer`: Serde deserializer
pub fn from_config_optional<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(ConfigOptionalVisitor {})
}
