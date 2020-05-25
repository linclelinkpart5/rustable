/// Represents all data types supported by `rustable`.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DType {
    I8, I16, I32, I64, ISIZE,
    U8, U16, U32, U64, USIZE,
    F32, F64,
    CHAR,
    BOOL,
    // I128, U128,
    // STRING,
    // DECIMAL,
    // DATE, TIME, DATETIME,
}
