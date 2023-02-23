use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::env;

#[derive(Debug, Default)]
struct Config {
    port: u16,
    path: Option<String>,
}

#[proc_macro_attribute]
pub fn main(attrs: TokenStream, main_fn: TokenStream) -> TokenStream {

    let wasm_fn: TokenStream2 = main_fn.into();
    let config                = parse_config_attrs(attrs.into());
    let wasm_main_fn          = make_wasm_main_fn(&wasm_fn);
    let server_main_fn        = make_server_main_fn(&wasm_fn, config);

    quote! {
        #[cfg(not(target_family = "wasm"))]
        #server_main_fn
        #[cfg(target_family = "wasm")]
        #wasm_main_fn
    }.into()
}

fn parse_config_attrs(attrs: TokenStream2) -> Config {
    let mut it = attrs.into_iter();
    let mut port: u16 = 8080;
    let mut path = None;

    loop {
        let Some(TokenTree::Ident(ident)) = it.next() else { break };
        let Some(TokenTree::Punct(punct)) = it.next() else { panic!("Incomplete attribute list, expected '=' or ':' after '{ident}'") };
        let Some(TokenTree::Literal(value)) = it.next() else { panic!("Incomplete attribute list, expected value after '{punct}'") };

        match punct.to_string().as_str() {
            ":" => (),
            "=" => (),
            _ => panic!("Must use ':' or '=' to assign a value to '{ident}'"),
        };

        match ident.to_string().as_str() {
            "port" => { 
                let Some(p) = value.to_string().parse().ok() else { 
                    panic!("Unable to parse port, {value} is not a u16")
                };
                port = p;
            },
            "path" => {
                let value = value.to_string();
                if value.as_bytes()[0] != ('"' as u8) || value.as_bytes()[value.as_bytes().len() - 1] != ('"' as u8) { 
                    panic!("Unable to set path, {value} is not a &str literal")
                }
                path = Some(value[1..value.len() - 1].to_string())
            },
            ident  => panic!("Unknown attribute: {ident}, Tip: possible attributes in {:?}", Config::default()),
        }

        match it.next() {
            Some(TokenTree::Punct(punct)) if punct.to_string() == "," => (),
            None => (),
            Some(tt) => panic!("Unrecognised character '{tt}', Tip: use ',' to separate attributes."),
        }
    };
    Config { port, path }
}

fn get_fn_name(func: &TokenStream2) -> Option<TokenTree> {
    let mut it = func.clone().into_iter().skip_while(|tt| {
        let TokenTree::Ident(ident) = tt else { return true };
        ident.to_string() != "fn"
    });
    it.next(); // Skip "fn" identifier
    it.next()  // Should be function name identifier
}

fn make_wasm_main_fn(wasm_main_fn: &TokenStream2) -> TokenStream2 {
    let wasm_main_fn_ident = get_fn_name(wasm_main_fn)
        .expect("Unable to get function name of main function");
    quote! {
        fn main() {
            #wasm_main_fn
            wasmdev::if_enabled_setup_panic_hook_once();
            #wasm_main_fn_ident ();
        }
    }
}

