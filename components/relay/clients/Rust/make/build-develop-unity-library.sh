#!/bin/bash
rustup default stable

cd ../../../

## macos
rm -f $OUTPUT/x86_64/libcheetah_relay_client.bundle
cargo build --manifest-path clients/Rust/Cargo.toml --release
mkdir -p 'Clients/Unity/Packages/Cheetah Relay/x86_64/'
cp target/release/libcheetah_relay_client.dylib 'Clients/Unity/Packages/Cheetah Relay/x86_64/libcheetah_relay_client.bundle'

