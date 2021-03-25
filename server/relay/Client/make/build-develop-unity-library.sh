#!/bin/bash
cd ../../
OUTPUT="../../clients/Unity/Packages/games.cheetah.relay/"
## macos
rm -f $OUTPUT/x86_64/libcheetah_relay_client.bundle
cargo build --manifest-path Client/Cargo.toml --release
cp target/release/libcheetah_relay_client.dylib "$OUTPUT/x86_64/libcheetah_relay_client.bundle"

## armv7
cross build --manifest-path Client/Cargo.toml --target armv7-linux-androideabi --release --message-format short
cp target/armv7-linux-androideabi/release/libcheetah_relay_client.so "$OUTPUT/Android/armv7/libcheetah_relay_client.so"

## armv64
cross build --manifest-path Client/Cargo.toml --target armv7-linux-androideabi --release --message-format short
cp target/aarch64-linux-android/release/libcheetah_relay_client.so "$OUTPUT/Android/aarch64/libcheetah_relay_client.so"