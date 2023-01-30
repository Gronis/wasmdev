pub use wasmdev_macro::main;
#[cfg(not(target_family = "wasm"))]
pub use wasmdev_server::*;