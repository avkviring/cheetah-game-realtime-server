#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../client/Unity/games.cheetah.matches.realtime.embedded-server/"
cargo build

cp ../../../target/debug/cheetah_matches_realtime_embedded.dll "$OUTPUT/Runtime/FFI/Library/"
cp ../../../target/debug/cheetah_matches_realtime_embedded.pdb "$OUTPUT/Runtime/FFI/Library/"
