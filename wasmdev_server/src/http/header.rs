use std::fmt;
use std::str::FromStr;

use super::error::*;

#[derive(Clone, PartialEq)]
pub enum Header{
    Host(String),
    Connection(String),
    Upgrade(String),
    SecWebSocketKey(String),
    SecWebSocketVersion(i32),
    SecWebSocketExtensions(String),
    SecWebSocketAccept(String),
    ContentLength(usize),
    ContentType(String),
    Unsupported,
}
impl Header{
    pub fn host (s: &str) -> Header { Header::Host(s.into()) }
    pub fn connection (s: &str) -> Header { Header::Connection(s.into()) }
    pub fn upgrade (s: &str) -> Header { Header::Upgrade(s.into()) }
    pub fn sec_websocket_key (s: &str) -> Header { Header::SecWebSocketKey(s.into()) }
    pub fn sec_websocket_version (i: i32) -> Header { Header::SecWebSocketVersion(i) }
    pub fn sec_websocket_extensions (s: &str) -> Header { Header::SecWebSocketExtensions(s.into()) }
    pub fn sec_websocket_accept (s: &str) -> Header { Header::SecWebSocketAccept(s.into()) }
    pub fn content_length (s: &str) -> Result<Header> { Ok(Header::ContentLength(s.parse()?))}
    pub fn content_type (s: &str) -> Header { Header::ContentType(s.into())}
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
            Header::ContentType(s) => write!(f, "Content-Type: {}", s),
            Header::Unsupported => write!(f, "Unsupported Header"),
        }
    }
}

impl FromStr for Header{
    type Err = Error;
    #[inline]
    fn from_str(s: &str) -> Result<Header> {
        let (header, value) = s.split_at(s.find(':')
            .ok_or(Error::format_error(format!("Unable to parse header '{s}'")))?);
        let value = &value[1..].trim();
        let parse_version = |v : &str | v.parse::<i32>();
        match header.to_ascii_lowercase().as_str() {
            "host" => Ok(Header::host(value)),
            "connection" => Ok(Header::connection(value)),
            "upgrade" => Ok(Header::upgrade(value)),
            "sec-websocket-key" => Ok(Header::sec_websocket_key(value)),
            "sec-websocket-version" => Ok(Header::sec_websocket_version(parse_version(value)?)),
            "sec-websocket-extensions" => Ok(Header::sec_websocket_extensions(value)),
            "sec-websocket-accept" => Ok(Header::sec_websocket_accept(value)),
            "content-length" => Ok(Header::content_length(value)?),
            "content-type" => Ok(Header::content_type(value)),
            _ => Ok(Header::Unsupported)
        }
    }
}

pub fn write_headers(f: &mut fmt::Formatter, headers: &Vec<Header>) -> fmt::Result {
    if headers.len() == 0 { 
        write!(f, "")
    } else {
        headers.iter().try_fold((), |_, header| { 
            write!(f, "{}", header)?; 
            write!(f, "\r\n")
        })
    }
}