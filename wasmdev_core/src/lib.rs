
#[cfg(not(target_family = "wasm"))]
mod utils;

#[cfg(not(target_family = "wasm"))]
pub use utils::*;
