use proc_macro::TokenStream;
use proc_macro2::{TokenTree, Delimiter, TokenStream as TokenStream2, Span};
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, main_fn: TokenStream) -> TokenStream {

    // We need to convert main function to better work with it.
    let main_fn_wasm: TokenStream2 = main_fn.into();
    
    // Extract the body of the main function.
    let Some(main_fn_wasm_body) = main_fn_wasm.clone().into_iter().find_map(|mut token| {
        let TokenTree::Group(ref mut group) = &mut token else { return None };
        if group.delimiter() == Delimiter::Brace { 
            token.set_span(Span::call_site());
            Some(token)
        } else { None }
    }) else { panic!("function does not have a body." ) };

    // Make the code for the http web server:
    let main_fn_not_wasm: TokenStream2 = make_main_fn_not_wasm(main_fn_wasm_body);

    // dbg!(&_attr);
    // dbg!(&main_fn_wasm);
    // dbg!(&main_fn_not_wasm);

    // Output the rust code for wasm and server main functions:
    quote! {
        #[cfg(not(target_family = "wasm"))]
        #main_fn_not_wasm
        #[cfg(target_family = "wasm")]
        #main_fn_wasm
    }.into()
}

// fn make_main_fn_not_wasm() -> TokenStream2 {
fn make_main_fn_not_wasm(main_fn_wasm_body: TokenTree) -> TokenStream2 {
    let _script = format!("<script>{}</script>", include_str!("index.js"));
    quote!{
        fn main() {
            // non-wasm only imports:
            use std::net::TcpListener;
            use wasmdev::Server;

            // Make sure rust analyzer (code-completion) runs on the wasm code:
            if false #main_fn_wasm_body
            
            // Setup and configure http server:
            let tcp_socket = TcpListener::bind("127.0.0.1:8123").expect("Unable to bind tcp port 8123");
            let mut server = Server::new(tcp_socket);
            // TODO: Add stuff here in order to:
            // - serve index.html
            // - serve wasm code
            // - build wasm code
            // - watch filesystem for changes
            // - add hot reload to wasm code
            // - figure out a way to test wasm-code in browser

            // Tell user how to run the app:
            println!("\x1b[1m\x1b[92m            \x1b[0m ┏\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┓");
            println!("\x1b[1m\x1b[92m     Serving\x1b[0m ┃\x1b[1m http://127.0.0.1:8123 \x1b[0m┃ <- Click to open your app! ");
            println!("\x1b[1m\x1b[92m            \x1b[0m ┗\x1b[0m━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m┛");

            // Start the http server:
            server.listen().expect("Unable to handle incomming connection");
        }
    }
}
