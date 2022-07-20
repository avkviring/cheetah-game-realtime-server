#!/bin/bash
OUTPUT="../../../clients/Unity/Packages/games.cheetah.matches.relay/"
cargo build --manifest-path Client/Cargo.toml --target aarch64-apple-ios --release

cp ../../target/aarch64-apple-ios/release/libcheetah_matches_relay_client.a "$OUTPUT/iOS/libcheetah_matches_relay_client.a"
