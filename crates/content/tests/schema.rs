//! `cargo test -p content --features schema export_schema` writes the JSON Schema for the
//! content files. Committed under schemas/ and diff-checked by `just gen-types-check`.
#![cfg(feature = "schema")]

#[test]
fn export_schema() {
    let schema = schemars::schema_for!(content::Content);
    let text = serde_json::to_string_pretty(&schema).unwrap();
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/content.schema.json");
    std::fs::write(&path, format!("{text}\n")).unwrap();
}
