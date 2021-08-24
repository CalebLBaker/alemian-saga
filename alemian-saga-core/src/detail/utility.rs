use crate::*;

// Error message type
pub struct Error {
    pub msg: String,
}

// Conversion into error type
impl<E: std::string::ToString> From<E> for Error {
    fn from(err: E) -> Error {
        Error {
            msg: err.to_string(),
        }
    }
}

pub fn multiply_frac<T: Scalar + From<u32>>(x: T, num: u32, den: u32) -> T {
    x * num.into() / den.into()
}

pub fn partial_ord_min<T: std::cmp::PartialOrd>(a: T, b: T) -> T {
    if b < a {
        b
    } else {
        a
    }
}

pub fn get_class_name(class: serialization::Class) -> &'static str {
    match class {
        serialization::Class::Noble => "noble",
    }
}
