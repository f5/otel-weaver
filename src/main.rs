use logger::Logger;
use resolver::SchemaResolver;
use std::process::exit;

fn main() {
    let mut log = Logger::new();
    let schema_name = "data/app-telemetry-schema.yaml";
    let schema = SchemaResolver::resolve_schema_file(schema_name, &mut log);
    match schema {
        Ok(schema) => {
            log.success(&format!("Loaded schema {}", schema_name));
            match serde_yaml::to_string(&schema) {
                Ok(yaml) => {
                    log.log(&yaml);
                }
                Err(e) => {
                    log.error(&format!("{}", e));
                    exit(1)
                }
            }
        }
        Err(e) => {
            log.error(&format!("{}", e));
            exit(1)
        }
    }
}
