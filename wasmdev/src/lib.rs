pub use wasmdev_macro::main;

#[cfg(not(target_family = "wasm"))]
pub use wasmdev_server::*;

#[cfg(target_family = "wasm")]
pub use console_error_panic_hook;