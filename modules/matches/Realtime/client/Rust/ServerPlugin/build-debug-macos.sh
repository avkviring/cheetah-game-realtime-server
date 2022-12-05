#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../../../../../../client/Unity/Packages/games.cheetah.uds/"
cargo build
cp ../../../../../target/debug/libcheetah_matches_realtime_server_plugin.dylib "$OUTPUT/Runtime/FFI/Library/libcheetah_matches_realtime_server_plugin.bundle"
