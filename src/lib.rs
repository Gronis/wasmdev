use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    // dbg!(&_attr);
    // dbg!(&_item);
    _item
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
