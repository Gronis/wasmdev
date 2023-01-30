use std::fmt;
use std::str::FromStr;

use super::error::ParseError;

pub enum Version{ V1_0, V1_1 }
impl fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::V1_0 => write!(f, "HTTP/1.0"),
            Self::V1_1 => write!(f, "HTTP/1.1"),
        }
    }
}
impl FromStr for Version{
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Version, ParseError> {
        match s {
            "HTTP/1.0" => Ok(Version::V1_0),
            "HTTP/1.1" => Ok(Version::V1_1),
            _ => Err(ParseError)
        }
    }
}
