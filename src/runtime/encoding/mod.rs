//! Encoding passes for runtime APIs.
//!
//! - [`base64`] — Base64 encoding/decoding (atob, btoa)

pub mod base64;

pub use base64::{Atob, Btoa, EncodingGroup};
