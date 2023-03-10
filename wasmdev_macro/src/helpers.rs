use proc_macro2::{TokenStream, TokenTree, Ident, Punct, Spacing, Group, Delimiter, Literal, Span};

pub(crate) fn get_fn_name(func: &TokenStream) -> Option<TokenTree> {
    let mut it = func.clone().into_iter().skip_while(|tt| {
        let TokenTree::Ident(ident) = tt else { return true };
        ident.to_string() != "fn"
    });
    it.next(); // Skip "fn" identifier
    it.next()  // Should be function name identifier
}

pub(crate) fn emit_compilation_error(msg: &str, span: &Span) -> TokenStream {
    let span = span.clone();
    TokenStream::from_iter(vec![
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct({
            let mut punct = Punct::new('!', Spacing::Alone);
            punct.set_span(span);
            punct
        }),
        TokenTree::Group({
            let mut group = Group::new(Delimiter::Brace, {
                TokenStream::from_iter(vec![TokenTree::Literal({
                    let mut string = Literal::string(msg);
                    string.set_span(span);
                    string
                })])
            });
            group.set_span(span);
            group
        }),
    ])
}

/// This trait is only here so that "compiler_error" macro works for both TokenTrees and Spans.
pub(crate) trait IntoSpanSelf {
    fn span(self) -> Span;
}

impl IntoSpanSelf for Span {
    fn span(self) -> Span {
        self
    }
}

macro_rules! compiler_error { 
    ( $i:ident, $($args:tt)* ) => { 
        {
            let span = ($i).span();
            Err(emit_compilation_error(&format!($($args)*), &span))
        }
    };
    ( $($args:tt)* ) => { 
        {
            let span = Span::call_site();
            Err(emit_compilation_error(&format!($($args)*), &span))
        }
    };
}

pub(crate) use compiler_error;
