#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../../Unity/games.cheetah.matches.realtime.uds/"
cargo build
cp ../../../../../target/debug/libcheetah_matches_realtime_uds_client.dylib "$OUTPUT/Runtime/FFI/Library/libcheetah_matches_realtime_uds_client.bundle"
