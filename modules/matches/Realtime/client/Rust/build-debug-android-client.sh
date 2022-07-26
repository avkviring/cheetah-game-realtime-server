#!/bin/bash
OUTPUT="../Unity/games.cheetah.matches.realtime/"
cd ../../../ && cross build --manifest-path matches/Realtime/client/Rust/Cargo.toml  --target armv7-linux-androideabi
cp ../../../../target/armv7-linux-androideabi/debug/libcheetah_matches_realtime_client.so "$OUTPUT/Runtime/Library/android-armv7.so"