use proc_macro::TokenStream as BuiltInTokenStream;
use proc_macro2::{TokenStream, TokenTree, Literal, Punct, Spacing, Ident, Group, Delimiter, Span};
use quote::quote;
use std::{env, fs};
use std::collections::HashSet;
use std::path::Path;
use std::str::from_utf8;
use wasmdev_server::utils::{build_wasm, minify_javascript, load_file, find_files};

mod util;

struct Attr<T> {
    value: T,
    tt: Option<TokenTree>,
}

impl <T> Attr<T> {
    fn new(value: T, tt: Option<TokenTree>) -> Self {
        Attr{ value, tt }
    }
}

struct Config {
    port: Attr<u16>,
    path: Attr<String>,
    addr: Attr<String>,
    watch: Attr<bool>,
}

///
/// Turns the main function for non-`wasm` targets into a development web-server.
/// 
/// ### Arguments
/// * **addr** (optional): Socket address to webserver 
///   - Default "127.0.0.1"
/// * **path** (optional): Path to static web assets 
///   - Default: "src"
/// * **port** (optional): TCP socket port to use 
///   - Default: 8080
/// * **watch** (optional): Reload assets on file-system changes
///   - Default: true
///   - Note: **Only affects debug build**, always false for release build
/// 
/// ### Usage
/// ```rust
/// // src/main.rs
/// #[wasmdev::main]
/// fn main() {
///     let window = web_sys::window().unwrap();
///     let document = window.document().unwrap();
///     let body = document.body().unwrap();
///     let val = document.create_element("p").unwrap();
///     val.set_text_content(Some("Hello World"));
///     body.append_child(&val).unwrap();
/// }
/// 
/// ```
/// From terminal:
/// ```bash
/// cargo run # No extra targets or tools required
/// ```
/// ### Example: Manually specifying static asset directory
/// ```rust
/// #[wasmdev::main(path: "www")]
/// fn main() {
/// // ...
/// }
/// ```
/// File tree:
/// ```
/// ├── Cargo.toml
/// ├── src
/// │   └── main.rs
/// └── www
///     └── index.html
/// ```
/// ### Example: Allow external devices to run app
/// ```rust
/// // This allows all traffic through the firewall, use with extreme care
/// #[wasmdev::main(addr: "0.0.0.0")]
/// fn main() {
/// // ...
/// }
/// ```
/// 
#[proc_macro_attribute]
pub fn main(attrs: BuiltInTokenStream, main_fn: BuiltInTokenStream) -> BuiltInTokenStream {
    match (|| -> Result<TokenStream, TokenStream> {
        let wasm_fn: TokenStream  = main_fn.into();
        let config                = parse_config_attrs(attrs.into())?;
        let wasm_main_fn          = make_wasm_main_fn(&wasm_fn)?;
        let server_main_fn        = make_server_main_fn(&wasm_fn, config)?;
    
        Ok(quote! {
            #[cfg(not(target_family = "wasm"))]
            #server_main_fn
            #[cfg(target_family = "wasm")]
            #wasm_main_fn
        })  
    })() {
        Ok(tt) => tt,
        Err(tt) => tt,
    }.into()
}

fn emit_compilation_error(msg: &str, span: &Span) -> TokenStream {
    let span = span.clone();
    TokenStream::from_iter(vec![
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct({
            let mut punct = Punct::new('!', Spacing::Alone);
            punct.set_span(span);
            punct
        }),
        TokenTree::Group({
            let mut group = Group::new(Delimiter::Brace, {
                TokenStream::from_iter(vec![TokenTree::Literal({
                    let mut string = Literal::string(msg);
                    string.set_span(span);
                    string
                })])
            });
            group.set_span(span);
            group
        }),
    ])
}

/// This trait is only here so that "compiler_error" macro works for both TokenTrees and Spans.
trait IntoSpanSelf {
    fn span(self) -> Span;
}

impl IntoSpanSelf for Span {
    fn span(self) -> Span {
        self
    }
}

macro_rules! compiler_error { 
    ( $i:ident, $($args:tt)* ) => { 
        {
            let span = ($i).span();
            Err(emit_compilation_error(&format!($($args)*), &span))
        }
    };
    ( $($args:tt)* ) => { 
        {
            let span = Span::call_site();
            Err(emit_compilation_error(&format!($($args)*), &span))
        }
    };
}

