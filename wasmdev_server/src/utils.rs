use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[cfg(not(target_family = "wasm"))]
pub use notify::event::Event;
#[cfg(not(target_family = "wasm"))]
pub use notify::{Result, Watcher, EventHandler};

pub fn load_file(file_path: &Path) -> Option<Vec<u8>> {
    let mut file_handle = File::open(file_path).ok()?;
    let mut file_contents = Vec::new();
    file_handle.read_to_end(&mut file_contents).ok()?;
    Some(file_contents)
}

#[cfg(not(target_family = "wasm"))]
pub fn build_wasm(input_path: &str, is_release: bool, target_dir: &str) -> Option<()> {
    use xshell::{Shell, cmd};
    use wasm_bindgen_cli_support::Bindgen;

    let mut args = vec![
        "--target", "wasm32-unknown-unknown",
        "--target-dir", target_dir,
        "--color", "always",
    ];
    if is_release { args.push("--release") };
    let args = args; // Remove mutability;
    let sh = Shell::new().expect("Unable to create shell");
    {
        // This lets wasmdev::main know if cargo was started from within wasmdev::main
        let _env_guard = sh.push_env("CARGO_WASMDEV", "1");
        cmd!(sh, "cargo build").args(args).quiet().run().ok()?;
    }
    let output_path = Path::new(input_path).parent().expect("No parent when building wasm");
    Bindgen::new()
        .input_path(input_path)
        .web(true)
        .map_err(|err| println!("{}", err)).ok()?
        .demangle(!is_release)
        .debug(!is_release)
        .remove_name_section(is_release)
        .remove_producers_section(is_release)
        .generate(output_path).map_err(|err| println!("{}", err)).ok()
}

#[cfg(not(target_family = "wasm"))]
pub fn minify_javascript(code_in: &[u8]) -> Vec<u8>{
    use minify_js::{Session, TopLevelMode, minify};
    let session = Session::new();
    let mut code_out = vec![];
    minify(&session, TopLevelMode::Module, code_in, &mut code_out).unwrap();
    code_out
} 

#[cfg(not(target_family = "wasm"))]
pub fn make_watcher(path: &Path, mut event_handler: impl EventHandler) -> Option<impl Watcher> {
    use notify::{recommended_watcher, RecursiveMode};
    use notify::event::{EventKind, ModifyKind};

    let mut watcher = recommended_watcher(move |e: Result<Event>| -> () {
        let Ok(event)                 = &e           else { return };
        let EventKind::Modify(modify) = &event.kind  else { return };
        let ModifyKind::Data(_)       = modify       else { return };
        
        event_handler.handle_event(e);
    }).expect("Unable to initiate watcher");
    watcher.watch(path, RecursiveMode::Recursive).ok()?;
    Some(watcher)
}

pub fn find_files(path: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    let mut paths = vec![path.to_path_buf()];
    use std::io;
    use std::fs::{self};
    let mut traverse = |paths_in: &mut Vec<PathBuf>| -> io::Result<Vec<PathBuf>> {
        let mut paths_out = vec![];
        for path in paths_in.drain(..) {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    paths_out.push(path);
                } else {
                    files.push(path);
                }
            }
        }
        Ok(paths_out)
    };

    // Recurse 3 layers down
    let Ok(mut paths) = traverse(&mut paths) else { return vec![] };
    let Ok(mut paths) = traverse(&mut paths) else { return vec![] };
    let Ok(_)         = traverse(&mut paths) else { return vec![] };

    files
}

pub struct Deferred <T: Fn() -> ()>{
    pub f: T,
}

impl<T: Fn() -> ()> Drop for Deferred<T> {
    fn drop(&mut self) {
       let s: &Self = self;
       let f = &(s.f);
       f();
    }
}

macro_rules! defer_expr { ($e: expr) => { $e } } // tt hack
macro_rules! defer {
    ( $($s:tt)* ) => {
        let _deferred = crate::utils::Deferred { f: || {
            crate::utils::defer_expr!({ $($s)* })
        }}; 
    };
    () => {};
}

pub(crate) use defer_expr;
pub(crate) use defer;