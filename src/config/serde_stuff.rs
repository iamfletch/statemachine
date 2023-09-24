use std::fmt::Formatter;
use log::LevelFilter;
use serde::{de, Serializer};

pub fn deserialize_level_filter<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error> where D: de::Deserializer<'de> {
    struct LevelFilterVisitor;

    impl<'de> de::Visitor<'de> for LevelFilterVisitor {
        type Value = LevelFilter;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing valid values")
        }

        fn visit_str<E>(self, input: &str) -> Result<Self::Value, E> where E: de::Error {
            match input {
                "off" => Ok(LevelFilter::Off),
                "error" => Ok(LevelFilter::Error),
                "warn" => Ok(LevelFilter::Warn),
                "info" => Ok(LevelFilter::Info),
                "debug" => Ok(LevelFilter::Debug),
                "trace" => Ok(LevelFilter::Trace),
                _ => Err(de::Error::invalid_value(de::Unexpected::Str(input), &self))
            }
        }
    }

    deserializer.deserialize_any(LevelFilterVisitor)
}

pub fn serialize_level_filter<S>(level_filter: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(
        match level_filter {
            LevelFilter::Off => "off",
            LevelFilter::Error => "error",
            LevelFilter::Warn => "warn",
            LevelFilter::Info => "info",
            LevelFilter::Debug => "debug",
            LevelFilter::Trace => "trace"
        }
    )
}