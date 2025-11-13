use json2toon_rs::{decode, encode, DecoderOptions, EncoderOptions};
use serde_json::json;

fn main() {
    println!("=== TOON Decode Examples ===\n");

    // Example 1: Decode simple object
    println!("1. Decode simple object:");
    let toon = "name: Alice\nage: 30\nactive: true";
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 2: Decode nested object
    println!("2. Decode nested object:");
    let toon = "user:\n  id: 123\n  name: Bob\n  email: bob@example.com";
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 3: Decode primitive array
    println!("3. Decode primitive array:");
    let toon = "tags[3]: admin,user,developer";
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 4: Decode tabular array
    println!("4. Decode tabular array:");
    let toon = "users[2]{id,name,role}:\n  1,Alice,admin\n  2,Bob,user";
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 5: Decode mixed array
    println!("5. Decode mixed array:");
    let toon = "items[3]:\n  - 42\n  - text\n  - true";
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 6: Round-trip encoding/decoding
    println!("6. Round-trip test:");
    let original = json!({
        "product": "Widget",
        "price": 9.99,
        "tags": ["new", "featured"],
        "specs": {
            "weight": 1.5,
            "color": "blue"
        }
    });
    println!(
        "Original JSON:\n{}\n",
        serde_json::to_string_pretty(&original).unwrap()
    );

    let toon = encode(&original, &EncoderOptions::default());
    println!("Encoded to TOON:\n{}\n", toon);

    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();
    println!(
        "Decoded back to JSON:\n{}\n",
        serde_json::to_string_pretty(&decoded).unwrap()
    );

    if original == decoded {
        println!("Round-trip successful!\n");
    }

    // Example 7: Decode with quoted strings and escapes
    println!("7. Decode quoted strings with escapes:");
    let toon = r#"url: "http://example.com:8080"
text: "Line1\nLine2\tTab"
reserved: "true""#;
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 8: Decode with tab delimiter
    println!("8. Decode tabular array with tab delimiter:");
    let toon_tab = "items[2\t]{id\tname\tprice}:\n  1\tWidget\t9.99\n  2\tGadget\t14.5";
    println!("TOON input (tabs):\n{}\n", toon_tab);
    let result = decode(toon_tab, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 9: Decode Unicode characters
    println!("9. Decode Unicode characters:");
    let toon = "message: Hello World\ngreeting: こんにちは\nmultilingual[3]: Hola,Bonjour,Ciao";
    println!("TOON input:\n{}\n", toon);
    let result = decode(toon, &DecoderOptions::default()).unwrap();
    println!(
        "JSON output:\n{}\n",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Example 10: Complex round-trip
    println!("10. Complex data structure round-trip:");
    let complex = json!({
        "company": "TechCorp",
        "employees": [
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false},
            {"id": 3, "name": "Charlie", "active": true}
        ],
        "metadata": {
            "created": "2025-01-01",
            "version": "2.0"
        }
    });

    let toon = encode(&complex, &EncoderOptions::default());
    println!("TOON format:\n{}\n", toon);

    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();
    println!(
        "Round-trip match: {}",
        if complex == decoded { "PASS" } else { "FAIL" }
    );
}
