use std::net::{TcpListener, TcpStream};
use std::io::{self, BufWriter};
use std::io::{BufRead, BufReader, Write};
use std::str::from_utf8;
use std::thread;

use sha1::{Sha1, Digest};

use base64::{Engine, DecodeError};

use self::http::{Request, Response};

mod http;

fn is_valid_websocket(request: &http::Request) -> bool {
    request.match_header(|h| matches!(h, http::Header::Connection(header) if header == "Upgrade")) && 
    request.match_header(|h| matches!(h, http::Header::Upgrade(header) if header == "websocket")) && 
    request.match_header(|h| matches!(h, http::Header::SecWebSocketVersion(13))) && 
    request.match_header(|h| matches!(h, http::Header::SecWebSocketKey(_)))
}

fn compute_accept(websocket_key_header: &str) -> Result<String, DecodeError> {
    let mut hasher = Sha1::new();
    hasher.update(&websocket_key_header);
    hasher.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let hash_b64 = base64::engine::general_purpose::STANDARD.encode(hasher.finalize()); 
    Ok(hash_b64)
}

fn make_websocket_accept_response(request: &http::Request) -> Result<http::Response, String> {
    request.headers
        .iter()
        .find_map(|header| {
            let http::Header::SecWebSocketKey(key) = header else { return None };
            let Ok(accept) = compute_accept(key) else { return None };
            Some(http::Response {
                version: http::Version::V1_1,
                status_code: http::StatusCode(101), 
                headers: vec![ 
                    http::Header::upgrade("websocket"),
                    http::Header::connection("Upgrade"),
                    http::Header::SecWebSocketAccept(accept),
                ],
            })
        }).ok_or("Unable to create websocket upgrade response from request".into())
}

fn make_http_response(status_code: http::StatusCode, body: Option<String>) -> Result<http::Response, String> {
    if body.is_some() { todo!() };
    Ok(http::Response {
        version: http::Version::V1_1,
        status_code, 
        headers: vec![http::Header::ContentLength(0)],
    })
}

pub struct Server {
    listener: TcpListener,
    _connections: Vec<TcpStream>,
}

fn parse_request(reader: &mut BufReader<&TcpStream>) -> Result<Request, String>{
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
    let request: http::Request = msg.parse().map_err(|err| dbg!(err)).map_err(|_| "Unable to parse http request")?;
    reader.consume(end_index + 4); // Consume end-of-message
    Ok(request)
}

fn write_response(writer: &mut BufWriter<&TcpStream>, response: &Response) -> Result<(), String> {
    writer.write_all(response.to_string().as_bytes()).map_err(|_| "Unable to write response to stream")?;
    writer.flush().map_err(|_| "Unable to flush stream".into())
}

// TODO; Put somewhere else.
struct Deferred <T: Fn() -> ()>{
    f: T,
}

impl<T: Fn() -> ()> Drop for Deferred<T> {
    fn drop(&mut self) {
       let s: &Self = self;
       let f = &(s.f);
       f();
    }
}

macro_rules! defer_expr { ($e: expr) => { $e } } // tt hack
macro_rules! defer {
    ( $($s:tt)* ) => {
        let _deferred = Deferred { f: || {
            defer_expr!({ $($s)* })
        }}; 
    };
    () => {};
}

impl Server{
    pub fn new(listener: TcpListener) -> Self {
        Server { listener, _connections: vec![] }
    }
    pub fn listen(&mut self) -> io::Result<()> {
        let listener = &self.listener;
        // let connections = &mut self.connections;
        for stream in listener.incoming() {
            let stream = stream?;
            let peer_addr = stream.peer_addr()?;
            // Each connection uses its own thread. Simple but does not scale. Fine for dev server.
            thread::spawn(move || {
                defer! { println!("Closed connection {peer_addr}") };
                println!("Got Connection {}", peer_addr);
                let mut reader = BufReader::new(&stream);
                let mut writer = BufWriter::new(&stream);
                let mut done = false;
                loop {
                    let Ok(req) = parse_request(&mut reader).map_err(|err| println!("{}", err)) else { return };
                    let resp = if is_valid_websocket(&req) { 
                        done = true;
                        make_websocket_accept_response(&req)
                    } else {
                        make_http_response(http::StatusCode(200), None)
                    };
                    let Ok(resp) = resp.map_err(|err| println!("{}", err)) else { continue };
                    print!("Sending HTTP response to {peer_addr}:\n{resp}");
                    let Ok(_) = write_response(&mut writer, &resp).map_err(|err| println!("{}", err)) else { continue };
                    if done { break };
                }
                defer! { println!("Closed WebSocket Connection {peer_addr}") };
                println!("Got WebSocket Connection {}", peer_addr);
                loop {
                    let Ok(buffer) = reader.fill_buf().map_err(|err| println!("{}", err)) else { return };
                    let length = buffer.len();
                    if length == 0 { break };
                    
                    // work with buffer
                    println!("Received websocket message from {peer_addr}: {buffer:?}");
                    
                    // ensure the bytes we worked with aren't returned again later
                    reader.consume(length);

                    // Reply with static message
                    let ws_message: [u8; 5] = [0x81, 0x03, b'h', b'e', b'j'];
                    println!("Sending websocket message to {peer_addr}");
                    let Ok(_) = writer.write_all(&ws_message).map_err(|err| println!("{}", err)) else { return };
                    let Ok(_) = writer.flush().map_err(|err| println!("{}", err)) else { return };
                };
            });
        }
        Ok(())
    }
}
