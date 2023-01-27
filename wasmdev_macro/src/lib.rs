use proc_macro::TokenStream;
use proc_macro2::{TokenTree, Delimiter, TokenStream as TokenStream2};
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

fn clone_fn_body(function: &TokenStream2) -> Option<TokenTree> {
    function.clone().into_iter().find_map(|token| {
        let TokenTree::Group(group) = &token else { return None };
        if group.delimiter() == Delimiter::Brace { Some(token) } else { None }
    })
}

fn make_server_main_fn(wasm_main_fn: &TokenStream2) -> TokenStream2 {
    let Some(wasm_body) = clone_fn_body(&wasm_main_fn) else { panic!("function does not have a body.") };
    let _script = format!("<script>{}</script>", include_str!("index.js"));
    quote!{
        fn main() {
            use std::net::TcpListener;
            use wasmdev::Server;

            // Make sure rust analyzer (code-completion) analyze the wasm code:
            if false #wasm_body

            let tcp_socket = TcpListener::bind("127.0.0.1:8123").expect("Unable to bind tcp port 8123");
            let mut server = Server::new(tcp_socket);

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
