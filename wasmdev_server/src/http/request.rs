use std::fmt;
use std::str::FromStr;
use std::slice::Iter;

use super::{Version, Header, write_headers};
use super::error::ParseError;


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
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<RequestType, ParseError> {
        match s {
            "GET" => Ok(RequestType::GET),
            "PUT" => Ok(RequestType::PUT),
            "POST" => Ok(RequestType::POST),
            "DELETE" => Ok(RequestType::DELETE),
            _ => Err(ParseError)
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
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Request, ParseError> {(|| {
        let mut lines = s.split("\r\n");
        let mut words = lines.next()?.split(" ");
        let request_type: RequestType = words.next()?.parse().ok()?;
        let path                      = words.next()?.to_string();
        let version: Version          = words.next()?.parse().ok()?;
        let headers: Vec<Header> = lines
            .filter_map(|s| s.parse::<Header>().ok())
            .collect();
        Some(Request {
            request_type,
            path,
            version,
            headers,
        })
    })().ok_or(ParseError)}
}