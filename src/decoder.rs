//! TOON to JSON decoder implementation

use crate::common::Delimiter;
use crate::error::DecodeError;
use serde_json::Value;
use std::borrow::Cow;

/// Decoder configuration options
#[derive(Debug, Clone)]
pub struct DecoderOptions {
    /// Spaces per indentation level (default: 2)
    pub indent: usize,
    /// Strict mode - enforces counts, indentation, etc. (default: true)
    pub strict: bool,
}

impl Default for DecoderOptions {
    fn default() -> Self {
        Self {
            indent: 2,
            strict: true,
        }
    }
}

/// Decode TOON format to JSON value
pub fn decode(input: &str, options: &DecoderOptions) -> Result<Value, DecodeError> {
    let mut decoder = Decoder::new(input, options)?;
    decoder.decode()
}

struct Decoder<'a> {
    lines: Vec<Line>,
    options: &'a DecoderOptions,
    pos: usize,
}

#[derive(Debug, Clone)]
struct Line {
    content: String,
    depth: usize,
    line_num: usize,
}

impl<'a> Decoder<'a> {
    fn new(input: &str, options: &'a DecoderOptions) -> Result<Self, DecodeError> {
        let lines = Self::parse_lines(input, options)?;
        Ok(Self {
            lines,
            options,
            pos: 0,
        })
    }

