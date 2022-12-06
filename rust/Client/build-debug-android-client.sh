#!/bin/bash
OUTPUT="../../../../../../client/Unity/Packages/games.cheetah.matches.realtime"
cd ../../../../
cross build --manifest-path matches/Realtime/client/rust/Realtime/Cargo.toml --target armv7-linux-androideabi
cross build --manifest-path matches/Realtime/client/rust/Realtime/Cargo.toml --target aarch64-linux-android
cd matches/Realtime/client/rust/Realtime/
mkdir -p "${OUTPUT}/Runtime/Library/android-armv7"
mkdir -p "${OUTPUT}/Runtime/Library/android-aarch64"
cp -rf ../../../../target/armv7-linux-androideabi/debug/libcheetah_matches_realtime_client.so "$OUTPUT/Runtime/Library/android-armv7/libcheetah_matches_realtime_client.so"
cp -rf ../../../../target/aarch64-linux-android/debug/libcheetah_matches_realtime_client.so "$OUTPUT/Runtime/Library/android-aarch64/libcheetah_matches_realtime_client.so"