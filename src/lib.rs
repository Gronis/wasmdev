use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, main_fn: TokenStream) -> TokenStream {
    // dbg!(&_attr);
    // dbg!(&main_fn);

    let main_fn_wasm: TokenStream2 = main_fn.into();
    let main_fn_not_wasm = make_main_fn_not_wasm();

    quote! {
        #[cfg(target_family = "wasm")]
        #main_fn_wasm
        #[cfg(not(target_family = "wasm"))]
        #main_fn_not_wasm
    }.into()
}

fn make_main_fn_not_wasm() -> TokenStream2 {
    quote!{
        fn main() {
            println!("This is the server");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
