#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../client/Unity/games.cheetah.matches.realtime.embedded-server/"
cargo build

cp ../../../target/debug/libcheetah_matches_realtime_embedded.dylib "$OUTPUT/Runtime/FFI/Library/libcheetah_matches_realtime_embedded.bundle"
