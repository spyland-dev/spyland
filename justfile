default: check

fmt:
    cargo fmt --all -- --check

build:
    cargo build --workspace --quiet

test:
    @if command -v cargo-nextest >/dev/null; then \
        cargo nextest run; \
    else \
        cargo test --workspace --quiet; \
    fi
doc:
    cargo doc --workspace --no-deps --quiet

clippy:
    cargo clippy --workspace --all-targets --all-features --quiet -- -D warnings

check: fmt build test doc clippy