fn make_server_main_fn(wasm_main_fn: &TokenStream2, config: Config) -> TokenStream2 {
    let wasm_main_fn_ident = get_fn_name(wasm_main_fn)
        .expect("Unable to get function name of main function");
    let proj_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Cargo did not set env var: CARGO_MANIFEST_DIR");
    let proj_name = env::var("CARGO_PKG_NAME")
        .expect("Cargo did not set env var: CARGO_PKG_NAME");

    let server_port      = config.port;
    let server_path      = config.path.unwrap_or("src".to_string());
    let is_release       = !cfg!(debug_assertions);
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

    if build_wasm_now {
        use std::fs;
        use std::path::Path;
        use std::ffi::OsStr;
        use std::str::from_utf8;
        use wasmdev_server::utils::{build_wasm, minify_javascript, load_file, find_files};
        let dist_path       = &format!("target/dist/{proj_name}");
        let Some(_)         = build_wasm(wasm_path.as_str(), is_release, target_path.as_str())
            else { panic! ("Failed to build wasm target") };
        let Some(wasm_code) = load_file(Path::new(index_wasm_path.as_str()))
            else { panic! ("Failed to read wasm code") };
        let Some(js_code)   = load_file(Path::new(index_js_path.as_str()))
            else { panic! ("Failed to read js code")  };
        let js_code         = minify_javascript(&js_code);
        let html_code = (|| -> Option<String>{
            let html_code = load_file(Path::new(proj_html_path.as_str()))?;
            let html_code = from_utf8(&html_code).ok()?;
            Some(format!("{}\n<script type=\"module\">{}</script>", html_code, index_js))
        })().unwrap_or(index_html.clone());

        match (|| -> Result<(), std::io::Error> {
            let _ = fs::create_dir_all(dist_path);
            fs::write(format!("{dist_path}/index.wasm"), wasm_code)?;
            fs::write(format!("{dist_path}/index.js"), js_code)?;
            fs::write(format!("{dist_path}/index.html"), html_code)?;
    
            let file_paths = find_files(Path::new(&proj_server_path));
            let file_path_iter = file_paths.iter()
                .filter_map(|file_path| file_path.to_str())
                .filter(|file_path| !file_path.ends_with("/index.html")); // index.html already handled.
            for file_path in file_path_iter {
                let file_contents = fs::read(file_path)?;
                let file_contents = if Path::new(file_path).extension() == Some(OsStr::new("js")) { 
                    minify_javascript(&file_contents)
                } else { file_contents };
                let file_rel_path = file_path.replace(&proj_server_path, "");
                let file_dist_path = format!("{dist_path}/{file_rel_path}");
                // Create parent directory to make sure it exists:
                let _ = Path::new(&file_dist_path).parent().map(|p| fs::create_dir_all(p));
                fs::write(file_dist_path, file_contents)?;
            }
            Ok(())
        })() {
            Ok(_)    => println!("\x1b[1m\x1b[92m    Finished\x1b[0m release artifacts in: '{dist_path}'"),
            Err(msg) => panic!("Failed to build project '{proj_name}' , {msg}"),
        };
    }

    quote!{
        fn main() {
            use std::net::TcpListener;
            use std::path::{Path, PathBuf};
            use std::str::from_utf8;
            use wasmdev::prelude::*;
            use wasmdev::{Server, ServerConfig};
            use wasmdev::utils::{build_wasm, load_file, minify_javascript, make_watcher, find_files, Result, Event};

            // Make sure rust analyzer analyze the wasm code for better code-completion experience:
            #wasm_main_fn
            // Make a call to it that never executes to avoid compiler warnings:
            if false { #wasm_main_fn_ident () }

            let wasm_path        = #wasm_path;
            let index_js_path    = #index_js_path;
            let index_wasm_path  = #index_wasm_path; 
            let proj_html_path   = #proj_html_path;
            let proj_server_path = #proj_server_path;
            let proj_src_path    = #proj_src_path;
            let server_port      = #server_port;

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

            let _watchers = if #is_release {None} else {Some((
                make_watcher(Path::new(proj_server_path), load_and_serve_file)
                    .expect("Unable to watch static files folder, required for hot-reload when updated."),
                make_watcher(Path::new(proj_src_path), move |_| { build_load_and_serve_app(); })
                    .expect("Unable to watch src folder, required for hot-reload."),
                make_watcher(Path::new(proj_html_path), move |_| load_and_serve_index_html()),
                    // Providing a custom index.html is optional, so open watcher is allowed to fail silently here.
            ))};
            
            println!("\x1b[1m\x1b[92m            \x1b[0m ┏\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┓");
            println!("\x1b[1m\x1b[92m     Serving\x1b[0m ┃\x1b[1m  http://127.0.0.1:{   } \x1b[0m┃ <= Click to open your app! ", format!("{: <5}", server_port));
            println!("\x1b[1m\x1b[92m            \x1b[0m ┗\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┛");
            
            let addr = format!("127.0.0.1:{}", server_port);
            let Some(tcp_socket) = TcpListener::bind(addr).ok() else { 
                panic!("Unable to bind tcp port: {}", server_port)
            };
            let Some(()) = server.listen(tcp_socket).ok() else { 
                panic!("Unable to handle incomming connection")
            };
        }
    }
}
