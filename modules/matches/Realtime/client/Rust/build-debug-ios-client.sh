#!/bin/bash
OUTPUT="../Unity/games.cheetah.matches.realtime/"
rm -f $OUTPUT/x86_64/libcheetah_matches_realtime_client.bundle
cargo build --target aarch64-apple-ios
cp ../../../../target/aarch64-apple-ios/debug/libcheetah_matches_realtime_client.a "$OUTPUT/Runtime/Library/ios.a"
