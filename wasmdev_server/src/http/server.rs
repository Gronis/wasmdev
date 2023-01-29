use std::net::{TcpListener, TcpStream};
use std::io::{self, BufWriter};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::utils::defer;

use super::{Header, StatusCode};
use super::helper::*;

// This struct configures how the server should respond to requests
pub struct ServerConfig {

}

impl ServerConfig {
    pub fn new() -> ServerConfig { ServerConfig {} }

    pub fn on_get(&mut self, _path: &Path, _headers: Vec<Header>, _response: &[u8]) {
    }
}

pub struct Server {
    listener: TcpListener,
    _config: Arc<Mutex<ServerConfig>>,
    _connections: Vec<TcpStream>,
}

impl Server{
    pub fn new(listener: TcpListener, config: Arc<Mutex<ServerConfig>>) -> Self {
        Server { listener, _config: config, _connections: vec![] }
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
                        make_http_response(StatusCode(200), None)
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
