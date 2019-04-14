use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer, Visitor};

use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum ElasticSearchServerType {
    Elastic,
    Kibana
}

impl Default for ElasticSearchServerType {
    fn default() -> Self {
        ElasticSearchServerType::Elastic
    }
}

struct ElasticSearchServerTypeVisitor;

impl Serialize for ElasticSearchServerType {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
       match self {
           ElasticSearchServerType::Elastic => s.serialize_str("elastic"),
           ElasticSearchServerType::Kibana => s.serialize_str("kibana")
       }
    }
}

impl<'de> Deserialize<'de> for ElasticSearchServerType {
    fn deserialize<D>(d: D) -> Result<ElasticSearchServerType, D::Error> where D: Deserializer<'de> {
        d.deserialize_str(ElasticSearchServerTypeVisitor)
    }
}

impl<'de> Visitor<'de> for ElasticSearchServerTypeVisitor {
    type Value = ElasticSearchServerType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("'elastic' or 'kibana'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: de::Error {
        ElasticSearchServerType::from_str(v)
            .map_err(|err| E::custom(format!("{}", err)) )
    }
}

#[derive(Debug, Fail)]
#[fail(display = "unknown server type: {}", value)]
pub struct UnknownElasticSearchServerTypeError {
    value: String
}

impl FromStr for ElasticSearchServerType {
    type Err = UnknownElasticSearchServerTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "elastic" => Ok(ElasticSearchServerType::Elastic),
            "kibana" => Ok(ElasticSearchServerType::Kibana),
            value => Err(UnknownElasticSearchServerTypeError { value: value.to_string() })
        }
    }
}