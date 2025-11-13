//! Common types and utilities shared between encoder and decoder

/// Delimiter type for separating array values and tabular rows
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delimiter {
    Comma,
    Tab,
    Pipe,
}

impl Delimiter {
    /// Returns the character representation
    pub(crate) fn as_char(&self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }

    /// Returns the header symbol (empty for comma, actual char for tab/pipe)
    pub(crate) fn header_symbol(&self) -> &str {
        match self {
            Delimiter::Comma => "",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }
}
