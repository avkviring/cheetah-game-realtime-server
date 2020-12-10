#!/bin/bash
rustup default stable

OUTPUT=Clients/Unity/Packages/CheetahRelay/

cd ../../../

## macos
rm -f $OUTPUT/x86_64/libcheetah_relay_client.bundle
cargo build --manifest-path clients/Rust/Cargo.toml --release
mkdir -p $OUTPUT/x86_64/
cp target/release/libcheetah_relay_client.dylib $OUTPUT/x86_64/libcheetah_relay_client.bundle

