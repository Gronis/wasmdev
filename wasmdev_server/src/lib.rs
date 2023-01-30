

#[cfg(not(target_family = "wasm"))]
pub mod http;
#[cfg(not(target_family = "wasm"))]
pub mod utils;

#[cfg(not(target_family = "wasm"))]
pub use http::{Server, ServerConfig};

#[cfg(not(target_family = "wasm"))]
pub mod prelude {
    pub use crate::http::{EndpointWithoutContentBuilder, EndpointAnyBuilder};
}
