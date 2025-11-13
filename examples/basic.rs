use json2toon_rs::{encode, Delimiter, EncoderOptions};
use serde_json::json;

fn main() {
    println!("=== JSON to TOON Examples ===\n");

    // Example 1: Simple object
    println!("1. Simple object:");
    let data = json!({
        "name": "Alice",
        "age": 30,
        "active": true
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 2: Nested object
    println!("2. Nested object:");
    let data = json!({
        "user": {
            "id": 123,
            "name": "Bob",
            "email": "bob@example.com"
        },
        "status": "active"
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 3: Primitive array
    println!("3. Primitive array:");
    let data = json!({
        "tags": ["admin", "user", "developer"]
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 4: Tabular array (uniform objects with primitives)
    println!("4. Tabular array:");
    let data = json!({
        "users": [
            {"id": 1, "name": "Alice", "role": "admin"},
            {"id": 2, "name": "Bob", "role": "user"}
        ]
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 5: Mixed array (expanded list format)
    println!("5. Mixed array:");
    let data = json!({
        "items": [
            42,
            "text",
            true,
            {"key": "value"}
        ]
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 6: Nested arrays
    println!("6. Nested arrays:");
    let data = json!({
        "matrix": [[1, 2, 3], [4, 5, 6]]
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 7: Tab delimiter
    println!("7. Tab delimiter:");
    let data = json!({
        "items": [
            {"id": 1, "name": "Widget", "price": 9.99},
            {"id": 2, "name": "Gadget", "price": 14.50}
        ]
    });
    let options = EncoderOptions {
        indent: 2,
        delimiter: Delimiter::Tab,
    };
    println!("{}\n", encode(&data, &options));

    // Example 8: Pipe delimiter
    println!("8. Pipe delimiter:");
    let data = json!({
        "categories": ["reading", "gaming", "coding"]
    });
    let options = EncoderOptions {
        indent: 2,
        delimiter: Delimiter::Pipe,
    };
    println!("{}\n", encode(&data, &options));

    // Example 9: Quoting special characters
    println!("9. Quoting special characters:");
    let data = json!({
        "url": "http://example.com:8080",
        "text": "Hello\nWorld",
        "reserved": "true",
        "numeric_string": "007"
    });
    println!("{}\n", encode(&data, &EncoderOptions::default()));

    // Example 10: Unicode support
    println!("10. Unicode support:");
    let data = json!({
        "message": "Hello World",
        "greeting": "こんにちは",
        "multilingual": ["Hola", "Bonjour", "Ciao"]
    });
    println!("{}", encode(&data, &EncoderOptions::default()));
}
