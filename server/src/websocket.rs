use std::net::{TcpListener, TcpStream};
use std::io::{self, BufWriter};
use std::io::{BufRead, BufReader, Write};
use std::str::from_utf8;
use std::thread;

use sha1::{Sha1, Digest};

use base64::{Engine, DecodeError};

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

fn make_websocket_response(sec_websocket_accept_header: http::Header) -> http::Response {
    http::Response {
        version: http::Version::V1_1,
        status_code: http::StatusCode(101), 
        headers: vec![ 
            http::Header::upgrade("websocket"),
            http::Header::connection("Upgrade"),
            sec_websocket_accept_header,
        ],
    }
}

fn make_http_response(status_code: http::StatusCode) -> http::Response {
    http::Response {
        version: http::Version::V1_1,
        status_code, 
        headers: vec![http::Header::ContentLength(0)],
    }
}

pub struct Server {
    listener: TcpListener,
    _connections: Vec<TcpStream>,
}

fn setup_websocket_connection(stream: TcpStream) -> Option<TcpStream>{
    let valid_websocket_request = {
        let mut buf_reader = BufReader::new(&stream);
        let mut buf_writer = BufWriter::new(&stream);
        // TODO: Be more sofisticated than this.
        // This is a small but inefficient way of reading
        // an http message.
        let valid_websocket_request = loop {
            buf_reader.fill_buf().ok()?;
            if buf_reader.buffer().is_empty() { return None };
            let Some((end_index, consume_count)) = buf_reader.buffer()
                .windows(4)
                .enumerate()
                .find_map(|(i, range)| {
                    let [a, b, c, d] = range else { return None };
                    if *a == '\r' as u8 && *b == '\n' as u8 && *c == '\r' as u8 && *d == '\n' as u8 { 
                        Some((i, i + 4)) 
                    } else if *c == '\n' as u8 && *d == '\n' as u8 { 
                        Some((i + 2, i + 4))
                    } else { 
                        None
                    }
                }) else { continue };
                
            let msg = from_utf8(&buf_reader.buffer()[0..end_index]).ok()?;
            let request: http::Request = msg.parse().map_err(|err| dbg!(err)).ok()?;
            buf_reader.consume(consume_count);
            let valid_websocket_request = is_valid_websocket(&request);
            let response = if valid_websocket_request {
                request.headers.iter()
                    .find_map(|header| { 
                        let http::Header::SecWebSocketKey(key) = header else { return None };
                        compute_accept(key).ok()
                    })
                    .map(|accept| http::Header::SecWebSocketAccept(accept))
                    .map(make_websocket_response)?
            } else { 
                make_http_response(http::StatusCode(200))
            };
            // TODO: Maybe we could write to stream directly
            // rather than converting to a String in-between?
            print!("Sending:\n{}", response);
            buf_writer.write_all(response.to_string().as_bytes()).ok()?;
            buf_writer.flush().ok()?;
            if valid_websocket_request { break true };
        };
        valid_websocket_request
    };
    if valid_websocket_request { return Some(stream) };
    None
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
            thread::spawn(move || {
                setup_websocket_connection(stream).map(|stream| -> io::Result<()> {
                    let peer_addr = stream.peer_addr()?;
                    println!("Got WebSocket Connection {}", peer_addr);
                    let mut buf_reader = BufReader::new(&stream);
                    let mut buf_writer = BufWriter::new(&stream);
                    loop {
                        let buffer = buf_reader.fill_buf()?;
                        let length = buffer.len();
                        if length == 0 { break };
                        
                        // work with buffer
                        println!("From {peer_addr}: {buffer:?}");
                        
                        // ensure the bytes we worked with aren't returned again later
                        buf_reader.consume(length);

                        // Reply with static message
                        let ws_message: [u8; 5] = [0x81, 0x03, b'h', b'e', b'j'];
                        buf_writer.write_all(&ws_message)?;
                        buf_writer.flush()?;

                    };
                    println!("Closed WebSocket Connection {peer_addr}");
                    Ok(())
                });
                println!("Closed connection");
            });
        }
        Ok(())
    }
}
