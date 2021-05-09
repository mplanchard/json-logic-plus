//! Error handling
//!
use serde_json::Value;
use thiserror;

use crate::op::NumParams;

/// Public error enumeration
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid data - value: {value:?}, reason: {reason:?}")]
    InvalidData { value: Value, reason: String },

    #[error("Invalid identifier '{0}': identifiers must be valid utf-8 of 2 or more characters, containing no whitespace")]
    InvalidIdentifier(String),

    #[error("Invalid rule - operator: '{key:?}', reason: {reason:?}")]
    InvalidOperation { key: String, reason: String },

    #[error("Invalid variable - '{value:?}', reason: {reason:?}")]
    InvalidVariable { value: Value, reason: String },

    #[error("Invalid variable key - '{value:?}', reason: {reason:?}")]
    InvalidVariableKey { value: Value, reason: String },

    #[error("Invalid argument for '{operation}' - '{value:?}', reason: {reason}")]
    InvalidArgument {
        value: Value,
        operation: &'static str,
        reason: String,
    },

    #[error("Invalid variable mapping - {0} is not an object.")]
    InvalidVarMap(Value),

    #[error("Overflow error during operation: '{0}' on values '{1}' and '{2}'")]
    OverflowBinaryOp(&'static str, String, String),

    #[error("Encountered an unexpected error. Please raise an issue on GitHub and include the following error message: {0}")]
    UnexpectedError(String),

    #[error("Wrong argument count - expected: {expected:?}, actual: {actual:?}")]
    WrongArgumentCount { expected: NumParams, actual: usize },
}
impl Error {
    pub(crate) fn invalid_argument<S: Into<String>>(
        value: Value,
        operation: &'static str,
        reason: S,
    ) -> Self {
        Self::InvalidArgument {
            value,
            operation,
            reason: reason.into(),
        }
    }
    pub(crate) fn wrong_argument_count(expected: NumParams, actual: usize) -> Self {
        Self::WrongArgumentCount { expected, actual }
    }
}
