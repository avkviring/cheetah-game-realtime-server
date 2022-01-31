#!/bin/bash

## сборка relay клиента для тестирования на macos
OUTPUT="../../../clients/Unity/Packages/games.cheetah.matches.relay/"
## macos
rm -f $OUTPUT/x86_64/libcheetah_matches_relay_client.bundle
cargo build --manifest-path Client/Cargo.toml
cp ../../target/debug/libcheetah_matches_relay_client.dylib "$OUTPUT/x86_64/libcheetah_matches_relay_client.bundle"