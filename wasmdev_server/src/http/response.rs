use std::fmt;

use super::{Version, StatusCode, Header, write_headers};


pub struct Response<'a> {
    pub version: Version,
    pub status_code: StatusCode,
    pub headers: Vec<Header>,
    pub body: Option<&'a Vec<u8>>,
}
impl <'a> fmt::Display for Response<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}\r\n", self.version, self.status_code, self.status_code.name())?;
        write_headers(f, &self.headers)?;
        write!(f, "\r\n")
    }
}
