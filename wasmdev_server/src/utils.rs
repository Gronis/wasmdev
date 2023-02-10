use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use notify::{recommended_watcher, Watcher, RecursiveMode, Result, EventHandler};
use notify::event::{Event, EventKind, ModifyKind};

pub fn load_file(file_path: &Path) -> Option<Vec<u8>> {
    let mut file_handle = File::open(file_path).ok()?;
    let mut file_contents = Vec::new();
    file_handle.read_to_end(&mut file_contents).ok()?;
    Some(file_contents)
}

#[cfg(not(target_family = "wasm"))]
pub fn build_wasm(input_path: &str, is_release: bool) -> Option<()> {
    use xshell::{Shell, cmd};
    use wasm_bindgen_cli_support::Bindgen;

    let args = if is_release { 
        vec!["--release", "--target", "wasm32-unknown-unknown"]
    } else {
        vec!["--target", "wasm32-unknown-unknown"]
    };
    let sh = Shell::new().expect("Unable to create shell");
    cmd!(sh, "cargo build").args(args).quiet().run().ok()?;
    let mut bind = Bindgen::new();
    bind.input_path(input_path)
        .web(true)
        .map_err(|err| println!("{}", err)).ok()?
        .demangle(is_release)
        .debug(!is_release)
        .remove_name_section(is_release)
        .remove_producers_section(is_release);

    let output_path = Path::new(input_path).parent().expect("No parent when building wasm");
    bind.generate(output_path).map_err(|err| println!("{}", err)).ok()
}

pub fn make_watcher(path: &Path, mut event_handler: impl EventHandler) -> Option<impl Watcher> {
    let mut watcher = recommended_watcher(move |e: Result<Event>| -> () {
        let Ok(event)                 = &e           else { return };
        let EventKind::Modify(modify) = &event.kind  else { return };
        let ModifyKind::Data(_)       = modify       else { return };
        
        event_handler.handle_event(e);
    }).expect("Unable to initiate watcher");
    watcher.watch(path, RecursiveMode::Recursive).ok()?;
    Some(watcher)
}

// TODO; Put somewhere else.
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