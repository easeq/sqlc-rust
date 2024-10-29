use cfg_block::cfg_block;
use std::collections::HashMap;

#[cfg(feature = "with-postgres")]
use postgres::{Error as PgError, Row};

#[cfg(feature = "with-tokio-postgres")]
use tokio_postgres::{Error as PgError, Row};

pub trait FromPostgresRow: Sized {
    fn from_row(row: &Row) -> Result<Self, PgError>;
}

macro_rules! from_primitive {
    ($t:ty) => {
        impl FromPostgresRow for $t {
            fn from_row(row: &Row) -> Result<Self, PgError> {
                Ok(row.try_get::<&str, $t>("0")?)
            }
        }
    };
}

from_primitive!(bool);
from_primitive!(String);
from_primitive!(i16);
from_primitive!(i32);
from_primitive!(i64);
from_primitive!(f64);
from_primitive!(HashMap<String, Option<String>>);

#[cfg(feature = "with-bit-vec-0_6")]
from_primitive!(bit_vec_06::BitVec);

#[cfg(feature = "with-uuid-0_8")]
from_primitive!(uuid_0_8::Uuid);

#[cfg(feature = "with-uuid-1")]
from_primitive!(uuid_1::Uuid);

#[cfg(feature = "with-eui48-0_4")]
from_primitive!(eui48_04::MacAddress);

#[cfg(feature = "with-eui48-1")]
from_primitive!(eui48_1::MacAddress);

#[cfg(feature = "with-serde_json-1")]
from_primitive!(serde_json_1::Value);

cfg_block! {
    #[cfg(feature = "with-cidr-0_2")] {
        from_primitive!(cidr_02::IpInet);
        from_primitive!(cidr_02::IpCidr);
    }

    #[cfg(feature = "with-geo-types-0_6")] {
        from_primitive!(geo_types_06::Point);
        from_primitive!(geo_types_06::Rect);
        from_primitive!(geo_types_06::LineString);
    }

    #[cfg(feature = "with-geo-types-0_7")] {
        from_primitive!(geo_types_0_7::Point);
        from_primitive!(geo_types_0_7::Rect);
        from_primitive!(geo_types_0_7::LineString);
    }

    #[cfg(feature = "with-time-0_2")] {
        from_primitive!(time_02::Time);
        from_primitive!(time_02::Date);
        from_primitive!(time_02::PrimitiveDateTime);
        from_primitive!(time_02::OffsetDateTime);
    }

    #[cfg(feature = "with-time-0_3")] {
        from_primitive!(time_03::Time);
        from_primitive!(time_03::Date);
        from_primitive!(time_03::PrimitiveDateTime);
        from_primitive!(time_03::OffsetDateTime);
    }
}
