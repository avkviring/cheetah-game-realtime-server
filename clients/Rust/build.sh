#!/bin/bash

UNITY_PLUGINS_DIR=../../unity/Client/Assets/Plugins/

cargo build
rm -f "$UNITY_PLUGINS_DIR"/x86_64/librelay.bundle
cp target/debug/librelay.dylib "$UNITY_PLUGINS_DIR"/x86_64/librelay.bundle

#rm -f ../Rust/Assets/Plugins/Android/librelay.so
#cross build --target armv7-linux-androideabi --release
#cp target/armv7-linux-androideabi/release/librelay.so ../Rust/Assets/Plugins/Android/
#
#rm -f ../Rust/Assets/Plugins/x86_64/relay.dll
#cross build --target x86_64-pc-windows-gnu --release
#cp target/x86_64-pc-windows-gnu/release/relay.dll ../Rust/Assets/Plugins/x86_64/
