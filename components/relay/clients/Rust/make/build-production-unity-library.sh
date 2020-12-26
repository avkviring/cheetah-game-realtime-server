#!/bin/bash

rustup default stable

OUTPUT=Clients/Unity/Packages/CheetahRelay/

cd ../../../


# macos
rm -f $OUTPUT/x86_64/libcheetah_relay_client.dylib
cargo build --manifest-path clients/Rust/Cargo.toml  --release --message-format short
mkdir -p $OUTPUT/x86_64/
cp target/release/libcheetah_relay_client.dylib $OUTPUT/x86_64/libcheetah_relay_client.bundle

# android
rm -f $OUTPUT/Android/armv7-linux-androideabi/libcheetah_relay_client.so
mkdir -p $OUTPUT/Android/
cross build --manifest-path clients/Rust/Cargo.toml --target armv7-linux-androideabi --release --message-format short
cp target/armv7-linux-androideabi/release/libcheetah_relay_client.so $OUTPUT/Android/

# windows
rm -f $OUTPUT/x86_64/libcheetah_relay_client.dll
cross build --manifest-path clients/Rust/Cargo.toml --target x86_64-pc-windows-gnu --release --message-format short
cp target/x86_64-pc-windows-gnu/release/cheetah_relay_client.dll $OUTPUT/x86_64/libcheetah_relay_client.dll

# linux
rm -f $OUTPUT/x86_64/libcheetah_relay_client.so
cross build --manifest-path clients/Rust/Cargo.toml --target x86_64-unknown-linux-gnu --release --message-format short
cp target/x86_64-unknown-linux-gnu/release/libcheetah_relay_client.so $OUTPUT/x86_64/libcheetah_relay_client.so