    /// Parse input into lines with depth information
    fn parse_lines(input: &str, options: &DecoderOptions) -> Result<Vec<Line>, DecodeError> {
        input
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                // Skip completely blank lines outside structures
                if line.trim().is_empty() {
                    return None;
                }

                let leading_spaces = line.len() - line.trim_start().len();

                // Validate indentation in strict mode
                if options.strict && leading_spaces % options.indent != 0 {
                    return Some(Err(DecodeError::InvalidIndentation { line: i + 1 }));
                }

                let depth = leading_spaces / options.indent;
                Some(Ok(Line {
                    content: line.trim().to_string(),
                    depth,
                    line_num: i + 1,
                }))
            })
            .collect()
    }

    fn decode(&mut self) -> Result<Value, DecodeError> {
        if self.lines.is_empty() {
            // Empty document = empty object
            return Ok(Value::Object(serde_json::Map::new()));
        }

        // Determine root form (§5)
        if self.is_root_array() {
            self.decode_array(0)
        } else if self.lines.len() == 1 && !self.is_key_value(&self.lines[0].content) {
            // Single primitive line
            Ok(self.parse_primitive(&self.lines[0].content, self.lines[0].line_num)?)
        } else {
            // Object
            self.decode_object(0, None)
        }
    }

    /// Check if root is an array (starts with array header)
    fn is_root_array(&self) -> bool {
        if self.lines.is_empty() {
            return false;
        }
        let content = &self.lines[0].content;
        content.starts_with('[') && content.contains("]:")
    }

    /// Check if line is key-value format (has unquoted colon)
    fn is_key_value(&self, line: &str) -> bool {
        let mut in_quotes = false;
        for ch in line.chars() {
            if ch == '"' {
                in_quotes = !in_quotes;
            } else if ch == ':' && !in_quotes {
                return true;
            }
        }
        false
    }

    /// Decode an object starting at given depth
    fn decode_object(
        &mut self,
        start_depth: usize,
        end_line: Option<usize>,
    ) -> Result<Value, DecodeError> {
        let mut obj = serde_json::Map::new();

        while self.pos < self.lines.len() {
            let line_num = self.lines[self.pos].line_num;
            let depth = self.lines[self.pos].depth;

            // Stop if we've reached the end marker or depth decreased
            if let Some(end) = end_line {
                if self.pos >= end {
                    break;
                }
            }

            if depth < start_depth {
                break;
            }

            if depth > start_depth {
                // Skip - handled by nested structure
                self.pos += 1;
                continue;
            }

            // Clone content to avoid borrowing issues with self.pos modification
            let content = self.lines[self.pos].content.clone();

            // Parse key-value at this depth
            if let Some((key, value_part)) = self.parse_key_value(&content, line_num)? {
                self.pos += 1;

                // Check if key contains array header (e.g., "tags[3]")
                let (actual_key, array_header) = if key.contains('[') {
                    if let Some(bracket_pos) = key.find('[') {
                        let k = &key[..bracket_pos];
                        let h = &key[bracket_pos..];
                        (k.to_string(), Some(h.to_string()))
                    } else {
                        (key.clone(), None)
                    }
                } else {
                    (key.clone(), None)
                };

                let value = if let Some(header) = array_header {
                    // Key has array header - parse as array
                    let full_header = if value_part.is_empty() {
                        header
                    } else {
                        format!("{}:{}", header, value_part)
                    };

                    if let Some(array_value) =
                        self.try_parse_array_header(&full_header, start_depth, line_num)?
                    {
                        array_value
                    } else {
                        return Err(DecodeError::InvalidArrayHeader(format!(
                            "Invalid array header in key: {}",
                            key
                        )));
                    }
                } else if value_part.is_empty() {
                    // Nested object or empty object
                    if self.pos < self.lines.len() && self.lines[self.pos].depth > start_depth {
                        self.decode_object(start_depth + 1, None)?
                    } else {
                        Value::Object(serde_json::Map::new())
                    }
                } else {
                    // Primitive value
                    self.parse_primitive(&value_part, line_num)?
                };

                obj.insert(actual_key, value);
            } else {
                return Err(DecodeError::InvalidLine {
                    line: line_num,
                    content,
                });
            }
        }

        Ok(Value::Object(obj))
    }

    /// Try to parse array header and content
    fn try_parse_array_header(
        &mut self,
        header_part: &str,
        parent_depth: usize,
        line_num: usize,
    ) -> Result<Option<Value>, DecodeError> {
        if !header_part.starts_with('[') {
            return Ok(None);
        }

        let (length, delimiter, fields) = self.parse_array_header(header_part, line_num)?;

        // Check if inline values follow
        if let Some(colon_pos) = header_part.find(':') {
            let after_colon = header_part[colon_pos + 1..].trim();

            if !after_colon.is_empty() {
                // Inline primitive array
                return Ok(Some(self.decode_inline_array(
                    after_colon,
                    delimiter,
                    length,
                    line_num,
                )?));
            }
        }

        // Check for tabular or list format
        if !fields.is_empty() {
            // Tabular format
            Ok(Some(self.decode_tabular_array(
                parent_depth + 1,
                length,
                delimiter,
                &fields,
            )?))
        } else {
            // List format
            Ok(Some(self.decode_list_array(
                parent_depth + 1,
                length,
                delimiter,
            )?))
        }
    }

    /// Parse array header: [N<delim?>]{fields}:
    fn parse_array_header(
        &self,
        header: &str,
        line_num: usize,
    ) -> Result<(usize, Delimiter, Vec<String>), DecodeError> {
        let bracket_end = header.find(']').ok_or_else(|| {
            DecodeError::InvalidArrayHeader("Missing ] in array header".to_string())
        })?;
        let bracket_content = &header[1..bracket_end];

        // Parse length and delimiter
        let (length, delimiter) = if let Some(stripped) = bracket_content.strip_suffix('\t') {
            (
                stripped.parse().map_err(|_| {
                    DecodeError::InvalidArrayHeader("Invalid array length".to_string())
                })?,
                Delimiter::Tab,
            )
        } else if let Some(stripped) = bracket_content.strip_suffix('|') {
            (
                stripped.parse().map_err(|_| {
                    DecodeError::InvalidArrayHeader("Invalid array length".to_string())
                })?,
                Delimiter::Pipe,
            )
        } else {
            (
                bracket_content.parse().map_err(|_| {
                    DecodeError::InvalidArrayHeader("Invalid array length".to_string())
                })?,
                Delimiter::Comma,
            )
        };

        // Check for fields
        let mut fields = Vec::new();
        let after_bracket = &header[bracket_end + 1..];
        if after_bracket.starts_with('{') {
            if let Some(close_brace) = after_bracket.find('}') {
                let fields_str = &after_bracket[1..close_brace];
                fields = self
                    .split_by_delimiter(fields_str, delimiter)
                    .into_iter()
                    .map(|f| self.unescape_string(&f, line_num))
                    .collect::<Result<Vec<_>, _>>()?;
            }
        }

        Ok((length, delimiter, fields))
    }

    /// Decode inline primitive array
    fn decode_inline_array(
        &self,
        values_str: &str,
        delimiter: Delimiter,
        expected_len: usize,
        line_num: usize,
    ) -> Result<Value, DecodeError> {
        let values = self.split_by_delimiter(values_str, delimiter);

        if self.options.strict && values.len() != expected_len {
            return Err(DecodeError::ArrayLengthMismatch {
                expected: expected_len,
                found: values.len(),
            });
        }

        let arr: Result<Vec<Value>, _> = values
            .iter()
            .map(|v| self.parse_primitive(v, line_num))
            .collect();

        Ok(Value::Array(arr?))
    }

    /// Decode tabular array
    fn decode_tabular_array(
        &mut self,
        row_depth: usize,
        expected_rows: usize,
        delimiter: Delimiter,
        fields: &[String],
    ) -> Result<Value, DecodeError> {
        let mut arr = Vec::new();

        while self.pos < self.lines.len() && self.lines[self.pos].depth == row_depth {
            let line = &self.lines[self.pos];
            let values = self.split_by_delimiter(&line.content, delimiter);

            if self.options.strict && values.len() != fields.len() {
                return Err(DecodeError::RowWidthMismatch {
                    line: line.line_num,
                    expected: fields.len(),
                    found: values.len(),
                });
            }

            let mut obj = serde_json::Map::new();
            for (i, field) in fields.iter().enumerate() {
                if i < values.len() {
                    obj.insert(
                        field.clone(),
                        self.parse_primitive(&values[i], line.line_num)?,
                    );
                }
            }
            arr.push(Value::Object(obj));
            self.pos += 1;
        }

        if self.options.strict && arr.len() != expected_rows {
            return Err(DecodeError::ArrayLengthMismatch {
                expected: expected_rows,
                found: arr.len(),
            });
        }

        Ok(Value::Array(arr))
    }

    /// Helper to decode an object that appears as a list item
    fn decode_list_item_object(
        &mut self,
        first_key: String,
        first_value: String,
        item_depth: usize,
        line_num: usize,
    ) -> Result<serde_json::Map<String, Value>, DecodeError> {
        let mut obj = serde_json::Map::new();

        // Process first field
        if first_value.is_empty() {
            // Nested structure
            if self.pos < self.lines.len() && self.lines[self.pos].depth > item_depth {
                obj.insert(first_key, self.decode_object(item_depth + 1, None)?);
            } else {
                obj.insert(first_key, Value::Object(serde_json::Map::new()));
            }
        } else if let Some(arr_val) =
            self.try_parse_array_header(&first_value, item_depth, line_num)?
        {
            obj.insert(first_key, arr_val);
        } else {
            obj.insert(first_key, self.parse_primitive(&first_value, line_num)?);
        }

        // Process remaining fields at item_depth
        while self.pos < self.lines.len()
            && self.lines[self.pos].depth == item_depth
            && !self.lines[self.pos].content.starts_with("- ")
        {
            let field_line = &self.lines[self.pos].clone();
            if let Some((k, v)) = self.parse_key_value(&field_line.content, field_line.line_num)? {
                self.pos += 1;
                if v.is_empty() {
                    if self.pos < self.lines.len() && self.lines[self.pos].depth > item_depth {
                        obj.insert(k, self.decode_object(item_depth + 1, None)?);
                    } else {
                        obj.insert(k, Value::Object(serde_json::Map::new()));
                    }
                } else if let Some(arr_val) =
                    self.try_parse_array_header(&v, item_depth, field_line.line_num)?
                {
                    obj.insert(k, arr_val);
                } else {
                    obj.insert(k, self.parse_primitive(&v, field_line.line_num)?);
                }
            } else {
                break;
            }
        }

        Ok(obj)
    }

    /// Decode list array (expanded format)
    fn decode_list_array(
        &mut self,
        item_depth: usize,
        expected_len: usize,
        _delimiter: Delimiter,
    ) -> Result<Value, DecodeError> {
        let mut arr = Vec::new();

        while self.pos < self.lines.len() && self.lines[self.pos].depth == item_depth {
            let line = self.lines[self.pos].clone();

            if !line.content.starts_with("- ") {
                break;
            }

            let item_content = &line.content[2..];
            self.pos += 1;

            let value = if item_content.starts_with('[') {
                // Inline array item
                let (length, delim, _) = self.parse_array_header(item_content, line.line_num)?;
                if let Some(colon_pos) = item_content.find(':') {
                    let after_colon = item_content[colon_pos + 1..].trim();
                    self.decode_inline_array(after_colon, delim, length, line.line_num)?
                } else {
                    Value::Null
                }
            } else if let Some((key, value_part)) =
                self.parse_key_value(item_content, line.line_num)?
            {
                // Object as list item - decode it without mutating internal state
                let obj =
                    self.decode_list_item_object(key, value_part, item_depth, line.line_num)?;
                Value::Object(obj)
            } else {
                // Primitive item
                self.parse_primitive(item_content, line.line_num)?
            };

            arr.push(value);
        }

        if self.options.strict && arr.len() != expected_len {
            return Err(DecodeError::ArrayLengthMismatch {
                expected: expected_len,
                found: arr.len(),
            });
        }

        Ok(Value::Array(arr))
    }

    /// Decode root array
    fn decode_array(&mut self, depth: usize) -> Result<Value, DecodeError> {
        let line = &self.lines[0];
        let (length, delimiter, fields) = self.parse_array_header(&line.content, line.line_num)?;

        self.pos = 1;

        if !fields.is_empty() {
            self.decode_tabular_array(depth + 1, length, delimiter, &fields)
        } else {
            self.decode_list_array(depth + 1, length, delimiter)
        }
    }

    /// Parse key: value line
    fn parse_key_value(
        &self,
        line: &str,
        line_num: usize,
    ) -> Result<Option<(String, String)>, DecodeError> {
        let mut in_quotes = false;
        let mut colon_pos = None;

        for (i, ch) in line.chars().enumerate() {
            if ch == '"' && (i == 0 || line.chars().nth(i - 1) != Some('\\')) {
                in_quotes = !in_quotes;
            } else if ch == ':' && !in_quotes {
                colon_pos = Some(i);
                break;
            }
        }

        if let Some(pos) = colon_pos {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();

            let unescaped_key = self.unescape_string_cow(key, line_num)?;
            Ok(Some((unescaped_key.into_owned(), value.to_string())))
        } else {
            Ok(None)
        }
    }

    /// Split string by delimiter, respecting quotes
    fn split_by_delimiter(&self, s: &str, delimiter: Delimiter) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let delim_char = delimiter.as_char();

        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '"' {
                in_quotes = !in_quotes;
                current.push(ch);
            } else if ch == '\\' && in_quotes {
                current.push(ch);
                if let Some(&next_ch) = chars.peek() {
                    current.push(next_ch);
                    chars.next();
                }
            } else if ch == delim_char && !in_quotes {
                result.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(ch);
            }
        }

        result.push(current.trim().to_string());
        result
    }

    /// Parse primitive value
    fn parse_primitive(&self, s: &str, line_num: usize) -> Result<Value, DecodeError> {
        let trimmed = s.trim();

        // Quoted string
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return Ok(Value::String(self.unescape_string(trimmed, line_num)?));
        }

        // Booleans and null
        match trimmed {
            "true" => return Ok(Value::Bool(true)),
            "false" => return Ok(Value::Bool(false)),
            "null" => return Ok(Value::Null),
            _ => {}
        }

        // Try parsing as number (reject leading zeros per spec)
        if !trimmed.is_empty() && !trimmed.starts_with('0')
            || trimmed == "0"
            || trimmed.starts_with("0.")
            || trimmed.starts_with("-0")
        {
            if let Ok(i) = trimmed.parse::<i64>() {
                return Ok(Value::Number(i.into()));
            }
            if let Ok(f) = trimmed.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(f) {
                    return Ok(Value::Number(num));
                }
            }
        }

        // Default to string
        Ok(Value::String(trimmed.to_string()))
    }

    /// Unescape string with Cow optimization (remove quotes and handle escapes)
    /// Returns Cow::Borrowed if no unescaping is needed, Cow::Owned otherwise
    fn unescape_string_cow<'b>(
        &self,
        s: &'b str,
        line_num: usize,
    ) -> Result<Cow<'b, str>, DecodeError> {
        let trimmed = s.trim();

        // If not quoted, return borrowed
        if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
            return Ok(Cow::Borrowed(trimmed));
        }

        let inner = &trimmed[1..trimmed.len() - 1];

        // Check if we need to allocate (has escape sequences)
        if !inner.contains('\\') {
            return Ok(Cow::Borrowed(inner));
        }

        // Need to process escape sequences - allocate
        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(other) => {
                        // Validate that the escape character is ASCII
                        if !other.is_ascii() && self.options.strict {
                            return Err(DecodeError::InvalidEscapeSequence {
                                line: line_num,
                                sequence: format!("{} (non-ASCII character in escape)", other),
                            });
                        }

                        if self.options.strict {
                            return Err(DecodeError::InvalidEscapeSequence {
                                line: line_num,
                                sequence: other.to_string(),
                            });
                        }
                        result.push('\\');
                        result.push(other);
                    }
                    None => {
                        if self.options.strict {
                            return Err(DecodeError::ParseError(
                                "Unterminated escape sequence".to_string(),
                            ));
                        }
                        result.push('\\');
                    }
                }
            } else {
                // Regular character - push directly (Rust String guarantees UTF-8)
                result.push(ch);
            }
        }

        // Final validation: ensure result is valid UTF-8
        // This is guaranteed by Rust's String type, but we check explicitly for documentation
        debug_assert!(result.is_char_boundary(0) && result.is_char_boundary(result.len()));

        Ok(Cow::Owned(result))
    }

    /// Unescape string (remove quotes and handle escapes)
    /// Legacy wrapper for backward compatibility
    fn unescape_string(&self, s: &str, line_num: usize) -> Result<String, DecodeError> {
        self.unescape_string_cow(s, line_num)
            .map(|cow| cow.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{encode, EncoderOptions};
    use serde_json::json;

    #[test]
    fn test_decode_empty() {
        let result = decode("", &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_decode_simple_object() {
        let toon = "name: Alice\nage: 30\nactive: true";
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"name": "Alice", "age": 30, "active": true}));
    }

    #[test]
    fn test_decode_nested_object() {
        let toon = "user:\n  id: 123\n  name: Bob";
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"user": {"id": 123, "name": "Bob"}}));
    }

    #[test]
    fn test_decode_primitive_array() {
        let toon = "tags[3]: admin,user,dev";
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"tags": ["admin", "user", "dev"]}));
    }

    #[test]
    fn test_decode_tabular_array() {
        let toon = "users[2]{id,name,active}:\n  1,Alice,true\n  2,Bob,false";
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(
            result,
            json!({
                "users": [
                    {"id": 1, "name": "Alice", "active": true},
                    {"id": 2, "name": "Bob", "active": false}
                ]
            })
        );
    }

    #[test]
    fn test_decode_quoted_strings() {
        let toon = r#"url: "http://example.com:8080""#;
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"url": "http://example.com:8080"}));
    }

    #[test]
    fn test_decode_escape_sequences() {
        let toon = r#"text: "Line1\nLine2\tTab""#;
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"text": "Line1\nLine2\tTab"}));
    }

    #[test]
    fn test_round_trip() {
        let original = json!({
            "name": "Test",
            "items": [
                {"id": 1, "value": "A"},
                {"id": 2, "value": "B"}
            ]
        });

        let toon = encode(&original, &EncoderOptions::default());
        let decoded = decode(&toon, &DecoderOptions::default()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_decode_mixed_array() {
        let toon = "items[3]:\n  - 42\n  - text\n  - true";
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"items": [42, "text", true]}));
    }

    #[test]
    fn test_decode_unicode() {
        let toon = "message: Hello 世界 👋";
        let result = decode(toon, &DecoderOptions::default()).unwrap();
        assert_eq!(result, json!({"message": "Hello 世界 👋"}));
    }

    #[test]
    fn test_invalid_indentation() {
        let toon = "user:\n id: 123"; // 1 space instead of 2
        let result = decode(toon, &DecoderOptions::default());
        assert!(matches!(
            result,
            Err(DecodeError::InvalidIndentation { line: 2 })
        ));
    }

    #[test]
    fn test_array_length_mismatch() {
        let toon = "tags[2]: one,two,three";
        let result = decode(toon, &DecoderOptions::default());
        assert!(matches!(
            result,
            Err(DecodeError::ArrayLengthMismatch {
                expected: 2,
                found: 3
            })
        ));
    }

    #[test]
    fn test_tabular_row_width_mismatch() {
        let toon = "users[1]{id,name}:\n  1,Alice,admin";
        let result = decode(toon, &DecoderOptions::default());
        assert!(matches!(
            result,
            Err(DecodeError::RowWidthMismatch {
                line: 2,
                expected: 2,
                found: 3
            })
        ));
    }
}
