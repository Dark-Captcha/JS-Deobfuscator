//! Deprecated runtime API passes.
//!
//! - [`escape`] — URL escape/unescape (deprecated but still used by obfuscators)

pub mod escape;

pub use escape::{DeprecatedGroup, Escape, Unescape};
