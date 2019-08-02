//! Duration type serialization

use std::fmt;
use std::time::Duration;

/// Serde visitor for deserializing durations
struct DurationVisitor;

impl<'a> serde::de::Visitor<'a> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("duration")
    }

    fn visit_str<A>(self, string: &str) -> Result<Self::Value, A>
    where
        A: serde::de::Error,
    {
        string
            .parse::<humantime::Duration>()
            .map(std::convert::Into::<Duration>::into)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

/// Parse a duration from a string
///
/// # Parameters
///
/// `deserializer`: Serde deserializer
pub fn hyperion_parse_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(DurationVisitor {})
}

/// Serialize a duration to a string
///
/// # Parameters
///
/// * `duration`: duration to serialize
/// * `serializer`: Serde serializer
pub fn hyperion_write_duration<S>(
    duration: &Duration,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{}", humantime::Duration::from(*duration)))
}

