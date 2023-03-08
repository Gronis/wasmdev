use proc_macro2::TokenTree;

pub(crate) struct Attr<T> {
    pub(crate) value: T,
    pub(crate) tt: Option<TokenTree>,
}

impl <T> Attr<T> {
    pub(crate) fn new(value: T, tt: Option<TokenTree>) -> Self {
        Attr{ value, tt }
    }
}

pub(crate) struct Config {
    pub(crate) port: Attr<u16>,
    pub(crate) path: Attr<String>,
    pub(crate) addr: Attr<String>,
    pub(crate) watch: Attr<bool>,
}
