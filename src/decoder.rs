//! TOON to JSON decoder implementation

use crate::common::Delimiter;
use serde_json::Value;

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
pub fn decode(input: &str, options: &DecoderOptions) -> Result<Value, String> {
    let mut decoder = Decoder::new(input, options);
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
    fn new(input: &str, options: &'a DecoderOptions) -> Self {
        let lines = Self::parse_lines(input, options);
        Self {
            lines,
            options,
            pos: 0,
        }
    }

    /// Parse input into lines with depth information
    fn parse_lines(input: &str, options: &DecoderOptions) -> Vec<Line> {
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
                    return Some(Line {
                        content: format!("ERROR: Invalid indentation at line {}", i + 1),
                        depth: 0,
                        line_num: i + 1,
                    });
                }

                let depth = leading_spaces / options.indent;
                Some(Line {
                    content: line.trim().to_string(),
                    depth,
                    line_num: i + 1,
                })
            })
            .collect()
    }

    fn decode(&mut self) -> Result<Value, String> {
        if self.lines.is_empty() {
            // Empty document = empty object
            return Ok(Value::Object(serde_json::Map::new()));
        }

        // Check if error line exists
        if !self.lines.is_empty() && self.lines[0].content.starts_with("ERROR:") {
            return Err(self.lines[0].content.clone());
        }

        // Determine root form (§5)
        if self.is_root_array() {
            self.decode_array(0)
        } else if self.lines.len() == 1 && !self.is_key_value(&self.lines[0].content) {
            // Single primitive line
            Ok(self.parse_primitive(&self.lines[0].content))
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
    ) -> Result<Value, String> {
        let mut obj = serde_json::Map::new();

        while self.pos < self.lines.len() {
            let line = &self.lines[self.pos].clone();

            // Stop if we've reached the end marker or depth decreased
            if let Some(end) = end_line {
                if self.pos >= end {
                    break;
                }
            }

            if line.depth < start_depth {
                break;
            }

            if line.depth > start_depth {
                // Skip - handled by nested structure
                self.pos += 1;
                continue;
            }

            // Parse key-value at this depth
            if let Some((key, value_part)) = self.parse_key_value(&line.content)? {
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
                        self.try_parse_array_header(&full_header, start_depth)?
                    {
                        array_value
                    } else {
                        return Err(format!("Invalid array header in key: {}", key));
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
                    self.parse_primitive(&value_part)
                };

                obj.insert(actual_key, value);
            } else {
                return Err(format!(
                    "Invalid line at {}: {}",
                    line.line_num, line.content
                ));
            }
        }

        Ok(Value::Object(obj))
    }

    /// Try to parse array header and content
    fn try_parse_array_header(
        &mut self,
        header_part: &str,
        parent_depth: usize,
    ) -> Result<Option<Value>, String> {
        if !header_part.starts_with('[') {
            return Ok(None);
        }

        let (length, delimiter, fields) = self.parse_array_header(header_part)?;

        // Check if inline values follow
        if let Some(colon_pos) = header_part.find(':') {
            let after_colon = header_part[colon_pos + 1..].trim();

            if !after_colon.is_empty() {
                // Inline primitive array
                return Ok(Some(self.decode_inline_array(
                    after_colon,
                    delimiter,
                    length,
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
    fn parse_array_header(&self, header: &str) -> Result<(usize, Delimiter, Vec<String>), String> {
        let bracket_end = header.find(']').ok_or("Missing ] in array header")?;
        let bracket_content = &header[1..bracket_end];

        // Parse length and delimiter
        let (length, delimiter) = if let Some(stripped) = bracket_content.strip_suffix('\t') {
            (
                stripped.parse().map_err(|_| "Invalid array length")?,
                Delimiter::Tab,
            )
        } else if let Some(stripped) = bracket_content.strip_suffix('|') {
            (
                stripped.parse().map_err(|_| "Invalid array length")?,
                Delimiter::Pipe,
            )
        } else {
            (
                bracket_content
                    .parse()
                    .map_err(|_| "Invalid array length")?,
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
                    .map(|f| self.unescape_string(&f))
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
    ) -> Result<Value, String> {
        let values = self.split_by_delimiter(values_str, delimiter);

        if self.options.strict && values.len() != expected_len {
            return Err(format!(
                "Array length mismatch: expected {}, got {}",
                expected_len,
                values.len()
            ));
        }

        let arr: Vec<Value> = values.iter().map(|v| self.parse_primitive(v)).collect();

        Ok(Value::Array(arr))
    }

    /// Decode tabular array
    fn decode_tabular_array(
        &mut self,
        row_depth: usize,
        expected_rows: usize,
        delimiter: Delimiter,
        fields: &[String],
    ) -> Result<Value, String> {
        let mut arr = Vec::new();

        while self.pos < self.lines.len() && self.lines[self.pos].depth == row_depth {
            let line = &self.lines[self.pos];
            let values = self.split_by_delimiter(&line.content, delimiter);

            if self.options.strict && values.len() != fields.len() {
                return Err(format!(
                    "Row width mismatch at line {}: expected {} fields, got {}",
                    line.line_num,
                    fields.len(),
                    values.len()
                ));
            }

            let mut obj = serde_json::Map::new();
            for (i, field) in fields.iter().enumerate() {
                if i < values.len() {
                    obj.insert(field.clone(), self.parse_primitive(&values[i]));
                }
            }
            arr.push(Value::Object(obj));
            self.pos += 1;
        }

        if self.options.strict && arr.len() != expected_rows {
            return Err(format!(
                "Array length mismatch: expected {} rows, got {}",
                expected_rows,
                arr.len()
            ));
        }

        Ok(Value::Array(arr))
    }

    /// Decode list array (expanded format)
    fn decode_list_array(
        &mut self,
        item_depth: usize,
        expected_len: usize,
        _delimiter: Delimiter,
    ) -> Result<Value, String> {
        let mut arr = Vec::new();

        while self.pos < self.lines.len() && self.lines[self.pos].depth == item_depth {
            let line = &self.lines[self.pos].clone();

            if !line.content.starts_with("- ") {
                break;
            }

            let item_content = &line.content[2..];
            self.pos += 1;

            let value = if item_content.starts_with('[') {
                // Inline array item
                let (length, delim, _) = self.parse_array_header(item_content)?;
                if let Some(colon_pos) = item_content.find(':') {
                    let after_colon = item_content[colon_pos + 1..].trim();
                    self.decode_inline_array(after_colon, delim, length)?
                } else {
                    Value::Null
                }
            } else if let Some((key, value_part)) = self.parse_key_value(item_content)? {
                // Object as list item - decode it
                self.pos -= 1; // Back up to re-process as object
                let saved_content = self.lines[self.pos].content.clone();
                self.lines[self.pos].content = format!("{}: {}", key, value_part);

                let mut obj = serde_json::Map::new();

                // First field
                if value_part.is_empty() {
                    // Nested structure
                    if self.pos + 1 < self.lines.len()
                        && self.lines[self.pos + 1].depth > item_depth
                    {
                        self.pos += 1;
                        obj.insert(key, self.decode_object(item_depth + 1, None)?);
                    } else {
                        obj.insert(key, Value::Object(serde_json::Map::new()));
                        self.pos += 1;
                    }
                } else if let Some(arr_val) =
                    self.try_parse_array_header(&value_part, item_depth)?
                {
                    obj.insert(key, arr_val);
                } else {
                    obj.insert(key, self.parse_primitive(&value_part));
                    self.pos += 1;
                }

                // Remaining fields at item_depth
                while self.pos < self.lines.len()
                    && self.lines[self.pos].depth == item_depth
                    && !self.lines[self.pos].content.starts_with("- ")
                {
                    let field_line = &self.lines[self.pos].clone();
                    if let Some((k, v)) = self.parse_key_value(&field_line.content)? {
                        self.pos += 1;
                        if v.is_empty() {
                            if self.pos < self.lines.len()
                                && self.lines[self.pos].depth > item_depth
                            {
                                obj.insert(k, self.decode_object(item_depth + 1, None)?);
                            } else {
                                obj.insert(k, Value::Object(serde_json::Map::new()));
                            }
                        } else if let Some(arr_val) = self.try_parse_array_header(&v, item_depth)? {
                            obj.insert(k, arr_val);
                        } else {
                            obj.insert(k, self.parse_primitive(&v));
                        }
                    } else {
                        break;
                    }
                }

                self.lines[self.pos - arr.len() - 1].content = saved_content;
                Value::Object(obj)
            } else {
                // Primitive item
                self.parse_primitive(item_content)
            };

            arr.push(value);
        }

        if self.options.strict && arr.len() != expected_len {
            return Err(format!(
                "Array length mismatch: expected {} items, got {}",
                expected_len,
                arr.len()
            ));
        }

        Ok(Value::Array(arr))
    }

    /// Decode root array
    fn decode_array(&mut self, depth: usize) -> Result<Value, String> {
        let line = &self.lines[0].content;
        let (length, delimiter, fields) = self.parse_array_header(line)?;

        self.pos = 1;

        if !fields.is_empty() {
            self.decode_tabular_array(depth + 1, length, delimiter, &fields)
        } else {
            self.decode_list_array(depth + 1, length, delimiter)
        }
    }

    /// Parse key: value line
    fn parse_key_value(&self, line: &str) -> Result<Option<(String, String)>, String> {
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

            let unescaped_key = self.unescape_string(key)?;
            Ok(Some((unescaped_key, value.to_string())))
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
    fn parse_primitive(&self, s: &str) -> Value {
        let trimmed = s.trim();

        // Quoted string
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return match self.unescape_string(trimmed) {
                Ok(s) => Value::String(s),
                Err(_) => Value::String(trimmed.to_string()),
            };
        }

        // Booleans and null
        match trimmed {
            "true" => return Value::Bool(true),
            "false" => return Value::Bool(false),
            "null" => return Value::Null,
            _ => {}
        }

        // Try parsing as number (reject leading zeros per spec)
        if !trimmed.is_empty() && !trimmed.starts_with('0')
            || trimmed == "0"
            || trimmed.starts_with("0.")
            || trimmed.starts_with("-0")
        {
            if let Ok(i) = trimmed.parse::<i64>() {
                return Value::Number(i.into());
            }
            if let Ok(f) = trimmed.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(f) {
                    return Value::Number(num);
                }
            }
        }

        // Default to string
        Value::String(trimmed.to_string())
    }

    /// Unescape string (remove quotes and handle escapes)
    fn unescape_string(&self, s: &str) -> Result<String, String> {
        let s = s.trim();

        if !s.starts_with('"') || !s.ends_with('"') {
            return Ok(s.to_string());
        }

        let inner = &s[1..s.len() - 1];
        let mut result = String::new();
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
                        if self.options.strict {
                            return Err(format!("Invalid escape sequence: \\{}", other));
                        }
                        result.push('\\');
                        result.push(other);
                    }
                    None => {
                        if self.options.strict {
                            return Err("Unterminated escape sequence".to_string());
                        }
                        result.push('\\');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
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
}
