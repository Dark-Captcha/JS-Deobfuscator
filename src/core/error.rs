//! Error types for the deobfuscation engine.

use thiserror::Error;

// ============================================================================
// DeobError
// ============================================================================

/// Main error type for the deobfuscator.
#[derive(Debug, Error)]
pub enum DeobError {
    /// Parsing failed.
    #[error("parse error: {0}")]
    Parse(String),

    /// Transform error.
    #[error("transform error: {0}")]
    Transform(String),

    /// Pass-specific error.
    #[error("pass error in {pass}: {message}")]
    Pass {
        /// Name of the pass that failed.
        pass: &'static str,
        /// Error message.
        message: String,
    },

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Max iterations reached without convergence.
    #[error("max iterations reached without convergence")]
    MaxIterations,
}

impl DeobError {
    /// Create a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    /// Create a transform error.
    pub fn transform(message: impl Into<String>) -> Self {
        Self::Transform(message.into())
    }

    /// Create a pass error.
    pub fn pass(pass: &'static str, message: impl Into<String>) -> Self {
        Self::Pass {
            pass,
            message: message.into(),
        }
    }

    /// Create a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

// ============================================================================
// PassError
// ============================================================================

/// Error from a specific pass.
///
/// Used for non-fatal errors that should be collected but not stop execution.
#[derive(Debug, Clone)]
pub struct PassError {
    /// Name of the pass that encountered the error.
    pub pass: &'static str,

    /// Error message.
    pub message: String,

    /// Whether the error is recoverable.
    pub recoverable: bool,
}

impl PassError {
    /// Create a new pass error.
    pub fn new(pass: &'static str, message: impl Into<String>) -> Self {
        Self {
            pass,
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a non-recoverable pass error.
    pub fn fatal(pass: &'static str, message: impl Into<String>) -> Self {
        Self {
            pass,
            message: message.into(),
            recoverable: false,
        }
    }
}

impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.pass, self.message)
    }
}
