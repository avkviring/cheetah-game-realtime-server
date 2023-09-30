#!/bin/sh
set -e

UnityProject="../client/Unity/"
NetProject="../client/Net/Embedded/Src/"

# macos
cd ../
cargo build --manifest-path rust/Cargo.toml --target x86_64-apple-darwin
cargo build --manifest-path rust/Cargo.toml --target aarch64-apple-darwin

cd rust

lipo -create -output target/libcheetah_client.dylib target/aarch64-apple-darwin/debug/libcheetah_client.dylib  target/x86_64-apple-darwin/debug/libcheetah_client.dylib
lipo -create -output target/libcheetah_embedded.dylib target/aarch64-apple-darwin/debug/libcheetah_embedded.dylib  target/x86_64-apple-darwin/debug/libcheetah_embedded.dylib

cp target/libcheetah_client.dylib "$UnityProject/Packages/games.cheetah.client/Runtime/Library/libcheetah_client.bundle"
cp target/libcheetah_embedded.dylib "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/libcheetah_embedded.bundle"
cp target/libcheetah_embedded.dylib "$NetProject"




# linux
#cd ../
#cross build --manifest-path rust/Cargo.toml --target x86_64-unknown-linux-gnu
#cd rust
#cp target/x86_64-unknown-linux-gnu/debug/libcheetah_client.so "$UnityProject/Packages/games.cheetah.client/Runtime/Library/"
#cp target/x86_64-unknown-linux-gnu/debug/libcheetah_embedded.so "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/"
#cp target/x86_64-unknown-linux-gnu/debug/libcheetah_embedded.so "$NetProject/"

# windows
#cd ../
#cross build --manifest-path rust/Cargo.toml --target x86_64-pc-windows-gnu
#cd rust
#cp target/x86_64-pc-windows-gnu/debug/cheetah_client.dll "$UnityProject/Packages/games.cheetah.client/Runtime/Library/"
#cp target/x86_64-pc-windows-gnu/debug/cheetah_embedded.dll "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/"
#cp target/x86_64-pc-windows-gnu/debug/cheetah_embedded.dll "$NetProject/"

