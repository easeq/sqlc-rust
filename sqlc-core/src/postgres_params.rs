use cfg_block::cfg_block;
use std::collections::HashMap;

pub trait PostgresParams {
    fn as_params(&self) -> Vec<&(dyn postgres_types::ToSql + Sync)>;
}

impl<T> PostgresParams for &T
where
    T: PostgresParams,
{
    fn as_params(&self) -> Vec<&(dyn postgres_types::ToSql + Sync)> {
        return (**self).as_params();
    }
}

macro_rules! as_params {
    ($t:ty) => {
        impl PostgresParams for $t {
            fn as_params(&self) -> Vec<&(dyn postgres_types::ToSql + Sync)> {
                vec![self]
            }
        }
    };
}

impl PostgresParams for () {
    fn as_params(&self) -> Vec<&(dyn postgres_types::ToSql + Sync)> {
        vec![]
    }
}

as_params!(bool);
as_params!(String);
as_params!(i16);
as_params!(i32);
as_params!(i64);
as_params!(f64);
as_params!(HashMap<String, Option<String>>);

#[cfg(feature = "with-bit-vec-0_6")]
as_params!(bit_vec_06::BitVec);

#[cfg(feature = "with-uuid-0_8")]
as_params!(uuid_0_8::Uuid);

#[cfg(feature = "with-uuid-1")]
as_params!(uuid_1::Uuid);

#[cfg(feature = "with-eui48-0_4")]
as_params!(eui48_04::MacAddress);

#[cfg(feature = "with-eui48-1")]
as_params!(eui48_1::MacAddress);

#[cfg(feature = "with-serde_json-1")]
as_params!(serde_json_1::Value);

#[cfg(feature = "with-rust_decimal-postgres")]
as_params!(rust_decimal_postgres::prelude::Decimal);

#[cfg(feature = "with-rust_decimal-tokio-postgres")]
as_params!(rust_decimal_tokio_postgres::prelude::Decimal);

cfg_block! {
    #[cfg(feature = "with-cidr-0_2")] {
        pub use cidr_02::{IpInet, IpCidr};
        as_params!(cidr_02::IpInet);
        as_params!(cidr_02::IpCidr);
    }

    #[cfg(feature = "with-geo-types-0_6")] {
        as_params!(geo_types_06::Point);
        as_params!(geo_types_06::Rect);
        as_params!(geo_types_06::LineString);
    }

    #[cfg(feature = "with-geo-types-0_7")] {
        as_params!(geo_types_0_7::Point);
        as_params!(geo_types_0_7::Rect);
        as_params!(geo_types_0_7::LineString);
    }

    #[cfg(feature = "with-time-0_2")] {
        as_params!(time_02::Time);
        as_params!(time_02::Date);
        as_params!(time_02::PrimitiveDateTime);
        as_params!(time_02::OffsetDateTime);
    }

    #[cfg(feature = "with-time-0_3")] {
        as_params!(time_03::Time);
        as_params!(time_03::Date);
        as_params!(time_03::PrimitiveDateTime);
        as_params!(time_03::OffsetDateTime);
    }
}
