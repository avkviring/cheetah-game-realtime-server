#!/bin/bash
## сборка runtime клиента для тестирования на macos
OUTPUT="../client/Unity/games.cheetah.matches.realtime.embedded-server/"
cd ../../../
cross build --manifest-path matches/Realtime/Embedded/Cargo.toml --target x86_64-unknown-linux-gnu
#cp ../../../target/debug/libcheetah_matches_realtime_embedded.dylib
# "$OUTPUT/Runtime/FFI/Library/embedded_realtime_server.bundle"
