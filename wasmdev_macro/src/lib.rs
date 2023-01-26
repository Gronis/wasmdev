use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, main_fn: TokenStream) -> TokenStream {
    // dbg!(&_attr);
    // dbg!(&main_fn);
    
    let main_fn_wasm:     TokenStream2 = main_fn.into();
    let main_fn_not_wasm: TokenStream2 = make_main_fn_not_wasm();
    
    quote! {
        #[cfg(not(target_family = "wasm"))]
        #main_fn_not_wasm
        #[cfg(target_family = "wasm")]
        #main_fn_wasm
    }.into()
}

fn make_main_fn_not_wasm() -> TokenStream2 {
    let _script = format!("<script>{}</script>", include_str!("index.js"));
    quote!{
        fn main() {
            use std::net::TcpListener;
            use wasmdev::Server;
            let tcp_socket = TcpListener::bind("127.0.0.1:8123").expect("Unable to bind tcp port 8123");
            let mut server = Server::new(tcp_socket);
            // TODO: Add stuff here in order to:
            // - serve index.html
            // - serve wasm code
            // - build wasm code
            // - watch filesystem for changes
            // - add hot reload to wasm code
            // - figure out a way to test wasm-code in browser
            println!("             ┏━━━━━━━━━━━━━━━━━━━━━━━┓");
            println!("     \x1b[1m\x1b[92mServing\x1b[0m ┃ \x1b[1mhttp://127.0.0.1:8123\x1b[0m ┃ <- Click to open your app! ");
            println!("             ┗━━━━━━━━━━━━━━━━━━━━━━━┛");
            server.listen().expect("Unable to handle incomming connection");
        }
    }
}
