#!/bin/bash
cd ../../
OUTPUT="../../clients/Unity/Packages/games.cheetah.relay/"
## macos
rm -f $OUTPUT/x86_64/libcheetah_relay_client.bundle
cargo build --manifest-path Client/Cargo.toml --release
cp target/release/libcheetah_relay_client.dylib "$OUTPUT/x86_64/libcheetah_relay_client.bundle"