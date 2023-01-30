use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, Clone)]
pub struct ParseError;
impl fmt::Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse HTTP message")
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_value: ParseIntError) -> Self {
        ParseError
    }
}