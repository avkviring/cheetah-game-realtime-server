#!/bin/bash
## сборка runtime клиента для тестирования на macos
UNITY_OUTPUT="../../Unity/games.cheetah.matches.realtime.embedded-server/Runtime/FFI/Library/"
NET_OUTPUT="../../Net/Embedded/Libraries/"
cargo build --release

cp ../../../../../target/release/libcheetah_matches_realtime_embedded.dylib "$UNITY_OUTPUT/libcheetah_matches_realtime_embedded.bundle"
#cp ../../../../../target/release/libcheetah_matches_realtime_embedded.dylib  "$NET_OUTPUT/libcheetah_matches_realtime_embedded.dylib"
