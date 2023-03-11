use std::fmt;
use std::num::ParseIntError;
use std::str::Utf8Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum ParseErrorKind{
    Utf8Error(Utf8Error),
    IntError(ParseIntError),
    FormatError(String),
    IncompleteError,
}

impl fmt::Display for ParseErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Utf8Error(err) => write!(f, "utf8 error, {}", err),
            Self::IntError(err) => write!(f, "parse int error, {}", err),
            Self::FormatError(msg) => write!(f, "format error, {}", msg),
            Self::IncompleteError => write!(f, "incomplete"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HttpErrorKind{
    ParseError(ParseErrorKind),
    IncompleteReqError(String),
    UnsupportedReqTypeError,
    UnsupportedVersionError,
}

impl fmt::Display for HttpErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::ParseError(kind) => 
                write!(f, "Unable to parse HTTP message: {kind}"),
            Self::IncompleteReqError(msg) => 
                write!(f, "Request is incomplete: {msg}"),
            Self::UnsupportedVersionError =>
                write!(f, "Unsupported http version. Only v 1.0 and 1.1 is supported"),
            Self::UnsupportedReqTypeError =>
                write!(f, "Unsupported requst type. GET, POST, PUT and DELETE is supported."),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BufferErrorKind{
    EmptyBuffer,
}

impl fmt::Display for BufferErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::EmptyBuffer => write!(f, "Buffer is empty"),
        }
    }
}


#[derive(Debug, Clone)]
pub enum Error{
    IOError(std::io::ErrorKind),
    HttpError(HttpErrorKind),
    BufferError(BufferErrorKind),
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::IOError(error_kind) => error_kind.fmt(f),
            Self::HttpError(error_kind) => error_kind.fmt(f),
            Self::BufferError(error_kind) => error_kind.fmt(f),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::HttpError(HttpErrorKind::ParseError(ParseErrorKind::IntError(err)))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value.kind())
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Error::HttpError(HttpErrorKind::ParseError(ParseErrorKind::Utf8Error(value)))
    }
}

impl From<ParseErrorKind> for Error {
    fn from(value: ParseErrorKind) -> Self {
        Error::HttpError(HttpErrorKind::ParseError(value))
    }
}

impl From<BufferErrorKind> for Error {
    fn from(value: BufferErrorKind) -> Self {
        Error::BufferError(value)
    }
}

impl From<HttpErrorKind> for Error {
    fn from(value: HttpErrorKind) -> Self {
        Error::HttpError(value)
    }
}

impl Error {
    pub fn format_error(msg: impl Into<String>) -> Self {
        Error::HttpError(HttpErrorKind::ParseError(ParseErrorKind::FormatError(msg.into())))
    }
    pub fn incomplete_req_error(msg: impl Into<String>) -> Self {
        Error::HttpError(HttpErrorKind::IncompleteReqError(msg.into()))
    }
}
