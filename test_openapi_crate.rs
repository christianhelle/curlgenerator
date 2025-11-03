use std::fs;

fn main() {
    let content = fs::read_to_string("./test/openapi/v2.0/petstore.json").unwrap();
    
    // Try parsing with openapi crate
    match openapi::from_str(&content) {
        Ok(spec) => {
            println!("Successfully parsed!");
            println!("Version: {:?}", spec.version);
            println!("Info: {:?}", spec.info);
            println!("Paths count: {}", spec.paths.len());
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}
