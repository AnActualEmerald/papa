set positional-arguments

build:
    cargo build

@test *args='':
    cargo run --features="cluster" -- $@