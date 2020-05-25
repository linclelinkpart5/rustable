
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
    ( $( ($type:ty, $s_name:ident, $l_name:ident $( , $cfg_flag:meta )?), )+ ) => {
        /// Represents all data types supported by `rustable`.
        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        pub enum DType {
            $( $(#[$cfg_flag])? $l_name ),*
        }

        /// Provides references to elements within a potentially heterogeneous
        /// row for data.
        #[derive(PartialEq)]
        pub enum Datum<'a> {
            $( $(#[$cfg_flag])? $s_name(&'a $type) ),*
        }

        /// An enum representation of a `Series`, typically only seen when
        /// trying to get a reference to a column from a `Frame` without knowing
        /// its type beforehand.
        pub enum Column {

        }
    };
}

define_types!(

    (i8, I8, I8),
    (i16, I16, I16),
    (i32, I32, I32),
    (i64, I64, I64),
    (isize, ISIZE, ISIZE),
    (i128, I128, I128, cfg(feature = "128")),

    (u8, U8, U8),
    (u16, U16, U16),
    (u32, U32, U32),
    (u64, U64, U64),
    (usize, USIZE, USIZE),
    (u128, U128, U128, cfg(feature = "128")),

    (f32, F32, F32),
    (f64, F64, F64),

    (char, CHAR, CHAR),
    (bool, BOOL, BOOL),

    (String, STRING, STRING),

    (Decimal, DECIMAL, DECIMAL, cfg(feature = "decimal")),

    (Date, DATE, DATE, cfg(feature = "date-time")),
    (Time, TIME, TIME, cfg(feature = "date-time")),
    (DateTime, DATETIME, DATETIME, cfg(feature = "date-time")),
);
