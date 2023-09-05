use logger::Logger;
use resolver::SchemaResolver;
use std::process::exit;

fn main() {
    let mut log = Logger::new();
    let schema = SchemaResolver::resolve_schema_file("data/app-telemetry-schema.yaml", &mut log);
    if schema.is_err() {
        exit(1)
    }
}
