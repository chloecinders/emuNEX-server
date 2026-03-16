use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sqlx::{Database, Decode, Encode, Postgres, Type, encode::IsNull, error::BoxDynError, postgres::{PgTypeInfo, PgValueRef}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub i64);

impl Id {
    pub fn new(v: i64) -> Self {
        Self(v)
    }

    pub fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Id {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Id(s.parse()?))
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct IdVisitor;
        impl<'de> de::Visitor<'de> for IdVisitor {
            type Value = Id;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a string or integer snowflake ID")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Id, E> {
                v.parse::<i64>().map(Id).map_err(de::Error::custom)
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Id, E> {
                Ok(Id(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Id, E> {
                Ok(Id(v as i64))
            }
        }
        d.deserialize_any(IdVisitor)
    }
}

impl Type<Postgres> for Id {
    fn type_info() -> PgTypeInfo {
        <i64 as Type<Postgres>>::type_info()
    }
}

impl<'r> Decode<'r, Postgres> for Id {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let v = <i64 as Decode<Postgres>>::decode(value)?;
        Ok(Id(v))
    }
}

impl<'q> Encode<'q, Postgres> for Id {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError> {
        <i64 as Encode<Postgres>>::encode_by_ref(&self.0, buf)
    }
}

impl From<i64> for Id {
    fn from(v: i64) -> Self {
        Self(v)
    }
}
