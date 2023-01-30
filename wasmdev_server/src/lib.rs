pub mod http;
pub mod utils;

pub use http::{Server, ServerConfig};
pub use http::{EndpointWithoutContentBuilder, EndpointAnyBuilder};

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
