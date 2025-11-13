use criterion::{black_box, criterion_group, criterion_main, Criterion};
use json2toon_rs::{decode, encode, DecoderOptions, EncoderOptions};
use serde_json::json;

fn get_complex_json() -> serde_json::Value {
    json!({
        "name": "Complex JSON for Benchmarking",
        "version": "1.0.0",
        "author": "Benchmark Runner",
        "license": "MIT",
        "description": "A more involved JSON structure to test performance of encoding and decoding.",
        "keywords": ["json", "toon", "benchmark", "performance", "rust"],
        "repository": {
            "type": "git",
            "url": "https://github.com/anperrone/json2toon_rs.git"
        },
        "users": [
            {
                "id": 101,
                "username": "alice",
                "email": "alice@example.com",
                "active": true,
                "roles": ["admin", "editor"],
                "profile": {
                    "fullName": "Alice Anderson",
                    "joinDate": "2023-01-15T10:00:00Z",
                    "avatar": "https://example.com/avatars/alice.png"
                }
            },
            {
                "id": 102,
                "username": "bob",
                "email": "bob@example.com",
                "active": false,
                "roles": ["viewer"],
                "profile": {
                    "fullName": "Bob Brown",
                    "joinDate": "2023-02-20T14:30:00Z",
                    "avatar": "https://example.com/avatars/bob.png"
                }
            },
            {
                "id": 103,
                "username": "charlie",
                "email": "charlie@example.com",
                "active": true,
                "roles": ["editor", "contributor"],
                "profile": {
                    "fullName": "Charlie Clark",
                    "joinDate": "2023-03-10T09:00:00Z",
                    "avatar": "https://example.com/avatars/charlie.png"
                }
            }
        ],
        "settings": {
            "theme": "dark",
            "notifications": {
                "email": true,
                "push": false,
                "sms": false
            },
            "pagination": {
                "pageSize": 20,
                "defaultSort": "createdAt"
            }
        },
        "features": {
            "featureA": true,
            "featureB": false,
            "featureC": true
        },
        "matrix": [
            [1, 2, 3, 4, 5],
            [6, 7, 8, 9, 10],
            [11, 12, 13, 14, 15]
        ],
        "empty_object": {},
        "empty_array": []
    })
}

fn benchmark_encode(c: &mut Criterion) {
    let data = get_complex_json();
    let options = EncoderOptions::default();

    c.bench_function("encode_complex_json", |b| {
        b.iter(|| encode(black_box(&data), black_box(&options)))
    });
}

fn benchmark_decode(c: &mut Criterion) {
    let data = get_complex_json();
    let options = EncoderOptions::default();
    let toon_string = encode(&data, &options);
    let decode_options = DecoderOptions::default();

    c.bench_function("decode_complex_toon", |b| {
        b.iter(|| decode(black_box(&toon_string), black_box(&decode_options)).unwrap())
    });
}

criterion_group!(benches, benchmark_encode, benchmark_decode);
criterion_main!(benches);
