pub use wasmdev_macro::main;

#[cfg(not(target_family = "wasm"))]
pub use wasmdev_server::*;

#[cfg(all(target_family = "wasm", feature = "panic_hook"))]
use console_error_panic_hook;

#[cfg(all(target_family = "wasm", feature = "panic_hook"))]
#[inline]
pub fn if_enabled_setup_panic_hook_once() {
    console_error_panic_hook::set_once();
}

#[cfg(not(all(target_family = "wasm", feature = "panic_hook")))]
#[inline]
pub fn if_enabled_setup_panic_hook_once() {}