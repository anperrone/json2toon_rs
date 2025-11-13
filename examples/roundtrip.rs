use json2toon_rs::{decode, encode, DecoderOptions, EncoderOptions};
use serde_json::json;

fn main() {
    println!("=== JSON <-> TOON Round-Trip Examples ===\n");

    // Example 1: Simple data
    example_1();

    // Example 2: Nested structures
    example_2();

    // Example 3: Arrays and tables
    example_3();

    // Example 4: Complex real-world data
    example_4();
}

fn example_1() {
    println!("Example 1: Simple Object");
    println!("========================");

    let json = json!({
        "name": "Alice",
        "age": 30,
        "active": true
    });

    println!("Original JSON:");
    println!("{}\n", serde_json::to_string_pretty(&json).unwrap());

    let toon = encode(&json, &EncoderOptions::default());
    println!("TOON format:");
    println!("{}\n", toon);

    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();
    println!("Decoded JSON:");
    println!("{}\n", serde_json::to_string_pretty(&decoded).unwrap());

    assert_eq!(json, decoded, "Round-trip failed!");
    println!("Round-trip successful!\n");
}

fn example_2() {
    println!("Example 2: Nested Objects");
    println!("==========================");

    let json = json!({
        "user": {
            "profile": {
                "name": "Bob",
                "email": "bob@example.com"
            },
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        }
    });

    println!("Original JSON:");
    println!("{}\n", serde_json::to_string_pretty(&json).unwrap());

    let toon = encode(&json, &EncoderOptions::default());
    println!("TOON format:");
    println!("{}\n", toon);

    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();

    assert_eq!(json, decoded, "Round-trip failed!");
    println!("Round-trip successful!\n");
}

fn example_3() {
    println!("Example 3: Tabular Data");
    println!("========================");

    let json = json!({
        "products": [
            {"id": 1, "name": "Widget", "price": 9.99, "stock": 50},
            {"id": 2, "name": "Gadget", "price": 14.50, "stock": 30},
            {"id": 3, "name": "Doohickey", "price": 7.25, "stock": 100}
        ]
    });

    println!("Original JSON:");
    println!("{}\n", serde_json::to_string_pretty(&json).unwrap());

    let toon = encode(&json, &EncoderOptions::default());
    println!("TOON format (automatic tabular detection):");
    println!("{}\n", toon);

    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();

    assert_eq!(json, decoded, "Round-trip failed!");
    println!("Round-trip successful!\n");
}

fn example_4() {
    println!("Example 4: Complex Real-World Data");
    println!("===================================");

    let json = json!({
        "api_version": "v2",
        "server": {
            "host": "api.example.com",
            "port": 8080,
            "ssl": true
        },
        "endpoints": [
            {"method": "GET", "path": "/users", "auth": true},
            {"method": "POST", "path": "/users", "auth": true},
            {"method": "GET", "path": "/health", "auth": false}
        ],
        "database": {
            "type": "postgres",
            "connection": {
                "host": "db.example.com",
                "port": 5432,
                "database": "myapp"
            }
        },
        "features": ["auth", "cache", "metrics"],
        "limits": {
            "max_connections": 100,
            "timeout": 30,
            "rate_limit": 1000
        }
    });

    println!("Original JSON:");
    println!("{}\n", serde_json::to_string_pretty(&json).unwrap());

    let toon = encode(&json, &EncoderOptions::default());
    println!("TOON format:");
    println!("{}\n", toon);

    let decoded = decode(&toon, &DecoderOptions::default()).unwrap();

    assert_eq!(json, decoded, "Round-trip failed!");
    println!("Round-trip successful!");
    println!("\nAll examples completed successfully!");
}
