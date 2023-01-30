use std::fmt;
use std::str::FromStr;

use super::error::ParseError;


pub struct StatusCode (pub i32);
impl StatusCode {
    pub fn name(&self) -> &'static str {
        match self {
            StatusCode(101) => "Switching Protocols",
            StatusCode(200..=299) => "OK",
            _ => "",
        }
    }
}
impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for StatusCode{
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<StatusCode, ParseError> {
        Ok(StatusCode(s.parse::<i32>()?))
    }
}