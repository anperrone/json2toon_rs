//! JSON to TOON encoder implementation

use crate::common::Delimiter;
use serde_json::Value;

/// Encoder configuration options
#[derive(Debug, Clone)]
pub struct EncoderOptions {
    /// Spaces per indentation level (default: 2)
    pub indent: usize,
    /// Document-wide delimiter for quoting decisions (default: Comma)
    pub delimiter: Delimiter,
}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self {
            indent: 2,
            delimiter: Delimiter::Comma,
        }
    }
}

/// Encode a JSON value to TOON format
pub fn encode(value: &Value, options: &EncoderOptions) -> String {
    let mut encoder = Encoder::new(options);
    encoder.encode_value(value, 0);
    encoder.output
}

struct Encoder<'a> {
    options: &'a EncoderOptions,
    output: String,
}

impl<'a> Encoder<'a> {
    fn new(options: &'a EncoderOptions) -> Self {
        Self {
            options,
            output: String::new(),
        }
    }

    /// Main encoding entry point
    fn encode_value(&mut self, value: &Value, depth: usize) {
        match value {
            Value::Object(obj) if obj.is_empty() => {
                // Empty object at root = empty document
                if depth == 0 {
                    // No output for root empty object
                } else {
                    // Empty nested object handled elsewhere
                }
            }
            Value::Object(obj) => self.encode_object(obj, depth),
            Value::Array(arr) => self.encode_array(arr, depth, None),
            Value::Null => self.output.push_str("null"),
            Value::Bool(b) => self.output.push_str(&b.to_string()),
            Value::Number(n) => self.output.push_str(&self.normalize_number(n)),
            Value::String(s) => self
                .output
                .push_str(&self.quote_string(s, self.options.delimiter)),
        }
    }

    /// Encode an object
    fn encode_object(&mut self, obj: &serde_json::Map<String, Value>, depth: usize) {
        for (i, (key, value)) in obj.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            } else if depth > 0 {
                // First field at non-root depth (don't add newline before first field at root)
                self.output.push('\n');
            }
            self.indent(depth);
            self.output.push_str(&self.encode_key(key));