fn parse_config_attrs(attrs: TokenStream) -> Result<Config, TokenStream> {
    let mut it = attrs.into_iter();
    let mut port = None;
    let mut path = None;
    let mut addr = None;
    let mut watch = None;

    struct NoQuotesError;
    let trim_quotes = |value: &str| -> Result<String, NoQuotesError> {
        if value.as_bytes()[0] != ('"' as u8) || value.as_bytes()[value.as_bytes().len() - 1] != ('"' as u8) { 
            return Err(NoQuotesError);
        }
        Ok(value[1..value.len() - 1].to_string())
    };

    loop {
        
        let Some(TokenTree::Ident(ident)) = it.next() else { 
            break
        };
        let Some(TokenTree::Punct(punct)) = it.next() else { 
            return compiler_error!(ident, "Incomplete attribute list, expected '=' or ':' after '{ident}'")
        };
        
        match punct.to_string().as_str() {
            ":" => (),
            "=" => (),
            _ => return compiler_error!(punct, "Unexpected character '{punct}'. Expected ':' or '='"),
        };
        
        let Some(value) = it.next() else { 
            return compiler_error!(punct, "Incomplete attribute list, expected value after '{punct}'")
        };
        let value_as_str = match &value {
            TokenTree::Literal(value) => value.to_string(),
            TokenTree::Ident(value) => value.to_string(),
            _ => return compiler_error!(value, "Unexpected token: '{value}'"),
        };

        match ident.to_string().as_str() {
            "addr" => { 
                let Ok(val) = trim_quotes(&value_as_str) else {
                    return compiler_error!(value, "Unable to parse addr, {value} is not a `&str`");                    
                };
                addr = Some(Attr::new(val, Some(value.into())));
            },
            "path" => { 
                let Ok(val) = trim_quotes(&value_as_str) else {
                    return compiler_error!(value, "Unable to parse path, {value} is not a `&str`");
                };
                path = Some(Attr::new(val, Some(value.into())));
            },
            "port" => { 
                let Ok(val) = value_as_str.parse() else { 
                    return compiler_error!(value, "Unable to parse port, {value} is not a u16");
                };
                port = Some(Attr::new(val, Some(value.into())));
            },
            "watch" => {
                let Ok(val) = value_as_str.parse() else { 
                    return compiler_error!(value, "Unable to parse watch, {value} is not boolean");
                };
                watch = Some(Attr::new(val, Some(value.into())));
            }
            i  => { 
                return compiler_error!(ident, "Unknown attribute: '{i}', help: available attributes are: 'addr', 'path', 'port' and watch");
            },
        }

        match it.next() {
            Some(TokenTree::Punct(punct)) if punct.to_string() == "," => (),
            None => (),
            Some(tt) => return compiler_error!(tt, "Unexpected character '{tt}', help: use ',' to separate attributes."),
        }
    };
    Ok(Config { 
        port: port.unwrap_or(Attr::new(8080, None)), 
        path: path.unwrap_or(Attr::new("src".into(), None)), 
        addr: addr.unwrap_or(Attr::new("127.0.0.1".into(), None)),
        watch: watch.unwrap_or(Attr::new(true, None)),
    })
}

fn get_fn_name(func: &TokenStream) -> Option<TokenTree> {
    let mut it = func.clone().into_iter().skip_while(|tt| {
        let TokenTree::Ident(ident) = tt else { return true };
        ident.to_string() != "fn"
    });
    it.next(); // Skip "fn" identifier
    it.next()  // Should be function name identifier
}

fn make_wasm_main_fn(wasm_main_fn: &TokenStream) -> Result<TokenStream, TokenStream> {
    let Some(wasm_main_fn_ident) = get_fn_name(wasm_main_fn) else {
        return compiler_error!("No main function found");
    };
    Ok(quote! {
        fn main() {
            #wasm_main_fn
            wasmdev::if_enabled_setup_panic_hook_once();
            #wasm_main_fn_ident ();
        }
    })
}

