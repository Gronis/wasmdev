use std::io::{BufWriter, Read};
use std::io::{BufRead, BufReader, Write};
use std::str::from_utf8;
use sha1::{Sha1, Digest};
use base64::{Engine, DecodeError};

use super::{Request, Response, Header, Version, StatusCode};


pub fn is_valid_websocket(request: &Request) -> bool {
    request.headers().any(|h| matches!(h, Header::Connection(header) if header == "Upgrade")) && 
    request.headers().any(|h| matches!(h, Header::SecWebSocketVersion(13))) && 
    request.headers().any(|h| matches!(h, Header::SecWebSocketKey(_)))
}

pub fn compute_accept(websocket_key_header: &str) -> Result<String, DecodeError> {
    let mut hasher = Sha1::new();
    hasher.update(&websocket_key_header);
    hasher.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let hash_b64 = base64::engine::general_purpose::STANDARD.encode(hasher.finalize()); 
    Ok(hash_b64)
}

pub fn make_websocket_accept_response(request: &Request) -> Result<Response, String> {
    request.headers
        .iter()
        .find_map(|header| {
            let Header::SecWebSocketKey(key) = header else { return None };
            let Ok(accept) = compute_accept(key) else { return None };
            Some(Response {
                version: Version::V1_1,
                status_code: StatusCode(101), 
                headers: vec![ 
                    Header::upgrade("websocket"),
                    Header::connection("Upgrade"),
                    Header::SecWebSocketAccept(accept),
                ],
                body: None,
            })
        }).ok_or("Unable to create websocket upgrade response from request".into())
}

pub fn make_http_response(status_code: StatusCode, headers: Vec<Header>, body: Option<&Vec<u8>>) -> Response {
    Response {
        version: Version::V1_1,
        status_code, 
        headers,
        body,
    }
}

pub fn parse_request<T: Read>(reader: &mut BufReader<T>) -> Result<Request, String>{
    reader.fill_buf().map_err(|_| "Unable to read data from buffer")?;
    if reader.buffer().is_empty() { return Err("Stream is closed (empty buffer)".into()) };
    let Some(end_index) = reader.buffer()
        .windows(4)
        .enumerate()
        .find_map(|(i, range)| {
            let [a, b, c, d] = range else { return None };
            if *a == b'\r' && *b == b'\n' && *c == b'\r' && *d == b'\n' { 
                Some(i) // Dont include end-of-message \r\n\r\n
            } else { None }
        }) else { return Err("Message in buffer is incomplete".into()) };
        
    let msg = from_utf8(&reader.buffer()[0..end_index]).map_err(|_| "Unable to parse utf8 string")?;
    let request: Request = msg.parse().map_err(|_| "Unable to parse http request")?;
    reader.consume(end_index + 4); // Consume end-of-message
    Ok(request)
}

pub fn write_response<T: Write>(writer: &mut BufWriter<T>, response: &Response) -> Result<(), String> {
    writer.write_all(response.to_string().as_bytes()).map_err(|_| "Unable to write response header to stream")?;
    if let Some(body) = &response.body {
        writer.write_all(body).map_err(|_| "Unable to write response body to stream")?;
        writer.write_all("\r\n".as_bytes()).map_err(|_| "Unable to write response body to stream")?;
    }
    writer.flush().map_err(|_| "Unable to flush stream".into())
}

