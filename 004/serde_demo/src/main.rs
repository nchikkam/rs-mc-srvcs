use serde::{Serialize, Deserialize}; // <- NO serde_derive needed
use serde_json; // or toml, yaml, etc.

#[derive(Serialize, Deserialize)]
struct Config {
    host: String,
    port: u16,
}

fn main() {
    let config = Config {
        host: "localhost".into(),
        port: 8080,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    println!("Serialized: {}", json);

    // Deserialize from JSON
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    println!("Deserialized host: {}", deserialized.host);
}