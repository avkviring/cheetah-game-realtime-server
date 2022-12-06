#!/bin/sh
set -e

UnityProject="../client/Unity/"

cargo build --manifest-path Client/Cargo.toml
cargo build --manifest-path Embedded/Cargo.toml
cargo build --manifest-path Plugin/Cargo.toml


cp target/debug/libcheetah_client.dylib "$UnityProject/Packages/games.cheetah.client/Runtime/Library/macos.bundle"
cp target/debug/libcheetah_embedded.dylib "$UnityProject/Packages/games.cheetah.embedded-server/Runtime/FFI/Library/libcheetah_embedded.bundle"
cp target/debug/libcheetah_plugin.dylib "$UnityProject/Packages/games.cheetah.uds/Runtime/FFI/Library/libcheetah_plugin.bundle"