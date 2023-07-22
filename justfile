set positional-arguments
set dotenv-load

run *args='':
    cargo run -- $@

build:
    cargo build

debug *args='': 
    RUST_LOG=debug cargo run -- $@

echo:
    echo "$PAPA_install_dir"