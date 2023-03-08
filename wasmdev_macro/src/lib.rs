use proc_macro::TokenStream as BuiltInTokenStream;
use proc_macro2::{TokenStream, Span};
use quote::quote;
use std::env;

mod util;
mod config;

use util::*;
use config::*;

///
/// Turns the main function for non-`wasm` targets into a development web-server.
/// 
/// ### Optional Arguments
/// * **addr**: Socket address to webserver 
///   - Default "127.0.0.1"
/// * **path**: Path to static web assets 
///   - Default: "src"
/// * **port**: TCP socket port to use 
///   - Default: 8080
/// * **watch**: Reload assets on file-system changes
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

fn make_server_main_fn(wasm_main_fn: &TokenStream, config: AttrConfig) -> Result<TokenStream, TokenStream> {
    // Fail early if macro is annotated on something that is not a function
    let Some(wasm_main_fn_ident) = get_fn_name(wasm_main_fn) else {
        return compiler_error!("No main function found");
    };

    // A build config has all paths and configs needed to build all web assets
    let config: BuildConfig = config.try_into()?;

    // Check that server path for static assets exists:
    let Ok(_) = std::fs::metadata(&config.proj_server_path) else {
        let span = config.attrs.path.tt.map(|tt| tt.span()).unwrap_or(Span::call_site());
        return compiler_error!(span, "Error: Unable to read directory: {}", &config.proj_server_path);
    };

    // Check that provided ip address is an ip address:
    let Ok(_) = config.attrs.addr.value.parse::<std::net::IpAddr>() else {
        let span = config.attrs.addr.tt.map(|tt| tt.span()).unwrap_or(Span::call_site());
        return compiler_error!(span, "Error: {} is not a valid ipv4 or ipv6 address", &config.attrs.addr.value);
    };

    // This enables support for "cargo build --release" to build all assets for us.
    let build_wasm_now = env::var("CARGO_WASMDEV").ok().is_none() && config.is_release;

    // Store static assets so "cargo build" cache invalidation works
    let static_asset_cache = if build_wasm_now { 
        build_all_web_assets(&config)? 
    } else { 
        quote! {} // If we don't bulid web assets at compile-time, we don't a cache
    };

    let address          = config.attrs.addr.value;
    let port             = config.attrs.port.value;
    let watch            = config.attrs.watch.value;
    let is_release       = config.is_release;
    let index_html       = config.index_html;
    let index_js         = config.index_js;
    let target_path      = config.target_path;
    let wasm_path        = config.wasm_path;
    let index_js_path    = config.index_js_path;
    let index_wasm_path  = config.index_wasm_path;
    let proj_html_path   = config.proj_html_path;
    let proj_server_path = config.proj_server_path;
    let proj_src_path    = config.proj_src_path;

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
                #static_asset_cache
                // Make sure main is referenced to avoid "unused" compiler warnings:
                #wasm_main_fn_ident;

                let wasm_path        = #wasm_path;
                let index_js_path    = #index_js_path;
                let index_wasm_path  = #index_wasm_path;
                let proj_html_path   = #proj_html_path;
                let proj_server_path = #proj_server_path;
                let proj_src_path    = #proj_src_path;
                let address      = #address;
                let port      = #port;
                let watch     = #watch;

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

                let file_path_to_req_path = move |path: &str| path.replace(proj_server_path, "").replace("\\", "/");

                let serve_static_files = || {
                    let file_paths = find_files(Path::new(proj_server_path));
                    let file_and_req_path_iter = file_paths.iter()
                        .filter_map(|file_path| file_path.to_str())
                        .map(|file_path| (file_path, file_path_to_req_path(file_path)))
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
                            let Some(req_path) = file_path.to_str().map(file_path_to_req_path) else { continue };
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

                let _watchers = if watch { Some((
                    make_watcher(Path::new(proj_server_path), load_and_serve_file)
                        .expect("Unable to watch static files folder, required for hot-reload when updated."),
                    make_watcher(Path::new(proj_src_path), move |_| { build_load_and_serve_app(); })
                        .expect("Unable to watch src folder, required for hot-reload."),
                    make_watcher(Path::new(proj_html_path), move |_| load_and_serve_index_html()),
                        // Providing a custom index.html is optional, so open watcher is allowed to fail silently here.
                ))} else { None };

                let addr = format!("{}:{}", address, port);
                let Ok(tcp_socket) = TcpListener::bind(addr) else { 
                    panic!("Unable to bind tcp port: {}", port)
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
