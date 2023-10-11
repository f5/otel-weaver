install:
    cargo install cargo-machete

pre-push-check:
    cargo machete
    cargo fmt --all
    cargo clippy --all
    cargo test --all
