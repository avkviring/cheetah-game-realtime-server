#!/bin/bash

OUTPUT=Clients/Unity/Packages/CheetahRelay/

cd ../../../

## macos
rm -f $OUTPUT/x86_64/libcheetah_relay_client.dylib
cargo build --manifest-path clients/Rust/Cargo.toml
mkdir -p $OUTPUT/x86_64/
cp target/debug/libcheetah_relay_client.dylib $OUTPUT/x86_64/libcheetah_relay_client.bundle
