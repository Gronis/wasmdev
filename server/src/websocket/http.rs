use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

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
// impl FromStr for StatusCode{
//     type Err = ParseIntError;
//     #[inline]
//     fn from_str(s: &str) -> Result<StatusCode, ParseIntError> {
//         Ok(StatusCode(s.parse::<i32>()?))
//     }
// }

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

#[derive(PartialEq)]
pub enum Header{
    Host(String),
    Connection(String),
    Upgrade(String),
    SecWebSocketKey(String),
    SecWebSocketVersion(i32),
    SecWebSocketExtensions(String),
    SecWebSocketAccept(String),
    ContentLength(usize),
}
impl Header{
    pub fn host (s: &str) -> Header { Header::Host(s.to_string()) }
    pub fn connection (s: &str) -> Header { Header::Connection(s.to_string()) }
    pub fn upgrade (s: &str) -> Header { Header::Upgrade(s.to_string()) }
    pub fn sec_websocket_key (s: &str) -> Header { Header::SecWebSocketKey(s.to_string()) }
    pub fn sec_websocket_version (i: i32) -> Header { Header::SecWebSocketVersion(i) }
    pub fn sec_websocket_extensions (s: &str) -> Header { Header::SecWebSocketExtensions(s.to_string()) }
    pub fn sec_websocket_accept (s: &str) -> Header { Header::SecWebSocketAccept(s.to_string()) }
    pub fn content_length (s: &str) -> Result<Header, ParseIntError> { Ok(Header::ContentLength(s.parse()?))}
}
impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Header::Host(s) => write!(f, "Host: {}", s),
            Header::Connection(s) => write!(f, "Connection: {}", s),
            Header::Upgrade(s) => write!(f, "Upgrade: {}", s),
            Header::SecWebSocketKey(s) => write!(f, "Sec-WebSocket-Key: {}", s),
            Header::SecWebSocketVersion(s) => write!(f, "Sec-WebSocket-Version: {}", s),
            Header::SecWebSocketExtensions(s) => write!(f, "Sec-WebSocket-Extensions: {}", s),
            Header::SecWebSocketAccept(s) => write!(f, "Sec-WebSocket-Accept: {}", s),
            Header::ContentLength(s) => write!(f, "Content-Length: {}", s),
        }
    }
}

impl FromStr for Header{
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Header, ParseError> {
        let (header, value) = s.split_at(s.find(':').ok_or(ParseError)?);
        let value = &value[1..].trim();
        let parse_version = |v : &str | v.parse::<i32>().or(Err(ParseError));
        match header {
            "Host" => Ok(Header::host(value)),
            "Connection" => Ok(Header::connection(value)),
            "Upgrade" => Ok(Header::upgrade(value)),
            "Sec-WebSocket-Key" => Ok(Header::sec_websocket_key(value)),
            "Sec-WebSocket-Version" => Ok(Header::sec_websocket_version(parse_version(value)?)),
            "Sec-WebSocket-Extensions" => Ok(Header::sec_websocket_extensions(value)),
            "Sec-WebSocket-Accept" => Ok(Header::sec_websocket_accept(value)),
            "Content-Length" => Ok(Header::content_length(value)?),
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
    pub fn match_header(&self, f: fn(&Header) -> bool) -> bool {
        self.headers.iter().any(f)
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

pub struct Response {
    pub version: Version,
    pub status_code: StatusCode,
    pub headers: Vec<Header>,
}
impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}\r\n", self.version, self.status_code, self.status_code.name())?;
        write_headers(f, &self.headers)?;
        write!(f, "\r\n")
    }
}

fn write_headers(f: &mut fmt::Formatter, headers: &Vec<Header>) -> fmt::Result {
    if headers.len() == 0 { 
        write!(f, "")
    } else {
        headers.iter().try_fold((), |_, header| { 
            write!(f, "{}", header)?; 
            write!(f, "\r\n")
        })
    }
}