            match value {
                Value::Object(nested) if nested.is_empty() => {
                    // Empty nested object: key:
                    self.output.push(':');
                    continue;
                }
                Value::Object(nested) => {
                    // Nested object: key:
                    self.output.push(':');
                    // Children will add their own leading newline
                    self.encode_object(nested, depth + 1);
                }
                Value::Array(arr) => {
                    // Array as object field: key[N]:
                    // Don't write colon yet - array header includes it
                    self.encode_array_after_key(arr, depth);
                }
                _ => {
                    // Primitive value: key: value
                    self.output.push(':');
                    self.output.push(' ');
                    self.encode_primitive(value, self.options.delimiter);
                }
            }
        }
    }

    /// Encode array when key has already been written (e.g., "key:")
    fn encode_array_after_key(&mut self, arr: &[Value], depth: usize) {
        let len = arr.len();
        let delim = self.options.delimiter;

        // Check if array qualifies for tabular format
        if let Some(fields) = self.detect_tabular(arr) {
            // Tabular format: key[N]{f1,f2,...}:
            self.write_array_header(len, delim, Some(&fields));

            for obj in arr.iter() {
                self.output.push('\n');
                self.indent(depth + 1);

                if let Value::Object(map) = obj {
                    // Write values in field order
                    for (j, field) in fields.iter().enumerate() {
                        if j > 0 {
                            self.output.push(delim.as_char());
                        }
                        if let Some(val) = map.get(field) {
                            self.output.push_str(&self.quote_primitive(val, delim));
                        }
                    }
                }
            }
        } else if self.is_inline_primitive_array(arr) {
            // Inline primitive array: key[N]: v1,v2,...
            self.write_array_header(len, delim, None);

            if !arr.is_empty() {
                self.output.push(' ');
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        self.output.push(delim.as_char());
                    }
                    self.output.push_str(&self.quote_primitive(val, delim));
                }
            }
        } else {
            // Expanded list format: key[N]:
            self.write_array_header(len, delim, None);

            for item in arr {
                self.output.push('\n');
                self.indent(depth + 1);
                self.output.push_str("- ");

                match item {
                    Value::Array(inner) => {
                        // Nested inline array
                        self.write_array_header(inner.len(), delim, None);
                        if !inner.is_empty() {
                            self.output.push(' ');
                            for (i, val) in inner.iter().enumerate() {
                                if i > 0 {
                                    self.output.push(delim.as_char());
                                }
                                self.output.push_str(&self.quote_primitive(val, delim));
                            }
                        }
                    }
                    Value::Object(obj) => {
                        // Object as list item
                        self.encode_object_as_list_item(obj, depth + 1);
                    }
                    _ => {
                        // Primitive list item
                        self.encode_primitive(item, delim);
                    }
                }
            }
        }
    }

    /// Encode an array at root level (no key prefix)
    /// This delegates to encode_array_after_key since the logic is identical
    /// for both root-level and field-level arrays
    fn encode_array(&mut self, arr: &[Value], depth: usize, _key: Option<&str>) {
        self.encode_array_after_key(arr, depth);
    }

    /// Encode object as a list item (first field on hyphen line)
    fn encode_object_as_list_item(&mut self, obj: &serde_json::Map<String, Value>, depth: usize) {
        let mut first = true;

        for (key, value) in obj.iter() {
            if !first {
                self.output.push('\n');
                self.indent(depth);
            }

            if first {
                // First field on hyphen line
                first = false;
            }

            self.output.push_str(&self.encode_key(key));
            self.output.push(':');

            match value {
                Value::Object(nested) if nested.is_empty() => {
                    continue;
                }
                Value::Object(nested) => {
                    self.output.push('\n');
                    self.encode_object(nested, if first { depth + 2 } else { depth + 1 });
                }
                Value::Array(arr) => {
                    // Array as object field in list item
                    self.encode_array_after_key(arr, depth);
                }
                _ => {
                    self.output.push(' ');
                    self.encode_primitive(value, self.options.delimiter);
                }
            }
        }
    }

    /// Check if array should use inline format (all same primitive type)
    fn is_inline_primitive_array(&self, arr: &[Value]) -> bool {
        if arr.is_empty() {
            return true;
        }

        // All must be primitives
        if !arr.iter().all(|v| {
            matches!(
                v,
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
            )
        }) {
            return false;
        }

        // Check if all are the same type (more strict)
        // Numbers and strings are commonly mixed, so allow those
        let has_number = arr.iter().any(|v| v.is_number());
        let has_string = arr.iter().any(|v| v.is_string());
        let has_bool = arr.iter().any(|v| v.is_boolean());
        let has_null = arr.iter().any(|v| v.is_null());

        // Count how many different types we have
        let type_count = [has_number, has_string, has_bool, has_null]
            .iter()
            .filter(|&&x| x)
            .count();

        // Only use inline format if all same type (1 type)
        type_count == 1
    }

    /// Detect if array qualifies for tabular format
    fn detect_tabular(&self, arr: &[Value]) -> Option<Vec<String>> {
        if arr.is_empty() {
            return None;
        }

        // All elements must be objects
        let objects: Vec<_> = arr.iter().filter_map(|v| v.as_object()).collect();
        if objects.len() != arr.len() {
            return None;
        }

        // Get field names from first object
        let first = objects[0];
        let fields: Vec<String> = first.keys().cloned().collect();

        // All objects must have same keys and all values must be primitives
        for obj in &objects {
            if obj.len() != fields.len() {
                return None;
            }
            for field in &fields {
                let value = obj.get(field)?;
                // Must be primitive (not object or array)
                if !matches!(
                    value,
                    Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
                ) {
                    return None;
                }
            }
        }

        Some(fields)
    }

    /// Write array header: `[N<delim>]` or `[N<delim>]{fields}:`
    fn write_array_header(&mut self, len: usize, delim: Delimiter, fields: Option<&[String]>) {
        self.output.push('[');
        self.output.push_str(&len.to_string());
        self.output.push_str(delim.header_symbol());
        self.output.push(']');
        if let Some(fields) = fields {
            self.output.push('{');
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    self.output.push(delim.as_char());
                }
                self.output.push_str(&self.encode_key(field));
            }
            self.output.push('}');
        }

        self.output.push(':');
    }

    /// Encode a key (with quoting if needed)
    fn encode_key(&self, key: &str) -> String {
        // Keys must be quoted unless they match: ^[A-Za-z_][A-Za-z0-9_.]*$
        let needs_quoting = key.is_empty()
            || (!key.chars().next().unwrap().is_ascii_alphabetic() && !key.starts_with('_'))
            || !key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');

        if needs_quoting {
            self.quote_and_escape(key)
        } else {
            key.to_string()
        }
    }

    /// Encode primitive with delimiter-aware quoting
    fn encode_primitive(&mut self, value: &Value, delim: Delimiter) {
        self.output.push_str(&self.quote_primitive(value, delim));
    }

    /// Quote primitive value with delimiter awareness
    fn quote_primitive(&self, value: &Value, delim: Delimiter) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => self.normalize_number(n),
            Value::String(s) => self.quote_string(s, delim),
            _ => String::new(),
        }
    }

    /// Quote string with delimiter-aware rules (§7.2)
    fn quote_string(&self, s: &str, delim: Delimiter) -> String {
        let needs_quoting = s.is_empty()
            || s.starts_with(' ')
            || s.ends_with(' ')
            || s == "true"
            || s == "false"
            || s == "null"
            || s == "-"
            || s.starts_with('-')
            || s.contains(':')
            || s.contains('"')
            || s.contains('\\')
            || s.contains('[')
            || s.contains(']')
            || s.contains('{')
            || s.contains('}')
            || s.contains('\n')
            || s.contains('\r')
            || s.contains('\t')
            || s.contains(delim.as_char())
            || self.is_numeric_like(s);

        if needs_quoting {
            self.quote_and_escape(s)
        } else {
            s.to_string()
        }
    }

    /// Check if string looks like a number
    /// Returns true if the string could be parsed as a numeric value,
    /// including edge cases like leading zeros (e.g., "007", "0123")
    fn is_numeric_like(&self, s: &str) -> bool {
        // Check for leading zeros which need quoting per TOON spec
        if s.starts_with('0') && s.len() > 1 {
            if let Some(second_char) = s.chars().nth(1) {
                // "0." and "-0" are valid, but "05", "0123" etc. need quoting
                if second_char.is_ascii_digit() {
                    return true;
                }
            }
        }

        // Use standard library parsing to detect numeric patterns
        // This handles integers, floats, scientific notation, etc.
        s.parse::<f64>().is_ok()
    }

    /// Quote and escape a string (§7.1)
    fn quote_and_escape(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 2);
        result.push('"');

        for c in s.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(c),
            }
        }

        result.push('"');
        result
    }

    /// Normalize number to canonical form (§2)
    /// Converts numbers to TOON-compliant format without scientific notation
    fn normalize_number(&self, n: &serde_json::Number) -> String {
        if let Some(i) = n.as_i64() {
            i.to_string()
        } else if let Some(u) = n.as_u64() {
            u.to_string()
        } else if let Some(f) = n.as_f64() {
            // Handle special cases - convert to null per TOON spec
            if f.is_nan() || f.is_infinite() {
                // Note: NaN and Infinity are not valid in TOON, converting to null
                return "null".to_string();
            }

            // Normalize -0 to 0
            if f == 0.0 {
                return "0".to_string();
            }

            // Format the number, then convert scientific notation if present
            let mut s = n.to_string();

            // Convert scientific notation (e.g., "1.5e10") to decimal form
            if s.contains('e') || s.contains('E') {
                // Parse and reformat without scientific notation
                // For very large/small numbers, this may produce long strings
                s = format!("{:.}", f);
            }

            // Remove trailing zeros after decimal point
            if s.contains('.') {
                let trimmed = s.trim_end_matches('0');
                if let Some(stripped) = trimmed.strip_suffix('.') {
                    stripped.to_string()
                } else {
                    trimmed.to_string()
                }
            } else {
                s
            }
        } else {
            n.to_string()
        }
    }

    /// Write indentation
    fn indent(&mut self, depth: usize) {
        for _ in 0..(depth * self.options.indent) {
            self.output.push(' ');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_object() {
        let data = json!({});
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "");
    }

    #[test]
    fn test_simple_object() {
        let data = json!({
            "name": "Alice",
            "age": 30
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "name: Alice\nage: 30");
    }

    #[test]
    fn test_nested_object() {
        let data = json!({
            "user": {
                "name": "Bob",
                "id": 123
            }
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "user:\n  name: Bob\n  id: 123");
    }

    #[test]
    fn test_primitive_array() {
        let data = json!({
            "tags": ["admin", "user", "dev"]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "tags[3]: admin,user,dev");
    }

    #[test]
    fn test_empty_array() {
        let data = json!({
            "items": []
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "items[0]:");
    }

    #[test]
    fn test_tabular_array() {
        let data = json!({
            "users": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false}
            ]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(
            result,
            "users[2]{id,name,active}:\n  1,Alice,true\n  2,Bob,false"
        );
    }

    #[test]
    fn test_mixed_array() {
        let data = json!({
            "items": [1, "text", true]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "items[3]:\n  - 1\n  - text\n  - true");
    }

    #[test]
    fn test_array_of_objects() {
        let data = json!({
            "items": [
                {"id": 1, "name": "First"},
                {"id": 2, "name": "Second"}
            ]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "items[2]{id,name}:\n  1,First\n  2,Second");
    }

    #[test]
    fn test_quoting_colon() {
        let data = json!({
            "url": "http://example.com:8080"
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "url: \"http://example.com:8080\"");
    }

    #[test]
    fn test_quoting_comma() {
        let data = json!({
            "tags": ["a,b", "c"]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "tags[2]: \"a,b\",c");
    }

    #[test]
    fn test_quoting_reserved_words() {
        let data = json!({
            "values": ["true", "false", "null"]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "values[3]: \"true\",\"false\",\"null\"");
    }

    #[test]
    fn test_number_normalization() {
        let data = json!({
            "int": 42,
            "float": 3.14,
            "negative": -100
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "int: 42\nfloat: 3.14\nnegative: -100");
    }

    #[test]
    fn test_tab_delimiter() {
        let data = json!({
            "items": [
                {"id": 1, "name": "A"},
                {"id": 2, "name": "B"}
            ]
        });
        let options = EncoderOptions {
            indent: 2,
            delimiter: Delimiter::Tab,
        };
        let result = encode(&data, &options);
        assert_eq!(result, "items[2\t]{id\tname}:\n  1\tA\n  2\tB");
    }

    #[test]
    fn test_pipe_delimiter() {
        let data = json!({
            "tags": ["a", "b", "c"]
        });
        let options = EncoderOptions {
            indent: 2,
            delimiter: Delimiter::Pipe,
        };
        let result = encode(&data, &options);
        assert_eq!(result, "tags[3|]: a|b|c");
    }

    #[test]
    fn test_deep_nesting() {
        let data = json!({
            "a": {
                "b": {
                    "c": "value"
                }
            }
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "a:\n  b:\n    c: value");
    }

    #[test]
    fn test_list_with_hyphen_values() {
        let data = json!({
            "items": ["-", "-test"]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "items[2]: \"-\",\"-test\"");
    }

    #[test]
    fn test_escape_sequences() {
        let data = json!({
            "text": "Line1\nLine2\tTab"
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "text: \"Line1\\nLine2\\tTab\"");
    }

    #[test]
    fn test_nested_arrays() {
        let data = json!({
            "matrix": [[1, 2], [3, 4]]
        });
        let result = encode(&data, &EncoderOptions::default());
        assert_eq!(result, "matrix[2]:\n  - [2]: 1,2\n  - [2]: 3,4");
    }
}
