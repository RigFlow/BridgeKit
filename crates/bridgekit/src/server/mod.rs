//! Server-side helpers for validating purchases against Apple and Microsoft APIs.

#[cfg(feature = "server-apple")]
pub mod apple;
#[cfg(feature = "server-microsoft")]
pub mod microsoft;
