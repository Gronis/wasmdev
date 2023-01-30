use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::{TcpListener, TcpStream};
use std::io::{self, BufWriter};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::utils::defer;

use super::{Header, StatusCode};
use super::helper::*;

pub struct Endpoint{
    headers: Vec<Header>,
    response_action: Option<ResponseAction>,
}

pub struct EndpointAny {}
pub trait EndpointAnyBuilder {
    fn build(self);
    fn add_response_header(&mut self, header: Header) -> &mut Self;
}

pub struct EndpointWithoutContent {}
pub trait EndpointWithoutContentBuilder<'a> : EndpointAnyBuilder {
    fn internal_redirect(self, path: &'a str) -> EndpointBuilder<'a, EndpointAny>;
    fn set_response_body(self, body: Vec<u8>) -> EndpointBuilder<'a, EndpointAny>;
}

pub enum ResponseAction {
    Content(Vec<u8>),
    InternalRedirect(String),
}

pub struct EndpointBuilder<'a, T> {
    server_config: &'a mut ServerConfig,
    path: &'a str,
    endpoint: Endpoint,
    _marker: PhantomData<T> 
}

// impl <'a, T> EndpointBuilder<'a, T> {
//     #[inline]
//     fn borrow_server_config_mut(&'a mut self) -> &'a mut ServerConfig {
//         todo!();
//     }
// }

impl <'a, T> EndpointAnyBuilder for EndpointBuilder<'a, T> {
    #[inline]
    fn build(self) {
        let mut endpoint = self.endpoint;
        let mime_type = match Path::new(self.path).extension() {
            Some(s) if s == "wasm" => Some("application/wasm"),
            Some(s) if s == "html" => Some("text/html"),
            Some(s) if s == "js"   => Some("application/javascript"),
            _                      => None,
        };
        if let Some(mime_type) = mime_type {
            if !endpoint.headers.iter().any(|h| matches!(h, Header::ContentType(_))) {
                endpoint.headers.push(Header::ContentType(mime_type.into()));
            }
        };
        if !endpoint.headers.iter().any(|h| matches!(h, Header::ContentLength(_))) {
            let size = match &endpoint.response_action {
                Some(ResponseAction::Content(body)) => body.len(),
                _ => 0,
            };
            endpoint.headers.push(Header::ContentLength(size));
        }
        self.server_config.endpoints.insert(self.path.to_string(), endpoint);
    }
    #[inline]
    fn add_response_header(&mut self, header: Header) -> &mut Self {
        self.endpoint.headers.push(header);
        self
    }
}

impl <'a> EndpointWithoutContentBuilder<'a> for EndpointBuilder<'a, EndpointWithoutContent> {
    #[inline]
    fn internal_redirect(self, path: &'a str) -> EndpointBuilder<'a, EndpointAny> {
        EndpointBuilder { 
            server_config: self.server_config, 
            path: self.path,
            endpoint: Endpoint { 
                headers: self.endpoint.headers, 
                response_action: Some(ResponseAction::InternalRedirect(path.to_string())),
            },
            _marker: Default::default()
        }
    }
    #[inline]
    fn set_response_body(self, body: Vec<u8>) -> EndpointBuilder<'a, EndpointAny> {
        EndpointBuilder { 
            server_config: self.server_config,
            path: self.path,
            endpoint: Endpoint { 
                headers: self.endpoint.headers, 
                response_action: Some(ResponseAction::Content(body)),
            },
            _marker: Default::default()
        }
    }
}

// impl <'a, T> Drop for EndpointBuilder <'a, T> {
//     #[inline]
//     fn drop(&mut self) {
//         todo!(); // Build and setup endpoint on server config
//     }
// }

// This struct configures how the server should respond to requests
pub struct ServerConfig{
    endpoints: HashMap<String, Endpoint>,
}

impl ServerConfig {
    pub fn new() -> ServerConfig { 
        ServerConfig {
            endpoints: HashMap::from([])
        }
    }

    pub fn on_get_request<'a>(&'a mut self, path: &'a str) -> EndpointBuilder<EndpointWithoutContent> {
        EndpointBuilder { 
            server_config: self, 
            path,
            endpoint: Endpoint { 
                headers: vec![], 
                response_action: None 
            },
            _marker: Default::default()
        }
    }
}

pub struct Server {
    listener: TcpListener,
    config: Arc<RwLock<ServerConfig>>,
    _connections: Vec<TcpStream>,
}

impl Server{
    pub fn new(listener: TcpListener, config: Arc<RwLock<ServerConfig>>) -> Self {
        Server { listener, config: config, _connections: vec![] }
    }
    pub fn listen(&mut self) -> io::Result<()> {
        let listener = &self.listener;
        // let connections = &mut self.connections;
        for stream in listener.incoming() {
            let stream = stream?;
            let peer_addr = stream.peer_addr()?;
            let config = self.config.clone();
            // Each connection uses its own thread. Simple but does not scale. Fine for dev server.
            thread::spawn(move || {
                defer! { println!("Closed connection {peer_addr}") };
                println!("Got Connection {}", peer_addr);
                let mut reader = BufReader::new(&stream);
                let mut writer = BufWriter::new(&stream);
                let mut upgrade_connection = false;
                loop {
                    let Ok(req) = parse_request(&mut reader).map_err(|err| println!("{}", err)) else { return };
                    let send_ok = if is_valid_websocket(&req) { 
                        upgrade_connection = true;
                        let resp = make_websocket_accept_response(&req);
                        let Ok(resp) = resp.map_err(|err| println!("{}", err)) else { continue };
                        write_response(&mut writer, &resp)
                    } else {
                        let mut path = &req.path;
                        let config = config.read().unwrap();
                        let resp = match loop {
                            let Some(endpoint) = config.endpoints.get(path) else { break None };
                            let Some(response_action) = &endpoint.response_action else { break None };
                            match response_action {
                                ResponseAction::Content(body) => break Some((&endpoint.headers, body)),
                                ResponseAction::InternalRedirect(redirect_path) => { path = &redirect_path; },
                            }
                        } {
                            Some((headers, body)) => make_http_response(StatusCode(200), headers.clone(), Some(body)),
                            None                  => make_http_response(StatusCode(404), vec![], None),
                        };
                        let Ok(resp) = resp.map_err(|err| println!("{}", err)) else { continue };
                        write_response(&mut writer, &resp)
                    };
                    let Ok(_) = send_ok.map_err(|err| println!("{}", err)) else { continue };
                    println!("Sent HTTP response to {peer_addr}");
                    if upgrade_connection { break };
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
