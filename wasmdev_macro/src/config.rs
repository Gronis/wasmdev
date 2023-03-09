use std::{env, str::from_utf8, fs, collections::HashSet};

use proc_macro2::{TokenStream, TokenTree, Span};
use quote::quote;

use crate::util::*;

pub(crate) struct Attr<T> {
    pub(crate) value: T,
    pub(crate) tt: Option<TokenTree>,
}

impl <T> Attr<T> {
    pub(crate) fn new(value: T, tt: Option<TokenTree>) -> Self {
        Attr{ value, tt }
    }
}

pub(crate) struct AttrConfig {
    pub(crate) port: Attr<u16>,
    pub(crate) path: Attr<String>,
    pub(crate) addr: Attr<String>,
    pub(crate) watch: Attr<bool>,
}

pub(crate) struct BuildConfig {
    pub(crate) proj_name: String,
    pub(crate) is_release: bool,
    pub(crate) attrs: AttrConfig,
    pub(crate) index_js: String,
    pub(crate) index_html: String,
    pub(crate) target_path: String,
    pub(crate) wasm_path: String,
    pub(crate) index_js_path: String,
    pub(crate) index_wasm_path: String,
    pub(crate) proj_html_path: String,
    pub(crate) proj_static_path: String,
    pub(crate) proj_src_path: String,
}

impl TryInto<BuildConfig> for AttrConfig {
    type Error = TokenStream;
    fn try_into(self) -> Result<BuildConfig, Self::Error> {
        let Ok(proj_dir) = env::var("CARGO_MANIFEST_DIR") else {
            return compiler_error!("Cargo did not set env var: CARGO_MANIFEST_DIR")
        };
        let Ok(proj_name) = env::var("CARGO_PKG_NAME") else {
            return compiler_error!("Cargo did not set env var: CARGO_PKG_NAME");
        };
    
        let is_release       = !cfg!(debug_assertions);
        let index_js         = include_str!("index.js");
        let index_html       = include_str!("index.html");
        let release_mode     = if is_release {"release"} else {"debug"};
        let index_js         = if is_release {index_js.split("// -- debug -- \\").next().unwrap()} else {index_js};
        let index_html       = format!("{index_html}\n<script type=\"module\">{index_js}</script>"); 
        let target_path      = format!("target/wasmdev-build-cache");
        let out_path         = format!("{target_path}/wasm32-unknown-unknown");
        let wasm_path        = format!("{out_path}/{release_mode}/{proj_name}.wasm");
        let index_js_path    = format!("{out_path}/{release_mode}/{proj_name}.js");
        let index_wasm_path  = format!("{out_path}/{release_mode}/{proj_name}_bg.wasm");
        let proj_html_path   = format!("{proj_dir}/{}/index.html", &self.path.value);
        let proj_static_path = format!("{proj_dir}/{}", &self.path.value);
        let proj_src_path    = format!("{proj_dir}/src");
    
        Ok(BuildConfig {
            attrs: self,
            index_html,
            index_js: index_js.into(),
            index_js_path,
            index_wasm_path,
            is_release,
            proj_html_path,
            proj_name,
            proj_static_path,
            proj_src_path,
            target_path,
            wasm_path,
        })
    }
}