fn make_server_main_fn(wasm_main_fn: &TokenStream, config: Config) -> Result<TokenStream, TokenStream> {
    let Some(wasm_main_fn_ident) = get_fn_name(wasm_main_fn) else {
        return compiler_error!("No main function found");
    };
    let Ok(proj_dir) = env::var("CARGO_MANIFEST_DIR") else {
        return compiler_error!("Cargo did not set env var: CARGO_MANIFEST_DIR")
    };
    let Ok(proj_name) = env::var("CARGO_PKG_NAME") else {
        return compiler_error!("Cargo did not set env var: CARGO_PKG_NAME");
    };

    let is_release       = !cfg!(debug_assertions);
    let server_port      = config.port.value;
    let server_path      = config.path.value;
    let server_addr      = config.addr.value;
    let server_watch     = config.watch.value && !is_release;
    let index_js         = include_str!("index.js");
    let index_html       = include_str!("index.html");
    let release_mode     = if is_release {"release"} else {"debug"};
    let index_js         = if is_release {index_js.split("// -- debug -- \\").next().unwrap()} else {index_js};
    let index_html       = format!("{index_html}\n<script type=\"module\">{index_js}</script>"); 
    let target_path      = format!("target/wasmdev-build-cache");
    let out_path         = format!("{target_path}/wasm32-unknown-unknown");
    let wasm_path        = format!("{out_path}/{release_mode}/{proj_name}.wasm");
    let index_js_path    = format!("{out_path}/{release_mode}/{proj_name}.js");
    let index_wasm_path  = format!("{out_path}/{release_mode}/{proj_name}_bg.wasm");
    let proj_html_path   = format!("{proj_dir}/{server_path}/index.html");
    let proj_server_path = format!("{proj_dir}/{server_path}");
    let proj_src_path    = format!("{proj_dir}/src");
    let build_wasm_now   = env::var("CARGO_WASMDEV").ok().is_none() && is_release;

    // Check that server path for static assets exists:
    let Ok(_) = std::fs::metadata(&proj_server_path) else {
        let span = config.path.tt.map(|tt| tt.span()).unwrap_or(Span::call_site());
        return compiler_error!(span, "Error: Unable to read directory: {}", &proj_server_path);
    };
    // Check that provided ip address is an ip address:
    let Ok(_) = server_addr.parse::<std::net::IpAddr>() else {
        let span = config.addr.tt.map(|tt| tt.span()).unwrap_or(Span::call_site());
        return compiler_error!(span, "Error: {} is not a valid ipv4 or ipv6 address", &server_addr);
    };

    // Build project as wasm target during the build step (if building in release mode)
    let tt_invalidate_static_asset_cache = if build_wasm_now {
        let Some(_)         = build_wasm(wasm_path.as_str(), is_release, target_path.as_str())
                              else { return compiler_error!("Failed to build wasm target") };
        let Some(wasm_code) = load_file(Path::new(index_wasm_path.as_str()))
                              else { return compiler_error!("Failed to read wasm code from {index_wasm_path}") };
        let Some(js_code)   = load_file(Path::new(index_js_path.as_str()))
                              else { return compiler_error!("Failed to read js code from {index_js_path}") };
        let js_code         = minify_javascript(&js_code);
        let dist_path       = &format!("target/dist/{proj_name}");
        let html_code = (|| -> Option<String>{
            let html_code = load_file(Path::new(proj_html_path.as_str()))?;
            let html_code = from_utf8(&html_code).ok()?;
            Some(format!("{}\n<script type=\"module\">{}</script>", html_code, index_js))
        })().unwrap_or(index_html.clone());

        match (|| -> Result<TokenStream, std::io::Error> {
            let _ = fs::create_dir_all(dist_path);
            fs::write(format!("{dist_path}/index.wasm"), wasm_code)?;
            fs::write(format!("{dist_path}/index.js"), js_code)?;
            fs::write(format!("{dist_path}/index.html"), html_code)?;
    
            let file_paths = find_files(Path::new(&proj_server_path));
            let file_path_iter = file_paths.iter()
                .filter_map(|p| p.to_str())
                .filter(|p| !p.ends_with(".rs"))          // Don't export src files.
                .filter(|p| !p.ends_with("/index.html")); // index.html already handled.

            // Clean up old files that were removed since last build:
            {
                let old_files = find_files(Path::new(&dist_path));
                let mut old_file_paths: HashSet<_> = old_files.iter()
                    .filter_map(|p| p.to_str())
                    .map(|p| p.to_string().replace(dist_path, ""))
                    .filter(|p| !p.ends_with("/index.wasm"))
                    .filter(|p| !p.ends_with("/index.js"))
                    .filter(|p| !p.ends_with("/index.html"))
                    .collect();
    
                for file_path in file_path_iter.clone() {
                    old_file_paths.remove(&file_path.replace(&proj_server_path, ""));
                }
                let files_to_remove = old_file_paths.iter().map(|p| format!("{dist_path}{p}"));
                for file_path in files_to_remove {
                    fs::remove_file(file_path)?;
                }
                util::remove_empty_dirs(Path::new(dist_path))?;
            }

            for file_path in file_path_iter {
                let file_contents = fs::read(file_path)?;
                let file_contents = if file_path.ends_with(".js") { 
                    minify_javascript(&file_contents)
                } else { file_contents };
                let file_rel_path = file_path.replace(&proj_server_path, "");
                let file_dist_path = format!("{dist_path}/{file_rel_path}");
                // Create parent directory to make sure it exists:
                let _ = Path::new(&file_dist_path).parent().map(|p| fs::create_dir_all(p));
                fs::write(file_dist_path, file_contents)?;
            }
            // Abuse "include_bytes" to make sure static web assets invalidate cargo build cache
            let tt_invalidate_static_asset_cache = TokenStream::from_iter(file_paths.iter()
                .filter_map(|p| p.to_str())
                .filter(|p| !p.ends_with(".rs"))
                .map(|p| quote!{ include_bytes!(#p); })
            );
            eprintln!("\x1b[1m\x1b[92m    Finished\x1b[0m release artifacts in: '{dist_path}'");
            Ok(tt_invalidate_static_asset_cache)
        })() {
            Ok(tt)   => tt,
            Err(msg) => return compiler_error!("Failed to build project '{proj_name}' , {msg}"),
        }
    } else { quote! {} }; // Debug build does not need to invalidate cargo build cache

    Ok(quote!{
        fn main() {

            // Make sure rust analyzer analyze the wasm code for better code-completion experience:
            #wasm_main_fn

            // Scope all this in order to not pollute main fn scope.
            {
                use std::net::TcpListener;
                use std::path::{Path, PathBuf};
                use std::str::from_utf8;
                use wasmdev::prelude::*;
                use wasmdev::{Server, ServerConfig};
                use wasmdev::utils::{build_wasm, load_file, minify_javascript, make_watcher, find_files, Result, Event};

                // Make sure that release build includes the latest versions of static assets:
                #tt_invalidate_static_asset_cache
                // Make sure main is referenced to avoid "unused" compiler warnings:
                #wasm_main_fn_ident;

                let wasm_path        = #wasm_path;
                let index_js_path    = #index_js_path;
                let index_wasm_path  = #index_wasm_path;
                let proj_html_path   = #proj_html_path;
                let proj_server_path = #proj_server_path;
                let proj_src_path    = #proj_src_path;
                let server_addr      = #server_addr;
                let server_port      = #server_port;
                let server_watch     = #server_watch;

                let server = Server::new();
                {
                    let mut server_config = server.config.write().unwrap();
                    server_config
                        .on_get_request("/")
                        .internal_redirect("/index.html")
                        .build();
                    server_config
                        .on_get_request("/index.html")
                        .set_response_body(#index_html.as_bytes().to_vec())
                        .build();
                }

                let build_load_and_serve_app = {
                    let mut server = server.clone();
                    move || -> Option<()>{
                        println!("\x1b[1m\x1b[92m    Building\x1b[0m wasm target");
                        let _         = build_wasm(wasm_path, #is_release, #target_path)?;
                        let wasm_code = load_file(Path::new(index_wasm_path))?;
                        let js_code   = load_file(Path::new(index_js_path))?;
                        let js_code   = if #is_release { minify_javascript(&js_code) } else { js_code };
                        let code_did_update = {
                            let mut server_config = server.config.write().unwrap();
                            server_config
                                .on_get_request("/index.js")
                                .set_response_body(js_code)
                                .build();
                            server_config
                                .on_get_request("/index.wasm")
                                .set_response_body(wasm_code)
                                .build()
                        };
                        if code_did_update {
                            println!("\x1b[1m\x1b[92m     Serving\x1b[0m /index.wasm, /index.js");
                            server.broadcast("reload /index.wasm".as_bytes());
                        }
                        Some(())
                    }
                };

                let serve_static_files = || {
                    let file_paths = find_files(Path::new(proj_server_path));
                    let file_and_req_path_iter = file_paths.iter()
                        .filter_map(|file_path| file_path.to_str())
                        .map(|file_path| (file_path, file_path.replace(proj_server_path, "")))
                        .filter(|(_, req_path)| *req_path != "/index.html");
                    {
                        let mut conf = server.config.write().unwrap();
                        for (file_path, req_path) in file_and_req_path_iter.clone(){
                            conf.on_get_request(&req_path)
                                .lazy_load(file_path)
                                .build();
                        }
                    }
                    for (_, req_path) in file_and_req_path_iter{
                        println!("\x1b[1m\x1b[92m     Serving\x1b[0m {}", req_path);
                    }
                };
                
                let load_and_serve_file = {
                    let mut server = server.clone();
                    move |event: Result<Event> | {
                        let Some(event) = event.ok() else { return };
                        for file_path in event.paths {
                            let file_path = file_path.as_path();
                            let Some(req_path) = file_path.to_str().map(|path| path.replace(proj_server_path, "")) else { continue };
                            if req_path == "/index.html" { continue }; // index.html is handled in another watcher, so skip it.
                            let Some(file_contents) = load_file(file_path) else { continue };
                            let file_did_update = {
                                server.config.write().unwrap()
                                    .on_get_request(&req_path)
                                    .set_response_body(file_contents)
                                    .build()
                            };
                            if file_did_update {
                                println!("\x1b[1m\x1b[92m     Serving\x1b[0m {}", req_path);
                                server.broadcast(format!("reload {}", req_path).as_bytes());
                            }
                        }
                    }
                };
                
                let load_and_serve_index_html = {
                    let mut server = server.clone();
                    move || {
                        let Some(index_html) = load_file(Path::new(proj_html_path)) else { return };
                        let index_html       = from_utf8(&index_html).expect("index.html is not utf8 encoded.");
                        let index_html       = format!("{}\n<script type=\"module\">{}</script>",index_html, #index_js); 
                        let file_did_update = {
                            server.config.write().unwrap()
                            .on_get_request("/index.html")
                            .set_response_body(index_html.as_bytes().to_vec())
                            .build()
                        };
                        if file_did_update {
                            println!("\x1b[1m\x1b[92m     Serving\x1b[0m /index.html");
                            server.broadcast("reload /index.html".as_bytes());
                        }
                    }
                };

                // Load server resources:
                serve_static_files();
                load_and_serve_index_html();
                build_load_and_serve_app();

                let _watchers = if server_watch { Some((
                    make_watcher(Path::new(proj_server_path), load_and_serve_file)
                        .expect("Unable to watch static files folder, required for hot-reload when updated."),
                    make_watcher(Path::new(proj_src_path), move |_| { build_load_and_serve_app(); })
                        .expect("Unable to watch src folder, required for hot-reload."),
                    make_watcher(Path::new(proj_html_path), move |_| load_and_serve_index_html()),
                        // Providing a custom index.html is optional, so open watcher is allowed to fail silently here.
                ))} else { None };

                let addr = format!("{}:{}", server_addr, server_port);
                let Ok(tcp_socket) = TcpListener::bind(addr) else { 
                    panic!("Unable to bind tcp port: {}", server_port)
                };
                let Ok(addr) = tcp_socket.local_addr() else {
                    panic!("Unable to get local socket address.")
                };

                let addr_char_count = addr.to_string().chars().into_iter().count();
                print!("             ┏━━━━━━━━");
                for _ in 0..addr_char_count { print!("━") };
                println!("━┓");
                println!("\x1b[1m\x1b[92m     Serving\x1b[0m ┃\x1b[1m http://{} \x1b[0m┃ <= Click to open your app! ", addr);
                print!("             ┗━━━━━━━━");
                for _ in 0..addr_char_count { print!("━") };
                println!("━┛");

                let Ok(()) = server.listen(tcp_socket) else { 
                    panic!("Unable to handle incomming connection")
                };
            }
        }
    })
}
