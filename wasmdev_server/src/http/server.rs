use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{self, BufWriter};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::utils::{defer, load_file};

use super::{Header, StatusCode};
use super::helper::*;

#[derive(PartialEq)]
pub struct Endpoint{
    headers: Vec<Header>,
    response_action: Option<ResponseAction>,
}

pub struct EndpointHasResponse {}
pub trait EndpointBuilderHasResponse {
    /// Build the endpoint. Returns true if endpoint body did change.
    fn build(self) -> bool;
    fn add_response_header(self, header: Header) -> Self;
    fn add_response_headers(self, headers: Vec<Header>) -> Self;
}

pub struct EndpointNoResponse {}
pub trait EndpointBuilderNoResponse<'a> : EndpointBuilderHasResponse {
    fn internal_redirect(self, path: &'a str) -> EndpointBuilder<'a, EndpointHasResponse>;
    fn set_response_body(self, body: Vec<u8>) -> EndpointBuilder<'a, EndpointHasResponse>;
    fn lazy_load(self, path: &'a str)         -> EndpointBuilder<'a, EndpointHasResponse>;
}

#[derive(PartialEq)]
pub enum ResponseAction {
    Content(Vec<u8>),
    InternalRedirect(String),
    LazyLoad(String),
}

pub struct EndpointBuilder<'a, T> {
    server_config: &'a mut ServerConfig,
    path: &'a str,
    endpoint: Endpoint,
    _marker: PhantomData<T> 
}

fn simple_hash(bin: &Vec<u8>) -> u32 {
    let mut res = 0u32;
    let mut index = 0u32;
    for byte in bin {
        res ^= (*byte as u32) << index;
        index = (index + 8) % 32;
    }
    res
}

impl <'a, T> EndpointBuilderHasResponse for EndpointBuilder<'a, T> {
    fn build(self) -> bool {
        let mut endpoint = self.endpoint;
        if !endpoint.headers.iter().any(|h| matches!(h, Header::ContentType(_))) {
            let mime_type = match Path::new(self.path).extension() {
                Some(s) if s == "wasm" => Some("application/wasm"),
                Some(s) if s == "js"   => Some("application/javascript"),
                Some(s) if s == "html" => Some("text/html"),
                Some(s) if s == "css"  => Some("text/css"),
                _                      => None,
                // TODO: Add more mime types in a compact way
            };
            if let Some(mime_type) = mime_type {
                endpoint.headers.push(Header::ContentType(mime_type.into()));
            }
        };
        if !endpoint.headers.iter().any(|h| matches!(h, Header::ContentLength(_))) {
            if let Some(size) = match &endpoint.response_action {
                Some(ResponseAction::Content(body)) => Some(body.len()),
                _ => None,
            } {
                endpoint.headers.push(Header::ContentLength(size));
            }
        }
        let endpoint_hash = match &endpoint.response_action {
            Some(ResponseAction::Content(body)) => Some(simple_hash(body)),
            _ => None,
        };
        let Some(old_endpoint) = self.server_config.endpoints.insert(self.path.into(), endpoint) else {
            return true;
        };
        let old_endpoint_hash = match &old_endpoint.response_action {
            Some(ResponseAction::Content(body)) => Some(simple_hash(body)),
            _ => None,
        };
        // Does not check headers, only response body. Might need to change in the future.
        old_endpoint_hash != endpoint_hash
    }
    #[inline]
    fn add_response_header(self, header: Header) -> Self {
        self.add_response_headers(vec![header])
    }
    fn add_response_headers(self, mut headers: Vec<Header>) -> Self {
        for header in self.endpoint.headers {
            headers.push(header);
        }
        Self {
            endpoint: Endpoint { 
                headers: headers, 
                response_action: self.endpoint.response_action 
            },
            path: self.path,
            server_config: self.server_config,
            _marker: self._marker,
        }
    }
}

