use std::fmt;
use std::str::FromStr;
use std::slice::Iter;

use super::{Version, Header, write_headers};
use super::error::*;


pub enum RequestType{ GET, PUT, POST, DELETE }
impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GET => write!(f, "GET"),
            Self::PUT => write!(f, "PUT"),
            Self::POST => write!(f, "POST"),
            Self::DELETE => write!(f, "DELETE"),
        }
    }
}
impl FromStr for RequestType{
    type Err = Error;
    #[inline]
    fn from_str(s: &str) -> Result<RequestType> {
        match s.to_ascii_uppercase().as_str() {
            "GET" => Ok(RequestType::GET),
            "PUT" => Ok(RequestType::PUT),
            "POST" => Ok(RequestType::POST),
            "DELETE" => Ok(RequestType::DELETE),
            _ => Err(HttpErrorKind::UnsupportedReqTypeError.into())
        }
    }
}


pub struct Request {
    pub request_type: RequestType, 
    pub path: String,
    pub version: Version,
    pub headers: Vec<Header>,
}
impl Request {
    pub fn headers(&self) -> Iter<Header> {
        self.headers.iter()
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}\r\n", self.request_type, self.path, self.version)?;
        write_headers(f, &self.headers)?;
        write!(f, "\r\n")
    }
}

impl FromStr for Request{
    type Err = Error;
    #[inline]
    fn from_str(s: &str) -> Result<Request> {(|| {
        let mut lines = s.split("\r\n");
        let mut words = lines.next()?.split(" ");
        let request_type: RequestType = words.next()?.parse().ok()?;
        let path                      = words.next()?.to_string();
        let version: Version          = words.next()?.parse().ok()?;
        let headers: Vec<Header> = lines
            // FIXME: If header fails to parse here, maybe the whole request should fail.
            .filter_map(|s| s.parse::<Header>().ok())
            .collect();
        Some(Request {
            request_type,
            path,
            version,
            headers,
        })
    })().ok_or(Error::format_error(format!("unable to parse request: '{s}'")))}
}