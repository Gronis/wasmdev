use std::{fs, path::Path};

use proc_macro2::{TokenStream, TokenTree, Ident, Punct, Spacing, Group, Delimiter, Literal, Span};

use crate::config::*;

/// Removes all directories in this directory that has no children
pub fn remove_empty_dirs(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                remove_empty_dirs(&path)?;
            } 
        }
        // We have to read dir again after potiential sub-dirs was removed
        if fs::read_dir(path)?.count() == 0 {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}

pub(crate) fn parse_config_attrs(attrs: TokenStream) -> Result<Config, TokenStream> {
    let mut it = attrs.into_iter();
    let mut port = None;
    let mut path = None;
    let mut addr = None;
    let mut watch = None;

    struct NoQuotesError;
    let trim_quotes = |value: &str| -> Result<String, NoQuotesError> {
        if value.as_bytes()[0] != ('"' as u8) || value.as_bytes()[value.as_bytes().len() - 1] != ('"' as u8) { 
            return Err(NoQuotesError);
        }
        Ok(value[1..value.len() - 1].to_string())
    };

    loop {
        
        let Some(TokenTree::Ident(ident)) = it.next() else { 
            break
        };
        let Some(TokenTree::Punct(punct)) = it.next() else { 
            return compiler_error!(ident, "Incomplete attribute list, expected '=' or ':' after '{ident}'")
        };
        
        match punct.to_string().as_str() {
            ":" => (),
            "=" => (),
            _ => return compiler_error!(punct, "Unexpected character '{punct}'. Expected ':' or '='"),
        };
        
        let Some(value) = it.next() else { 
            return compiler_error!(punct, "Incomplete attribute list, expected value after '{punct}'")
        };
        let value_as_str = match &value {
            TokenTree::Literal(value) => value.to_string(),
            TokenTree::Ident(value) => value.to_string(),
            _ => return compiler_error!(value, "Unexpected token: '{value}'"),
        };

        match ident.to_string().as_str() {
            "addr" => { 
                let Ok(val) = trim_quotes(&value_as_str) else {
                    return compiler_error!(value, "Unable to parse addr, {value} is not a `&str`");                    
                };
                addr = Some(Attr::new(val, Some(value.into())));
            },
            "path" => { 
                let Ok(val) = trim_quotes(&value_as_str) else {
                    return compiler_error!(value, "Unable to parse path, {value} is not a `&str`");
                };
                path = Some(Attr::new(val, Some(value.into())));
            },
            "port" => { 
                let Ok(val) = value_as_str.parse() else { 
                    return compiler_error!(value, "Unable to parse port, {value} is not a u16");
                };
                port = Some(Attr::new(val, Some(value.into())));
            },
            "watch" => {
                let Ok(val) = value_as_str.parse() else { 
                    return compiler_error!(value, "Unable to parse watch, {value} is not boolean");
                };
                watch = Some(Attr::new(val, Some(value.into())));
            }
            i  => { 
                return compiler_error!(ident, "Unknown attribute: '{i}', help: available attributes are: 'addr', 'path', 'port' and watch");
            },
        }

        match it.next() {
            Some(TokenTree::Punct(punct)) if punct.to_string() == "," => (),
            None => (),
            Some(tt) => return compiler_error!(tt, "Unexpected character '{tt}', help: use ',' to separate attributes."),
        }
    };
    Ok(Config { 
        port: port.unwrap_or(Attr::new(8080, None)), 
        path: path.unwrap_or(Attr::new("src".into(), None)), 
        addr: addr.unwrap_or(Attr::new("127.0.0.1".into(), None)),
        watch: watch.unwrap_or(Attr::new(true, None)),
    })
}

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
