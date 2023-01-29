use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, main_fn: TokenStream) -> TokenStream {

    let wasm_main_fn:   TokenStream2 = main_fn.into();
    let server_main_fn: TokenStream2 = make_server_main_fn(&wasm_main_fn);

    quote! {
        #[cfg(not(target_family = "wasm"))]
        #server_main_fn
        #[cfg(target_family = "wasm")]
        #wasm_main_fn
    }.into()
}

fn make_server_main_fn(wasm_main_fn: &TokenStream2) -> TokenStream2 {
    let index_js   = include_str!("index.js");
    let index_html = include_str!("index.html");

    let _index_html = format!("{index_html}<script>{index_js}</script>"); 

    quote!{
        fn main() {
            use std::sync::{Arc,Mutex};
            use std::net::TcpListener;
            use std::path::Path;
            use std::str::from_utf8;
            use wasmdev::{Server, ServerConfig, build_wasm, load_file, make_watcher};
            
            // Make sure rust analyzer analyze the wasm code for better code-completion:
            #[allow(dead_code)]
            #wasm_main_fn

            static wasm_path: &str       = concat!(env!("CARGO_MANIFEST_DIR"), "/", "target/wasm32-unknown-unknown/debug", "/", env!("CARGO_PKG_NAME"), ".wasm");
            static src_path: &str        = concat!(env!("CARGO_MANIFEST_DIR"), "/", "src");
            static index_html_path: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "index.html"); 
            // TODO: make path to static files configurable, including index.html -> ^^^^^^^^^^

            let server_config = Arc::new(Mutex::new(ServerConfig::new()));
            {
                let server_config = server_config.lock().unwrap();
                // server_config
                //     .on_get_request(Path::new("/"))
                //     .internal_redirect(Path::new("/index.html"));
                // server_config
                //     .on_get_request(Path::new("/index.html"))
                //     .set_response_body(#index_html.as_bytes());
            }

            let build_load_and_serve_wasm = {
                let server_config = server_config.clone();
                move || {
                    println!("\x1b[1m\x1b[92m    Building\x1b[0m wasm32-unknown-unknown target");
                    let Some(_) = build_wasm() else { return };
                    let Some(code) = load_file(Path::new(wasm_path)) else { return };
                    println!("\x1b[1m\x1b[92m      Loaded\x1b[0m {}{}", env!("CARGO_PKG_NAME"), ".wasm");
                    // server_config.lock().unwrap()
                    //     .on_get_request(Path::new("/index.wasm"))
                    //     .set_response_body(&code);
                }
            };
            
            let load_and_serve_file = {
                let server_config = server_config.clone();
                move |file_path| {
                    let Some(file_contents) = load_file(file_path) else { return };
                    // server_config.lock().unwrap()
                    //     .on_get_request(Path::new("/").join(file_path).as_path())
                    //     .set_response_body(&file_contents)
                }
            };
            
            let load_and_serve_index_html = {
                let server_config = server_config.clone();
                move || {
                    let Some(index_html) = load_file(Path::new(index_html_path)) else { return };
                    let index_html = from_utf8(&index_html).expect("index.html is not utf8 encoded.");
                    let index_html = format!("{}<script>{}</script>",index_html, #index_js); 
                    println!("\x1b[1m\x1b[92m      Loaded\x1b[0m index.html");
                    // server_config.lock().unwrap()
                    //     .on_get_request(Path::new("/index.html"))
                    //     .set_response_body(index_html.as_bytes())
                }
            };

            // Watcher on src-code required.
            build_load_and_serve_wasm();
            let _watcher_index_wasm = make_watcher(Path::new(src_path), move |_| build_load_and_serve_wasm())
                .expect("Unable to watch src folder, required for hot-reload.");

            // Providing a custom index.html is optional, so open watcher is allowed to fail silently here.
            load_and_serve_index_html();
            let _watcher_index_html = make_watcher(Path::new(index_html_path), move |_| load_and_serve_index_html());
            
            // TODO:
            // - serve get requests
            // - find all files in "static" path and tell server those paths exists
            // - watch "static" path for changes
            // - notify frontend of an update using websocket
            
            let tcp_socket = TcpListener::bind("127.0.0.1:8123").expect("Unable to bind tcp port 8123");
            let mut server = Server::new(tcp_socket, server_config);

            println!("\x1b[1m\x1b[92m            \x1b[0m ┏\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┓");
            println!("\x1b[1m\x1b[92m     Serving\x1b[0m ┃\x1b[1m http://127.0.0.1:8123 \x1b[0m┃ <- Click to open your app! ");
            println!("\x1b[1m\x1b[92m            \x1b[0m ┗\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┛");
            server.listen().expect("Unable to handle incomming connection");
        }
    }
}
