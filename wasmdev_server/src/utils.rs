use std::path::Path;

#[cfg(not(target_family = "wasm"))]
pub use notify::event::Event;
#[cfg(not(target_family = "wasm"))]
pub use notify::{Result, Watcher, EventHandler};
#[cfg(not(target_family = "wasm"))]
pub use wasmdev_core::*;

pub fn simple_hash(bin: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 1u32;
    let mut res = 0u32;
    for byte in bin {
        let ab = a.wrapping_add(b);
        res = res.wrapping_add((*byte as u32).wrapping_mul(ab));
        a = b;
        b = ab;
    }
    res
}

/// This function wraps notify crate with some logic that tries to avoid duplicate events after one-another
/// It also defaults to a Recursive watcher.
#[cfg(not(target_family = "wasm"))]
pub fn make_watcher(path: &Path, mut event_handler: impl EventHandler) -> Option<impl Watcher> {
    use notify::{recommended_watcher, RecursiveMode};
    use notify::event::EventKind;
    use std::sync::{Arc, RwLock};
    use std::sync::mpsc::channel;
    use std::thread;

    let (event_sender, event_receiver) = channel::<Event>();
    let active_event = Arc::new(RwLock::new(None));

    let hash_event = |event: &Event| event.paths.iter()
        .filter_map(|p| p.to_str())
        .map(|s| simple_hash(s.as_bytes()))
        .fold(0u32, |sum, v| sum.wrapping_add(v));

    {
        let active_event = active_event.clone();
        thread::spawn(move || {
            for event in event_receiver.iter() {
                *(active_event.write().unwrap()) = Some(hash_event(&event));
                event_handler.handle_event(Ok(event));
                *(active_event.write().unwrap()) = None;
            }
        });
    }

    let mut last_event = None;
    let mut watcher = recommended_watcher(move |event: Result<Event>| -> () {
        let Ok(event) = event                 else { return };
        let EventKind::Modify(_) = event.kind else { return };

        let active_event_hash = *active_event.read().unwrap();
        let hash = Some(hash_event(&event));

        if active_event_hash == hash                         { return }
        if active_event_hash.is_some() && last_event == hash { return }
        if active_event_hash.is_none() {
            // Ensure we have an active event
            *(active_event.write().unwrap()) = hash;
        }
        last_event = hash;
        let _ = event_sender.send(event);
    }).ok()?;

    watcher.watch(path, RecursiveMode::Recursive).ok()?;
    Some(watcher)
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