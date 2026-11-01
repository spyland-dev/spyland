default: fmt build clippy

[working-directory: invocation_directory()]
fmt:
    cargo fmt -- --check

[working-directory: invocation_directory()]
build:
    cargo build --quiet

[working-directory: invocation_directory()]
test:
    cargo nextest run --no-tests pass

[working-directory: invocation_directory()]
clippy:
    cargo clippy --all-targets --all-features --quiet -- -D warnings

[working-directory: invocation_directory()]
doc:
    cargo test --doc --quiet
    cargo doc --no-deps --quiet

alias c := cli
cli *args:
    @cargo run --quiet -p spyland -- {{args}}

alias d := daemon
daemon *args:
    @cargo run --quiet -p spylandd -- {{args}}

check: fmt build test doc clippy