/// Build all web assets and put it in target/dist/{proj_name}
#[cfg(not(target_family = "wasm"))]
pub(crate) fn build_all_web_assets(config: &BuildConfig) -> Result<TokenStream, TokenStream> {
    use wasmdev_core::{build_wasm, minify_javascript, find_files};
    
    let Some(_)       = build_wasm(&config.wasm_path, config.is_release, &config.target_path)
                            else { return compiler_error!("Failed to build wasm target") };
    let Ok(wasm_code) = fs::read(&config.index_wasm_path)
                            else { return compiler_error!("Failed to read wasm code from {}", config.index_wasm_path) };
    let Ok(js_code)   = fs::read(&config.index_js_path)
                            else { return compiler_error!("Failed to read js code from {}", config.index_js_path) };
    let js_code       = minify_javascript(&js_code);
    let dist_path     = &format!("target/dist/{}", config.proj_name);
    let html_code = (|| -> Option<String>{
        let html_code = fs::read(&config.proj_html_path).ok()?;
        let html_code = from_utf8(&html_code).ok()?;
        Some(format!("{}\n<script type=\"module\">{}</script>", html_code, config.index_js))
    })().unwrap_or(config.index_html.clone());

    match (|| -> Result<TokenStream, std::io::Error> {
        let _ = fs::create_dir_all(dist_path);
        fs::write(format!("{dist_path}/index.wasm"), wasm_code)?;
        fs::write(format!("{dist_path}/index.js"), js_code)?;
        fs::write(format!("{dist_path}/index.html"), html_code)?;

        let file_paths = find_files(&config.proj_static_path);
        let file_path_iter = file_paths.iter()
            .filter_map(|p| p.to_str())
            .filter(|p| !p.ends_with(".rs"))          // Don't export src files.
            .filter(|p| !p.ends_with("/index.html")); // index.html already handled.

        // Clean up old files that were removed since last build:
        {
            let old_files = find_files(&dist_path);
            let mut old_file_paths: HashSet<_> = old_files.iter()
                .filter_map(|p| p.to_str())
                .map(|p| p.to_string().replace(dist_path, ""))
                .filter(|p| !p.ends_with("/index.wasm"))
                .filter(|p| !p.ends_with("/index.js"))
                .filter(|p| !p.ends_with("/index.html"))
                .collect();

            for file_path in file_path_iter.clone() {
                old_file_paths.remove(&file_path.replace(&config.proj_static_path, ""));
            }
            let files_to_remove = old_file_paths.iter().map(|p| format!("{dist_path}{p}"));
            for file_path in files_to_remove {
                fs::remove_file(file_path)?;
            }
            remove_empty_dirs(&dist_path)?;
        }

        for file_path in file_path_iter {
            let file_contents = fs::read(file_path)?;
            let file_contents = if file_path.ends_with(".js") { 
                minify_javascript(&file_contents)
            } else { file_contents };
            let file_rel_path = file_path.replace(&config.proj_static_path, "");
            let file_dist_path = format!("{dist_path}/{file_rel_path}");
            create_parent_dir_all(&file_dist_path)?;
            fs::write(file_dist_path, file_contents)?;
        }
        // Abuse "include_bytes" to make sure static web assets invalidate cargo build cache
        let tt_invalidate_static_asset_cache = TokenStream::from_iter(file_paths.iter()
            .filter_map(|p| p.to_str())
            .filter(|p| !p.ends_with(".rs"))
            .map(|p| quote!{ include_bytes!(#p); })
        );
        eprintln!("\x1b[1m\x1b[92m    Finished\x1b[0m release artifacts in: '{dist_path}'");
        Ok(tt_invalidate_static_asset_cache)
    })() {
        Ok(tt)   => Ok(tt),
        Err(msg) => return compiler_error!("Failed to build project '{}' , {msg}", config.proj_name),
    }
}

pub(crate) fn parse_config_attrs(attrs: TokenStream) -> Result<AttrConfig, TokenStream> {
    let mut it = attrs.into_iter();
    let mut port = None;
    let mut path = None;
    let mut addr = None;
    let mut watch = None;

    struct NoQuotesError;
    let trim_quotes = |value: &str| -> Result<String, NoQuotesError> {
        if value.as_bytes()[0] != b'"' || value.as_bytes()[value.as_bytes().len() - 1] != b'"' { 
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
                return compiler_error!(ident, "Unknown attribute: '{i}', help: available attributes are: 'addr', 'path', 'port' and 'watch'");
            },
        }

        match it.next() {
            Some(TokenTree::Punct(punct)) if punct.to_string() == "," => (),
            None => (),
            Some(tt) => return compiler_error!(tt, "Unexpected character '{tt}', help: use ',' to separate attributes."),
        }
    };
    Ok(AttrConfig { 
        port: port.unwrap_or(Attr::new(8080, None)), 
        path: path.unwrap_or(Attr::new("src".into(), None)), 
        addr: addr.unwrap_or(Attr::new("127.0.0.1".into(), None)),
        watch: watch.unwrap_or(Attr::new(true, None)),
    })
}