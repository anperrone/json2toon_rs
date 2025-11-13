//! Custom error types for the decoder.

use std::fmt;

/// An error that can occur during the decoding of a TOON string.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// The input string contains invalid indentation at the given line.
    InvalidIndentation { line: usize },
    /// An array header has an invalid format.
    InvalidArrayHeader(String),
    /// An array's actual length does not match its declared length.
    ArrayLengthMismatch { expected: usize, found: usize },
    /// A row in a tabular array has a different number of columns than the header.
    RowWidthMismatch {
        line: usize,
        expected: usize,
        found: usize,
    },
    /// A key-value pair could not be parsed.
    InvalidLine { line: usize, content: String },
    /// An invalid escape sequence was found in a string.
    InvalidEscapeSequence { line: usize, sequence: String },
    /// A generic parsing error.
    ParseError(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidIndentation { line } => {
                write!(f, "Invalid indentation at line {}", line)
            }
            DecodeError::InvalidArrayHeader(msg) => write!(f, "Invalid array header: {}", msg),
            DecodeError::ArrayLengthMismatch { expected, found } => {
                write!(
                    f,
                    "Array length mismatch: expected {}, got {}",
                    expected, found
                )
            }
            DecodeError::RowWidthMismatch {
                line,
                expected,
                found,
            } => write!(
                f,
                "Row width mismatch at line {}: expected {} fields, got {}",
                line, expected, found
            ),
            DecodeError::InvalidLine { line, content } => {
                write!(f, "Invalid line at {}: {}", line, content)
            }
            DecodeError::InvalidEscapeSequence { line, sequence } => {
                write!(
                    f,
                    "Invalid escape sequence at line {}: \\{}",
                    line, sequence
                )
            }
            DecodeError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for DecodeError {}
