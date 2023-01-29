use std::fs::File;
use std::io::prelude::*;
use xshell::{Shell, cmd};
use std::path::Path;
use notify::{recommended_watcher, Watcher, RecursiveMode, Result, EventHandler};
use notify::event::{Event, EventKind, ModifyKind};

pub fn load_file(file_path: &Path) -> Option<Vec<u8>> {
    let mut file_handle = File::open(file_path).ok()?;
    let mut file_contents = Vec::new();
    file_handle.read_to_end(&mut file_contents).ok()?;
    Some(file_contents)
}

pub fn build_wasm() -> Option<()> {
    let sh = Shell::new().expect("Unable to create shell");
    cmd!(sh, "cargo build --target wasm32-unknown-unknown").quiet().run().ok()
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