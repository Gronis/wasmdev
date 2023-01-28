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

fn generate_make_src_watcher() -> TokenStream2 {
    quote!{
        use std::path::Path;
        use wasmdev::notify::{recommended_watcher, Watcher, RecommendedWatcher, RecursiveMode, Result, EventHandler};
        use wasmdev::notify::event::{Event, EventKind, ModifyKind};

        const SRC_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "src");

        fn make_src_watcher(mut on_wasm_code_updated: impl EventHandler) -> impl Watcher {
            let src_path = Path::new(SRC_PATH);
            let mut watcher = recommended_watcher(move |e: Result<Event>| -> () {
                let Ok(event)                 = &e           else { return };
                let EventKind::Modify(modify) = &event.kind  else { return };
                let ModifyKind::Data(_)       = modify       else { return };
                
                on_wasm_code_updated.handle_event(e);
            }).expect("Unable to initiate watcher");
            watcher.watch(src_path, RecursiveMode::Recursive).expect("Unable to watch src dir");
            watcher
        };
    }
}

fn generate_build_wasm() -> TokenStream2 {
    quote!{
        use std::fs::File;
        use std::io::prelude::*;
        use wasmdev::xshell::{Shell, cmd};

        fn build_wasm() -> Option<Vec<u8>> {
            println!("\x1b[1m\x1b[92m    Building\x1b[0m wasm32-unknown-unknown target");
            let sh = Shell::new().expect("Unable to create shell");
            cmd!(sh, "cargo build --target wasm32-unknown-unknown").quiet().run().ok()?;
    
            let mut wasm_file = File::open("target/wasm32-unknown-unknown/debug/simple.wasm").ok()?;
            let mut wasm_code = Vec::new();
            wasm_file.read_to_end(&mut wasm_code).ok()?;
            println!("\x1b[1m\x1b[92m      Loaded\x1b[0m wasm32-unknown-unknown code");
            Some(wasm_code)
        }
    }
}

fn make_server_main_fn(wasm_main_fn: &TokenStream2) -> TokenStream2 {
    let build_wasm       = generate_build_wasm();
    let make_src_watcher = generate_make_src_watcher();

    let index_js   = include_str!("index.js");
    let index_html = include_str!("index.html");
    let index_html = format!("{index_html}<script>{index_js}</script>"); 
    quote!{
        fn main() {
            use std::net::TcpListener;
            use wasmdev::{Server, Config};

            // Make sure rust analyzer analyze the wasm code for better code-completion:
            #[allow(dead_code)]
            #wasm_main_fn

            // Setup watcher that watches and builds the wasm code:
            #build_wasm
            #make_src_watcher

            let tcp_socket = TcpListener::bind("127.0.0.1:8123").expect("Unable to bind tcp port 8123");
            
            use std::sync::{Arc, Mutex};
            let config = Arc::new(Mutex::new(Config::new()));
            let index_html = #index_html;
            {
                let mut config = config.lock().unwrap();
                config.on_get("/", index_html);
            }
            let mut server = Server::new(tcp_socket);

            let _watcher = make_src_watcher(move |_| {
                let Some(code) = build_wasm() else { return };
            });
            let code = build_wasm().expect("Initial wasm compilation failed");

            // TODO: Add stuff here in order to:
            // - serve index.html
            // - serve wasm code
            // - build wasm code
            // - watch filesystem for changes
            // - add hot reload to wasm code
            // - figure out a way to run tests in browser, but show everyting in cli

            println!("\x1b[1m\x1b[92m            \x1b[0m ┏\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┓");
            println!("\x1b[1m\x1b[92m     Serving\x1b[0m ┃\x1b[1m http://127.0.0.1:8123 \x1b[0m┃ <- Click to open your app! ");
            println!("\x1b[1m\x1b[92m            \x1b[0m ┗\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┛");
            server.listen().expect("Unable to handle incomming connection");
        }
    }
}
