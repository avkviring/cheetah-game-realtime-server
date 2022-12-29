#!/bin/sh
set -e

UnityProject="../client/Unity/"
NetProject="../client/Net/Embedded/Src/"

cd ../
# macos поддерживается только на macos
cargo build --manifest-path rust/Cargo.toml --target x86_64-apple-darwin
cross build --manifest-path rust/Cargo.toml --target x86_64-unknown-linux-gnu
#cross build --manifest-path rust/Cargo.toml --target x86_64-pc-windows-gnu

cd rust

# macos
cp target/x86_64-apple-darwin/debug/libcheetah_client.dylib "$UnityProject/Packages/games.cheetah.client/Runtime/Library/libcheetah_client.bundle"
cp target/x86_64-apple-darwin/debug/libcheetah_embedded.dylib "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/libcheetah_embedded.bundle"
cp target/x86_64-apple-darwin/debug/libcheetah_plugin.dylib "$UnityProject/Packages/games.cheetah.uds/Runtime/FFI/Library/libcheetah_plugin.bundle"
cp target/x86_64-apple-darwin/debug/libcheetah_embedded.dylib "$NetProject"

# linux
cp target/x86_64-unknown-linux-gnu/debug/libcheetah_client.so "$UnityProject/Packages/games.cheetah.client/Runtime/Library/"
cp target/x86_64-unknown-linux-gnu/debug/libcheetah_embedded.so "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/"
cp target/x86_64-unknown-linux-gnu/debug/libcheetah_plugin.so "$UnityProject/Packages/games.cheetah.uds/Runtime/FFI/Library/"
cp target/x86_64-unknown-linux-gnu/debug/libcheetah_embedded.so "$NetProject/"

# windows
cp target/x86_64-pc-windows-gnu/debug/cheetah_client.dll "$UnityProject/Packages/games.cheetah.client/Runtime/Library/"
cp target/x86_64-pc-windows-gnu/debug/cheetah_embedded.dll "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/"
cp target/x86_64-pc-windows-gnu/debug/cheetah_plugin.dll "$UnityProject/Packages/games.cheetah.uds/Runtime/FFI/Library/"
cp target/x86_64-pc-windows-gnu/debug/cheetah_embedded.dll "$NetProject/"

