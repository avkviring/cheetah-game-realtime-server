#!/bin/bash

UNITY_PLUGINS_DIR=../Unity/Assets/CheetahRelay/

cargo build
rm -f "$UNITY_PLUGINS_DIR"/x86_64/libcheetah_relay_client.dylib
mkdir -p $UNITY_PLUGINS_DIR"/x86_64/"
cp target/debug/libcheetah_relay_client.dylib "$UNITY_PLUGINS_DIR"/x86_64/libcheetah_relay_client.bundle

#rm -f ../Rust/Assets/Plugins/Android/librelay.so
#cross build --target armv7-linux-androideabi --release
#cp target/armv7-linux-androideabi/release/librelay.so ../Rust/Assets/Plugins/Android/
#
#rm -f ../Rust/Assets/Plugins/x86_64/relay.dll
#cross build --target x86_64-pc-windows-gnu --release
#cp target/x86_64-pc-windows-gnu/release/relay.dll ../Rust/Assets/Plugins/x86_64/
