
// Import and re-export external types for easier downstream usage.
#[cfg(feature = "decimal")]
pub use rust_decimal::Decimal;
#[cfg(feature = "date-time")]
pub use chrono::naive::{
    NaiveDate as Date,
    NaiveTime as Time,
    NaiveDateTime as DateTime,
};

/// Helper macro to create the plumbing for each type supported in `rustable`.
macro_rules! define_types {
    ( $( ($type:ty, $name:ident $( , $cfg_flag:meta )?), )+ ) => {
        paste::item! {
            /// Represents all data types supported by `rustable`.
            #[derive(Debug, PartialEq, Eq, Copy, Clone)]
            pub enum DType {
                $(
                    $(#[$cfg_flag])? $name,
                    $(#[$cfg_flag])? [<Opt $name>],
                )*
            }

            /// Provides references to elements within a potentially
            /// heterogeneous row of data.
            #[derive(PartialEq)]
            pub enum Datum<'a> {
                $(
                    $(#[$cfg_flag])? $name(&'a $type),
                    $(#[$cfg_flag])? [<Opt $name>](&'a Option<$type>),
                )*
            }

            /// An enum representation of a `Series`, typically only seen when
            /// trying to get a reference to a column from a `Frame` without
            /// knowing its type beforehand.
            pub enum Column {

            }
        }
    };
}

define_types!(
    (i8, I8),
    (i16, I16),
    (i32, I32),
    (i64, I64),
    (isize, ISize),
    (i128, I128, cfg(feature = "128")),

    (u8, U8),
    (u16, U16),
    (u32, U32),
    (u64, U64),
    (usize, USize),
    (u128, U128, cfg(feature = "128")),

    (f32, F32),
    (f64, F64),

    (char, Char),
    (bool, Bool),

    (String, Str),

    (Decimal, Decimal, cfg(feature = "decimal")),

    (Date, Date, cfg(feature = "date-time")),
    (Time, Time, cfg(feature = "date-time")),
    (DateTime, DateTime, cfg(feature = "date-time")),
);
