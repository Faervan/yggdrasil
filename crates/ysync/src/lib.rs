//! This crate is used as networking library in yggrasil

/// functions and trait imlementations for use with the client side
pub mod client;
/// functions and trait imlementations for use with the server side
pub mod server;

mod tcp_types;
mod udp_types;
pub use tcp_types::*;
pub use udp_types::*;

#[cfg(test)]
mod tests;
