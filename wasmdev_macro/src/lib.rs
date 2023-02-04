use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, main_fn: TokenStream) -> TokenStream {

    let wasm_fn: TokenStream2 = main_fn.into();
    let wasm_main_fn   = make_wasm_main_fn(&wasm_fn);
    let server_main_fn = make_server_main_fn(&wasm_fn);

    quote! {
        #[cfg(not(target_family = "wasm"))]
        #server_main_fn
        #[cfg(target_family = "wasm")]
        #wasm_main_fn
    }.into()
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
            wasmdev::console_error_panic_hook::set_once();
            #wasm_main_fn_ident ();
        }
    }
}

fn make_server_main_fn(wasm_main_fn: &TokenStream2) -> TokenStream2 {
    let index_js     = include_str!("index.js");
    let index_html   = include_str!("index.html");
    let index_html   = format!("{index_html}\n<script type=\"module\">{index_js}</script>"); 
    let is_release   = !cfg!(debug_assertions);
    let release_mode = if is_release {"release"} else {"debug"};

    let wasm_main_fn_ident = get_fn_name(wasm_main_fn).expect("Unable to get function name of main function");

    quote!{
        fn main() {
            use std::net::TcpListener;
            use std::path::Path;
            use std::str::from_utf8;
            use std::sync::{Arc,RwLock};
            use wasmdev::prelude::*;
            use wasmdev::{Server, ServerConfig};
            use wasmdev::utils::{build_wasm, load_file, make_watcher};

            // Make sure rust analyzer analyze the wasm code for better code-completion experience:
            #wasm_main_fn
            // Make a call to it that never executes to avoid compiler warnings:
            if false { #wasm_main_fn_ident () }

            // TODO: make path to static files configurable, including index.html:
            static wasm_path:       &str = concat!("target/wasm32-unknown-unknown", "/", #release_mode, "/", env!("CARGO_PKG_NAME"), ".wasm");
            static index_js_path:   &str = concat!("target/wasm32-unknown-unknown", "/", #release_mode, "/", env!("CARGO_PKG_NAME"), ".js");
            static index_wasm_path: &str = concat!("target/wasm32-unknown-unknown", "/", #release_mode, "/", env!("CARGO_PKG_NAME"), "_bg.wasm");
            static index_html_path: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "index.html");
            static rust_src_path:   &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "src");

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
                move || {
                    println!("\x1b[1m\x1b[92m    Building\x1b[0m wasm target");
                    let Some(_)         = build_wasm(wasm_path, #is_release)    else { return };
                    let Some(wasm_code) = load_file(Path::new(index_wasm_path)) else { return };
                    let Some(js_code)   = load_file(Path::new(index_js_path))   else { return };
                    println!("\x1b[1m\x1b[92m      Loaded\x1b[0m index.wasm, index.js");
                    {
                        let mut server_config = server.config.write().unwrap();
                        server_config
                            .on_get_request("/index.wasm")
                            .set_response_body(wasm_code)
                            .build();
                        server_config
                            .on_get_request("/index.js")
                            .set_response_body(js_code)
                            .build();
                    }
                    server.broadcast("reload /index.wasm".as_bytes());
                }
            };
            
            let load_and_serve_file = {
                let mut server = server.clone();
                move |file_path| {
                    let Some(file_contents) = load_file(file_path) else { return };
                    // server_config.write().unwrap()
                    //     .on_get_request(Path::new("/").join(file_path).as_str())
                    //     .set_response_body(&file_contents)
                    //     .build();
                }
            };
            
            let load_and_serve_index_html = {
                let mut server = server.clone();
                move || {
                    let Some(index_html) = load_file(Path::new(index_html_path)) else { return };
                    let index_html = from_utf8(&index_html).expect("index.html is not utf8 encoded.");
                    let index_html = format!("{}\n<script type=\"module\">{}</script>",index_html, #index_js); 
                    println!("\x1b[1m\x1b[92m      Loaded\x1b[0m index.html");
                    server.config.write().unwrap()
                        .on_get_request("/index.html")
                        .set_response_body(index_html.as_bytes().to_vec())
                        .build();
                }
            };

            // Watcher on src-code required.
            build_load_and_serve_app();
            let _watcher_index_wasm = make_watcher(Path::new(rust_src_path), move |_| build_load_and_serve_app())
                .expect("Unable to watch src folder, required for hot-reload.");

            // Providing a custom index.html is optional, so open watcher is allowed to fail silently here.
            load_and_serve_index_html();
            let _watcher_index_html = make_watcher(Path::new(index_html_path), move |_| load_and_serve_index_html());
            
            // TODO:
            // - find all files in "static" path and tell server those paths exists
            // - watch "static" path for changes
            
            println!("\x1b[1m\x1b[92m            \x1b[0m ┏\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┓");
            println!("\x1b[1m\x1b[92m     Serving\x1b[0m ┃\x1b[1m http://127.0.0.1:8123 \x1b[0m┃ <- Click to open your app! ");
            println!("\x1b[1m\x1b[92m            \x1b[0m ┗\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┛");
            
            let tcp_socket = TcpListener::bind("127.0.0.1:8123")
                .expect("Unable to bind tcp port 8123");
            server.listen(tcp_socket)
                .expect("Unable to handle incomming connection");
        }
    }
}
