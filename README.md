# json2toon_rs

[![Crates.io](https://img.shields.io/crates/v/json2toon_rs.svg)](https://crates.io/crates/json2toon_rs)
[![Documentation](https://docs.rs/json2toon_rs/badge.svg)](https://docs.rs/json2toon_rs)
[![CI](https://github.com/anperrone/json2toon_rs/workflows/CI/badge.svg)](https://github.com/anperrone/json2toon_rs/actions)
[![codecov](https://codecov.io/gh/anperrone/json2toon_rs/graph/badge.svg?token=BDVVEVT65P)](https://codecov.io/gh/anperrone/json2toon_rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Fast, optimized JSON to TOON format converter based on the [TOON v2.0 specification](https://github.com/toon-format/spec/blob/main/SPEC.md).

## What is TOON?

TOON (Token-Oriented Object Notation) is a line-oriented, indentation-based text format that encodes JSON data with explicit structure and minimal quoting. It's particularly efficient for:

- Arrays of uniform objects (tabular data)
- Deterministic, human-readable representations
- LLM prompts and structured data interchange
- Configuration files with nested structures

## Features

- ✅ **Fully spec-compliant** with TOON v2.0
- ↔️ **Bidirectional conversion** - encode JSON to TOON and decode back
- 🚀 **Optimized** for performance with minimal allocations
- 📦 **Zero unsafe code** - fully safe Rust
- ✨ **Robust Error Handling** - provides detailed, structured errors for easier debugging.
- 🎯 **Automatic format detection** - tabular vs expanded arrays
- 🔧 **Configurable** delimiters (comma, tab, pipe)
- ⚙️ **Strict mode** - optional validation of structure and counts
- 🧪 **Comprehensive tests** covering all spec requirements
- 📖 **Well-documented** with inline comments

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
json2toon_rs = "0.1.0"
```

## Quick Start

### Encoding (JSON → TOON)

```rust
use json2toon_rs::{encode, EncoderOptions};
use serde_json::json;

fn main() {
    let data = json!({
        "users": [
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false}
        ]
    });

    let toon = encode(&data, &EncoderOptions::default());
    println!("{}", toon);
}
```

Output:

```
users[2]{id,name,active}:
  1,Alice,true
  2,Bob,false
```

### Decoding (TOON → JSON)

```rust
use json2toon_rs::{decode, DecoderOptions, DecodeError};

fn main() {
    let toon = "users[2]{id,name,active}:\n  1,Alice,true\n  2,Bob,false";

    let json = decode(toon, &DecoderOptions::default()).unwrap();
    println!("{}", serde_json::to_string_pretty(&json).unwrap());

    // Example of error handling
    let invalid_toon = "tags[2]: one,two,three";
    let result = decode(invalid_toon, &DecoderOptions::default());
    assert!(matches!(result, Err(DecodeError::ArrayLengthMismatch { .. })));
}
```

### Round-Trip

```rust
use json2toon_rs::{encode, decode, EncoderOptions, DecoderOptions, DecodeError};
use serde_json::json;

fn main() {
    let original = json!({"name": "Alice", "age": 30});

    // Encode to TOON
    let toon = encode(&original, &EncoderOptions::default());

    // Decode back to JSON
    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();

    assert_eq!(original, decoded); // Perfect round-trip!
}
```

## Examples

### Simple Object

```rust
let data = json!({
    "name": "Alice",
    "age": 30,
    "active": true
});
```

TOON output:

```
name: Alice
age: 30
active: true
```

### Nested Objects

```rust
let data = json!({
    "user": {
        "id": 123,
        "name": "Bob"
    }
});
```

TOON output:

```
user:
  id: 123
  name: Bob
```

### Primitive Arrays

```rust
let data = json!({
    "tags": ["admin", "user", "dev"]
});
```

TOON output:

```
tags[3]: admin,user,dev
```

### Tabular Arrays

Arrays of uniform objects with primitive values are automatically formatted as tables:

```rust
let data = json!({
    "items": [
        {"sku": "A1", "qty": 2, "price": 9.99},
        {"sku": "B2", "qty": 1, "price": 14.50}
    ]
});
```

TOON output:

```
items[2]{sku,qty,price}:
  A1,2,9.99
  B2,1,14.5
```

### Mixed Arrays

Arrays with non-uniform content use expanded list format:

```rust
let data = json!({
    "items": [
        42,
        "text",
        {"key": "value"}
    ]
});
```

TOON output:

```
items[3]:
  - 42
  - text
  - key: value
```

### Custom Delimiters

```rust
use json2toon_rs::{encode, Delimiter, EncoderOptions};

let options = EncoderOptions {
    indent: 2,
    delimiter: Delimiter::Tab,
};

let data = json!({
    "items": [
        {"id": 1, "name": "A"},
        {"id": 2, "name": "B"}
    ]
});

let toon = encode(&data, &options);
```

TOON output (with tabs):

```
items[2	]{id	name}:
  1	A
  2	B
```

## Configuration Options

### Encoder Options

```rust
pub struct EncoderOptions {
    /// Spaces per indentation level (default: 2)
    pub indent: usize,

    /// Document-wide delimiter (default: Comma)
    pub delimiter: Delimiter,
}

pub enum Delimiter {
    Comma,  // Default
    Tab,    // \t
    Pipe,   // |
}
```

### Decoder Options

```rust
pub struct DecoderOptions {
    /// Spaces per indentation level (default: 2)
    pub indent: usize,

    /// Strict mode - enforces counts, indentation, delimiter consistency (default: true)
    pub strict: bool,
}
```

In strict mode, the decoder will:

- Enforce exact indentation multiples
- Validate array/row counts match declared lengths
- Reject invalid escape sequences
- Check delimiter consistency

## Spec Compliance

This implementation follows the TOON v2.0 specification:

- ✅ Canonical number formatting (no exponents, no trailing zeros)
- ✅ Deterministic quoting rules
- ✅ Escape sequences: `\\`, `\"`, `\n`, `\r`, `\t`
- ✅ Tabular array detection
- ✅ Delimiter-aware quoting
- ✅ Object key preservation order
- ✅ UTF-8 support with Unicode and emoji
- ✅ Empty object/array handling
- ✅ Nested structure support

## Testing

Run the test suite:

```bash
cargo test
```

Run examples:

```bash
# Encoding examples
cargo run --example basic

# Decoding examples
cargo run --example decode

# Round-trip examples (encode + decode)
cargo run --example roundtrip
```

## Decoder Features

The decoder implements the complete TOON v2.0 specification:

- **Line-based parsing** with depth tracking
- **Array header parsing** with delimiter detection (`[N]`, `[N	]`, `[N|]`)
- **Tabular format** decoding with field mapping
- **Expanded list format** for mixed arrays
- **Object nesting** with proper depth handling
- **String unescaping** with only valid escapes (`\\`, `\"`, `\n`, `\r`, `\t`)
- **Type inference** (strings, numbers, booleans, null)
- **Quoted string handling** with escape sequence validation
- **Strict mode validation** (optional)
  - Indentation must be exact multiples
  - Array/row counts must match declared lengths
  - Invalid escapes are rejected

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Based on the [TOON v2.0 specification](https://github.com/toon-format/spec/blob/main/SPEC.md)
- Built with [serde](https://serde.rs/) and [serde_json](https://github.com/serde-rs/json)

## References

- [TOON Specification v2.0](https://github.com/toon-format/spec/blob/main/SPEC.md)
- [TOON Format Organization](https://github.com/toon-format)
