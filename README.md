# OTel Weaver
_A Schema-Driven Client SDK Generator for OpenTelemetry_

## Motivation

## Usage
### Generate a Client SDK
### Check a schema
### Export a schema

## Supported Languages
- Rust
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC
- Go
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC
- C++
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC
- Python
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC
- Java
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC
- C#
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC
- JavaScript
  - [ ] OTLP/HTTP
- Swift
  - [ ] OTLP/gRPC
  - [ ] OTel Arrow/gRPC

## How to Contribute
- Add support for a new language
  - [Via Tera templates](docs/contribution.md#via-tera-templates)
  - [Via WASM plugin](docs/contribution.md#via-wasm-plugin)
- Create other WASM plugins for 
  - [Schema validation](docs/contribution.md#schema-validation-plugin)
  - [Schema export](docs/contribution.md#schema-export-plugin)
  - [Variable resolver](docs/contribution.md#variable-resolver-plugin)

## Other links
- [Internal crates interdependencies](docs/dependencies.md)

## ToDo
- [ ] Add support for `all` in telemetry schema versions section.
- [ ] Add support for `span_events` in telemetry schema versions section.
- [ ] Add support for `apply_to_spans` in telemetry schema versions section.
- [ ] Add support for `apply_to_metrics` in telemetry schema metrics versions section.
- [ ] Add support for `split` in telemetry schema metrics versions section.
- [ ] Report unused semantic convention import