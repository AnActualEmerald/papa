set positional-arguments
set dotenv-load

build:
    cargo build

debug *args='': 
    RUST_LOG=debug cargo run -- $@

echo:
    echo "$PAPA_install_dir"