impl <'a> EndpointBuilderNoResponse<'a> for EndpointBuilder<'a, EndpointNoResponse> {
    fn internal_redirect(self, path: &'a str) -> EndpointBuilder<'a, EndpointHasResponse> {
        EndpointBuilder { 
            server_config: self.server_config, 
            path: self.path,
            endpoint: Endpoint { 
                headers: self.endpoint.headers, 
                response_action: Some(ResponseAction::InternalRedirect(path.to_owned())),
            },
            _marker: Default::default()
        }
    }
    fn set_response_body(self, body: Vec<u8>) -> EndpointBuilder<'a, EndpointHasResponse> {
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
    fn lazy_load(self, path: &'a str) -> EndpointBuilder<'a, EndpointHasResponse> {
        EndpointBuilder { 
            server_config: self.server_config,
            path: self.path,
            endpoint: Endpoint { 
                headers: self.endpoint.headers, 
                response_action: Some(ResponseAction::LazyLoad(path.to_owned())),
            },
            _marker: Default::default()
        }
    }
}

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

    pub fn on_get_request<'a>(&'a mut self, path: &'a str) -> EndpointBuilder<EndpointNoResponse> {
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

#[derive(Clone)]
pub struct Client {
    writer: Arc<RwLock<BufWriter<TcpStream>>>,
    addr: SocketAddr,
}

#[derive(Clone)]
pub struct Server {
    pub config: Arc<RwLock<ServerConfig>>,
    clients: Arc<RwLock<Vec<Client>>>,
}

impl Server{
    pub fn new() -> Self {
        Server { config: Arc::new(RwLock::new(ServerConfig::new())), clients: Arc::new(RwLock::new(vec![])) }
    }
    pub fn broadcast(&self, msg: &[u8]) {
        for client in self.clients.read().unwrap().iter() {
            let mut writer = client.writer.write().unwrap();
            let Ok(_) = writer.write_all(&[0x81, msg.len() as u8]).map_err(|err| println!("{}", err)) else { continue };
            let Ok(_) = writer.write_all(msg)                     .map_err(|err| println!("{}", err)) else { continue };
            let Ok(_) = writer.flush()                            .map_err(|err| println!("{}", err)) else { continue };
        }
    }
    pub fn listen(&self, listener: TcpListener) -> io::Result<()> {
        for stream in listener.incoming() {
            let stream = stream?;
            let peer_addr = stream.peer_addr()?;
            let config = self.config.clone();
            let clients = self.clients.clone();
            let mut reader = BufReader::new(stream.try_clone()?);
            let mut writer = BufWriter::new(stream);

            // Each connection uses its own thread. Simple but does not scale. Fine for dev server.
            thread::spawn(move || {
                defer! { println!("Closed connection {peer_addr}") };
                println!("Got Connection {}", peer_addr);
                let mut upgrade_connection = false;
                loop {
                    let Ok(req) = parse_request(&mut reader) else { return };
                    // If we have a lazy response, we need to store it at this scope-level
                    // in order to cache it after response has been sent.
                    let mut lazy_response = None;
                    let send_ok = if is_valid_websocket(&req) { 
                        upgrade_connection = true;
                        let resp = make_websocket_accept_response(&req);
                        let Ok(resp) = resp.map_err(|err| println!("{}", err)) else { return };
                        write_response(&mut writer, &resp)
                    } else {
                        let mut path = &req.path;
                        let config = config.read().unwrap();
                        let headers_and_action = loop {
                            let Some(endpoint) = config.endpoints.get(path) else { break None };
                            let Some(response_action) = &endpoint.response_action else { break None };
                            match response_action {
                                ResponseAction::InternalRedirect(redirect_path) => { path = &redirect_path; },
                                ResponseAction::LazyLoad(file_path) => {
                                    let Some(body) = load_file(Path::new(file_path)) else { break None };
                                    let mut headers = endpoint.headers.clone();
                                    headers.push(Header::ContentLength(body.len()));
                                    lazy_response = Some((path.clone(), headers, ResponseAction::Content(body)));
                                    let Some((_, headers, response_action)) = &lazy_response else { break None };
                                    break Some((headers, response_action));
                                },
                                _ => {
                                    break Some((&endpoint.headers, response_action));
                                },
                            }
                        };
                        let resp = {
                            match headers_and_action {
                                Some((headers, ResponseAction::Content(body))) => 
                                    make_http_response(StatusCode(200), headers.clone(), Some(&body)),
                                _ => 
                                    make_http_response(StatusCode(404), vec![], None),
                                
                            }
                        };
                        let Ok(resp) = resp.map_err(|err| println!("{}", err)) else { continue };
                        write_response(&mut writer, &resp)
                    };
                    let Ok(_) = send_ok.map_err(|err| println!("{}", err)) else { continue };
                    println!("Sent HTTP response to {peer_addr}");
                    if let Some((path, headers, ResponseAction::Content(body))) = lazy_response {
                        config.write().unwrap()
                            .on_get_request(&path)
                            .add_response_headers(headers)
                            .set_response_body(body)
                            .build();
                    }
                    if upgrade_connection { break };
                }
                defer! { 
                    clients.write().unwrap().retain(|client| client.addr != peer_addr);
                    println!("Closed WebSocket Connection {peer_addr}")
                };
                clients.write().unwrap().push(
                    Client { writer: Arc::new(RwLock::new(writer)), addr: peer_addr }
                );
                println!("Got WebSocket Connection {}", peer_addr);
                loop {
                    let Ok(buffer) = reader.fill_buf().map_err(|err| println!("{}", err)) else { return };
                    let length = buffer.len();
                    if length == 0 { break };
                    
                    // work with buffer
                    println!("Received websocket message from {peer_addr}: {buffer:?}");
                    
                    // ensure the bytes we worked with aren't returned again later
                    reader.consume(length);

                    // // Reply with static "reload" message
                    // let ws_message: [u8; 8] = [0x81, 0x06, b'r', b'e', b'l', b'o', b'a', b'd'];
                    // println!("Sending websocket message to {peer_addr}");
                    // let Ok(_) = writer.write_all(&ws_message).map_err(|err| println!("{}", err)) else { return };
                    // let Ok(_) = writer.flush().map_err(|err| println!("{}", err)) else { return };
                    // // break;
                };
            });
        }
        Ok(())
    }
}
