//! # json2toon_rs
//!
//! A fast, bidirectional JSON ⟷ TOON converter based on the TOON v2.0 specification.
//!
//! TOON (Token-Oriented Object Notation) is a line-oriented, indentation-based format
//! that efficiently represents JSON data with minimal quoting and explicit structure.
//!
//! This crate provides both encoding (JSON → TOON) and decoding (TOON → JSON) with
//! full spec compliance, automatic format detection, and configurable options.
//!
//! ## Example
//!
//! ```rust
//! use json2toon_rs::{encode, decode, EncoderOptions, DecoderOptions, DecodeError};
//! use serde_json::json;
//!
//! // Encode JSON to TOON
//! let data = json!({
//!     "name": "Alice",
//!     "age": 30,
//!     "tags": ["admin", "user"]
//! });
//!
//! let toon = encode(&data, &EncoderOptions::default());
//! println!("{}", toon);
//!
//! // Decode TOON back to JSON
//! let decoded = decode(&toon, &DecoderOptions::default()).unwrap();
//! assert_eq!(data, decoded);
//! ```

mod common;
mod decoder;
mod encoder;
mod error;

// Re-export public API
pub use common::Delimiter;
pub use decoder::{decode, DecoderOptions};
pub use encoder::{encode, EncoderOptions};
pub use error::DecodeError;
