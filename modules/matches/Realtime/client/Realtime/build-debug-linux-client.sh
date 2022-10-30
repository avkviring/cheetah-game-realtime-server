#!/bin/bash
set -e

# project dir
cd ../../../../../
cd modules
cross build --manifest-path matches/Realtime/client/Rust/Cargo.toml --target x86_64-unknown-linux-gnu
cd ../
cp modules/target/x86_64-unknown-linux-gnu/debug/libcheetah_matches_realtime_client.so "modules/matches/Realtime/client/Unity/games.cheetah.matches.realtime/Runtime/Library/linux.so"
