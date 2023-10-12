default: pre-push

install:
    cargo install cargo-machete
    cargo install cargo-depgraph

pre-push-check:
    cargo machete
    cargo fmt --all
    cargo clippy --all
    cargo test --all

pre-push: pre-push-check
    cargo depgraph --workspace-only | dot -Tsvg > docs/images/dependencies.svg